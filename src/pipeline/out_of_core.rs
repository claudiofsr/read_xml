//! # Motor Out-of-Core em Disco (`-b` / `--buffer`)
//!
//! Arquitetura para escalar de milhões a bilhões de XMLs dentro de `.zip`s mantendo a RAM limitada:
//! 1. **Ingestão em Hash Shards (`NUM_SHARDS`)**: cada XML é lido uma única vez do `.zip` e gravado
//!    em partições comprimidas (`postcard` + `zstd`) pelo hash da chave.
//! 2. **Resolução por Partição**: aplica eventos/cancelamentos, deduplica e extrai resumos compactos
//!    para o grafo CT-e ↔ NF-e e votos de canonicalização por CNPJ.
//! 3. **Correlação em Grafo & Canonicalização**: resolve componentes conexas sobre os resumos compactos
//!    e particiona enriquecimentos de volta para os shards.
//! 4. **Ordenação Local + K-Way Merge Sort**: ordena cada partição e mescla em streaming puro (`BinaryHeap`)
//!    para CSV, XLSX (`ExcelMemoryMode::Constant`) e arquivos de chaves (`-l`).

use claudiofsr_lib::HashSetExtension;
use indicatif::MultiProgress;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    Arguments, CachedCompNfeSummary, CsvWriter, CteComponents, ExcelMemoryMode, FiscalSampler,
    InfoExtension, InfoReceita, Information, MultiProgressBar, NumberFormat, OutputFilename,
    RECEITA_ND, ReceitaOptions, SampleConfig, SanitizeText, UniqueIdentification, XmlParserResult,
    adicionar_eventos_cte, adicionar_eventos_nfe, cmp_info_cte, cmp_info_efin, cmp_info_nfe,
    consolidate_ncm_items, consolidate_payer_items, convert_cte_to_receita,
    convert_nfe_chunk_to_receita, for_each_xml, get_xml_serialized_from_bytes,
    partition_sources_with_xml,
    pipeline::{
        ExecutionCounts, GeneratedFileSummary,
        memory::process_and_export_docs,
        print_execution_summary, sanitize_slice,
        spill_codec::{
            KWayMergeIter, MergeOrder, NUM_SHARDS, ShardedZstdWriters, SpillCte, SpillDir,
            SpillEFinanceira, SpillNfe, encode_framed, read_vec_zst, shard_index, write_vec_zst,
        },
    },
    resolve_canonical_map, show_docs, sort_receita_rows, vote_cte_names, write_xlsx_from_iter,
    xml_structs::{
        cancelamento_cte::InfoCteCancel, cancelamento_nfe::InfoNfeCancel, cte::InfoCte,
        cte_evento::InfoCteEvento, efinanceira::InfoEFinanceira, nfe::InfoNfe,
        nfe_evento::InfoNfeEvento,
    },
};

const FLUSH_THRESHOLD_BYTES: usize = 512 * 1024;

#[derive(Serialize, Deserialize)]
struct CteGraphSummary {
    cte: String,
    valid: bool,
    cte_refs: Vec<String>,
    nfe_refs: Vec<String>,
    tomador_base: Option<String>,
    tomador_codigo: Option<u8>,
    valor_total: Option<f64>,
}

#[derive(Serialize, Deserialize)]
struct NfeDocSummary {
    nfe: String,
    valid: bool,
    items: Vec<(Option<String>, Option<String>, Option<f64>)>,
    cte_refs: Vec<String>,
    nfe_refs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CteEnrich {
    cte: String,
    ctes_relacionados: Vec<String>,
    nfes: Vec<String>,
    ncm_descricao: Vec<String>,
    valor_total_nfes: Option<f64>,
}

#[derive(Serialize, Deserialize)]
struct NfeEnrich {
    nfe: String,
    ctes: Vec<String>,
    nfes_relacionadas: Vec<String>,
    tomadores: Vec<String>,
    valor_total_ctes: Option<f64>,
}

struct WorkerBuffers {
    cte_docs: Vec<Vec<u8>>,
    cte_ev: Vec<Vec<u8>>,
    cte_canc: Vec<Vec<u8>>,
    nfe_docs: Vec<Vec<u8>>,
    nfe_ev: Vec<Vec<u8>>,
    nfe_canc: Vec<Vec<u8>>,
    efin: Vec<Vec<u8>>,
}

impl WorkerBuffers {
    fn new() -> Self {
        let mk = || (0..NUM_SHARDS).map(|_| Vec::new()).collect();
        Self {
            cte_docs: mk(),
            cte_ev: mk(),
            cte_canc: mk(),
            nfe_docs: mk(),
            nfe_ev: mk(),
            nfe_canc: mk(),
            efin: mk(),
        }
    }
}

struct SpillTargets<'a> {
    cte_docs: &'a ShardedZstdWriters,
    cte_ev: &'a ShardedZstdWriters,
    cte_canc: &'a ShardedZstdWriters,
    nfe_docs: &'a ShardedZstdWriters,
    nfe_ev: &'a ShardedZstdWriters,
    nfe_canc: &'a ShardedZstdWriters,
    efin: &'a ShardedZstdWriters,
}

fn flush_bucket(
    buf: &mut Vec<u8>,
    writers: &ShardedZstdWriters,
    shard: usize,
    force: bool,
) -> XmlParserResult<()> {
    if !buf.is_empty() && (force || buf.len() >= FLUSH_THRESHOLD_BYTES) {
        writers.write_chunk(shard, buf)?;
        buf.clear();
    }
    Ok(())
}

fn flush_all_worker_buffers(wb: &mut WorkerBuffers, targets: &SpillTargets<'_>) {
    for p in 0..NUM_SHARDS {
        let _ = flush_bucket(&mut wb.cte_docs[p], targets.cte_docs, p, true);
        let _ = flush_bucket(&mut wb.cte_ev[p], targets.cte_ev, p, true);
        let _ = flush_bucket(&mut wb.cte_canc[p], targets.cte_canc, p, true);
        let _ = flush_bucket(&mut wb.nfe_docs[p], targets.nfe_docs, p, true);
        let _ = flush_bucket(&mut wb.nfe_ev[p], targets.nfe_ev, p, true);
        let _ = flush_bucket(&mut wb.nfe_canc[p], targets.nfe_canc, p, true);
        let _ = flush_bucket(&mut wb.efin[p], targets.efin, p, true);
    }
}

pub fn run_out_of_core(zip_paths: &[PathBuf], arguments: &Arguments) -> XmlParserResult<()> {
    if let Some(sample_cfg) = SampleConfig::from_args(arguments)? {
        return run_sampled_out_of_core(zip_paths, arguments, sample_cfg);
    }
    let (active_zips, skipped_zips, total_entries) = partition_sources_with_xml(zip_paths);
    if active_zips.is_empty() {
        eprintln!(
            "Aviso: nenhum arquivo .zip com XML ou .xml solto foi encontrado no caminho informado."
        );
        return Ok(());
    }
    if skipped_zips > 0 && arguments.verbose {
        eprintln!("Info: {skipped_zips} arquivo(s) .zip sem XML ignorado(s).");
    }

    let spill = SpillDir::create()?;
    let multi_progress = MultiProgress::new();
    let mut multi_progressbar = MultiProgressBar::default();
    multi_progressbar.add_parse_xml(&multi_progress, total_entries as usize)?;

    // =========================================================================
    // ESTÁGIO 1: Ingestão Paralela dos ZIPs -> Shards Comprimidos em Disco
    // =========================================================================
    let cte_docs_w = ShardedZstdWriters::new(&spill, "raw_cte")?;
    let cte_ev_w = ShardedZstdWriters::new(&spill, "ev_cte")?;
    let cte_canc_w = ShardedZstdWriters::new(&spill, "canc_cte")?;
    let nfe_docs_w = ShardedZstdWriters::new(&spill, "raw_nfe")?;
    let nfe_ev_w = ShardedZstdWriters::new(&spill, "ev_nfe")?;
    let nfe_canc_w = ShardedZstdWriters::new(&spill, "canc_nfe")?;
    let efin_w = ShardedZstdWriters::new(&spill, "raw_efin")?;

    let targets = SpillTargets {
        cte_docs: &cte_docs_w,
        cte_ev: &cte_ev_w,
        cte_canc: &cte_canc_w,
        nfe_docs: &nfe_docs_w,
        nfe_ev: &nfe_ev_w,
        nfe_canc: &nfe_canc_w,
        efin: &efin_w,
    };

    let xml_counter = AtomicUsize::new(0);
    let ev_cte_counter = AtomicUsize::new(0);
    let ev_nfe_counter = AtomicUsize::new(0);
    let canc_cte_counter = AtomicUsize::new(0);
    let canc_nfe_counter = AtomicUsize::new(0);

    thread_local! {
        static LOCAL_WB: RefCell<Option<WorkerBuffers>> = const { RefCell::new(None) };
    }

    for_each_xml(
        &active_zips,
        Some(&multi_progressbar.show_parse),
        |label, bytes| {
            let idx = xml_counter.fetch_add(1, Ordering::Relaxed) + 1;
            let Some(info) = get_xml_serialized_from_bytes(label, bytes, arguments) else {
                return;
            };
            if arguments.verbose {
                println!("xml {idx} ({label}): {info:#?}\n");
            }

            LOCAL_WB.with(|cell| {
                let mut opt = cell.borrow_mut();
                let wb = opt.get_or_insert_with(WorkerBuffers::new);
                match info {
                    Information::Cte(c) => {
                        let s = shard_index(&c.cte);
                        let spill_c = SpillCte::from(*c);
                        let _ = encode_framed(&spill_c, &mut wb.cte_docs[s]);
                        let _ = flush_bucket(&mut wb.cte_docs[s], targets.cte_docs, s, false);
                    }
                    Information::Nfe(items) => {
                        for n in items {
                            let s = shard_index(&n.nfe);
                            let spill_n = SpillNfe::from(n);
                            let _ = encode_framed(&spill_n, &mut wb.nfe_docs[s]);
                            let _ = flush_bucket(&mut wb.nfe_docs[s], targets.nfe_docs, s, false);
                        }
                    }
                    Information::EventoCte(e) => {
                        ev_cte_counter.fetch_add(1, Ordering::Relaxed);
                        let s = shard_index(&e.cte);
                        let _ = encode_framed(&*e, &mut wb.cte_ev[s]);
                        let _ = flush_bucket(&mut wb.cte_ev[s], targets.cte_ev, s, false);
                    }
                    Information::EventoNfe(e) => {
                        ev_nfe_counter.fetch_add(1, Ordering::Relaxed);
                        let s = shard_index(&e.nfe);
                        let _ = encode_framed(&*e, &mut wb.nfe_ev[s]);
                        let _ = flush_bucket(&mut wb.nfe_ev[s], targets.nfe_ev, s, false);
                    }
                    Information::CancelamentoCte(c) => {
                        canc_cte_counter.fetch_add(1, Ordering::Relaxed);
                        let s = shard_index(&c.cte);
                        let _ = encode_framed(&*c, &mut wb.cte_canc[s]);
                        let _ = flush_bucket(&mut wb.cte_canc[s], targets.cte_canc, s, false);
                    }
                    Information::CancelamentoNfe(c) => {
                        canc_nfe_counter.fetch_add(1, Ordering::Relaxed);
                        let s = shard_index(&c.nfe);
                        let _ = encode_framed(&*c, &mut wb.nfe_canc[s]);
                        let _ = flush_bucket(&mut wb.nfe_canc[s], targets.nfe_canc, s, false);
                    }
                    Information::EFinanceira(efs) => {
                        for ef in efs {
                            let s = shard_index(&ef.cnpj_do_declarante);
                            let spill_e = SpillEFinanceira::from(ef);
                            let _ = encode_framed(&spill_e, &mut wb.efin[s]);
                            let _ = flush_bucket(&mut wb.efin[s], targets.efin, s, false);
                        }
                    }
                    Information::None => {}
                }
            });
        },
    )?;

    // Garante flush de todas as threads trabalhadoras do Rayon
    rayon::broadcast(|_| {
        LOCAL_WB.with(|cell| {
            if let Some(mut wb) = cell.borrow_mut().take() {
                flush_all_worker_buffers(&mut wb, &targets);
            }
        });
    });
    LOCAL_WB.with(|cell| {
        if let Some(mut wb) = cell.borrow_mut().take() {
            flush_all_worker_buffers(&mut wb, &targets);
        }
    });

    multi_progressbar.show_parse.finish();

    cte_docs_w.finish_all()?;
    cte_ev_w.finish_all()?;
    cte_canc_w.finish_all()?;
    nfe_docs_w.finish_all()?;
    nfe_ev_w.finish_all()?;
    nfe_canc_w.finish_all()?;
    efin_w.finish_all()?;

    let mut counts = ExecutionCounts {
        xml_total: xml_counter.load(Ordering::Relaxed),
        eventos_cte: ev_cte_counter.load(Ordering::Relaxed),
        eventos_nfe: ev_nfe_counter.load(Ordering::Relaxed),
        cancel_cte: canc_cte_counter.load(Ordering::Relaxed),
        cancel_nfe: canc_nfe_counter.load(Ordering::Relaxed),
        ..Default::default()
    };

    // =========================================================================
    // ESTÁGIO 2: Resolução Local por Partição (Eventos, Cancelamentos, Unique)
    // =========================================================================
    let need_item_aggregations = !arguments.receita || arguments.exibir_correlacoes;
    let need_nfe_enrich = !arguments.receita || !arguments.diversos || arguments.exibir_correlacoes;
    let mut cte_summaries: Vec<CteGraphSummary> = Vec::new();
    let mut nfe_summaries: Vec<NfeDocSummary> = Vec::new();
    let mut cte_contrib_votes: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut cte_part_votes: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for p in 0..NUM_SHARDS {
        let raw_ctes: Vec<SpillCte> = read_vec_zst(&spill.file(&format!("raw_cte_{p:03}.zst")))?;
        if !raw_ctes.is_empty() {
            let mut ctes: Vec<InfoCte> = raw_ctes.into_iter().map(InfoCte::from).collect();
            let ev_ctes: Vec<InfoCteEvento> =
                read_vec_zst(&spill.file(&format!("ev_cte_{p:03}.zst")))?;
            let canc_ctes: Vec<InfoCteCancel> =
                read_vec_zst(&spill.file(&format!("canc_cte_{p:03}.zst")))?;

            adicionar_eventos_cte(&mut ctes, &ev_ctes, &canc_ctes);
            let mut ctes = ctes.get_unique_id();
            ctes.par_iter_mut().for_each(InfoCte::get_unique_elements);

            counts.ctes_total += ctes.len();
            counts.ctes_cancelados += ctes.iter().filter(|c| c.is_canceled()).count();

            for c in &ctes {
                if let Some(key) = &c.cte {
                    cte_summaries.push(CteGraphSummary {
                        cte: key.clone(),
                        valid: c.is_valid(),
                        cte_refs: c.get_correlated_ctes(),
                        nfe_refs: c.get_correlated_nfes(),
                        tomador_base: need_item_aggregations
                            .then(|| c.get_cnpj_cpf_base_do_tomador())
                            .flatten(),
                        tomador_codigo: need_item_aggregations
                            .then_some(c.tomador_codigo)
                            .flatten(),
                        valor_total: need_item_aggregations.then_some(c.valor_total).flatten(),
                    });
                }
                if arguments.receita {
                    vote_cte_names(
                        c,
                        arguments.clean_spaces,
                        &mut cte_contrib_votes,
                        &mut cte_part_votes,
                    );
                }
            }

            let spill_out: Vec<SpillCte> = ctes.into_iter().map(SpillCte::from).collect();
            write_vec_zst(&spill.file(&format!("res_cte_{p:03}.zst")), &spill_out)?;
        }

        let _ = std::fs::remove_file(spill.file(&format!("raw_cte_{p:03}.zst")));
        let _ = std::fs::remove_file(spill.file(&format!("ev_cte_{p:03}.zst")));
        let _ = std::fs::remove_file(spill.file(&format!("canc_cte_{p:03}.zst")));
    }

    for p in 0..NUM_SHARDS {
        let raw_nfes: Vec<SpillNfe> = read_vec_zst(&spill.file(&format!("raw_nfe_{p:03}.zst")))?;
        if !raw_nfes.is_empty() {
            let mut nfes: Vec<InfoNfe> = raw_nfes.into_iter().map(InfoNfe::from).collect();
            let ev_nfes: Vec<InfoNfeEvento> =
                read_vec_zst(&spill.file(&format!("ev_nfe_{p:03}.zst")))?;
            let canc_nfes: Vec<InfoNfeCancel> =
                read_vec_zst(&spill.file(&format!("canc_nfe_{p:03}.zst")))?;

            adicionar_eventos_nfe(&mut nfes, &ev_nfes, &canc_nfes);
            let mut nfes = nfes.get_unique_id();
            nfes.par_iter_mut().for_each(InfoNfe::get_unique_elements);

            counts.nfes_itens += nfes.len();
            let mut shard_docs: HashMap<&str, usize> = HashMap::new();
            let mut shard_summaries: Vec<NfeDocSummary> = Vec::new();
            let mut canceled_keys = HashSet::new();

            for n in &nfes {
                if let Some(ch) = n.nfe.as_deref() {
                    if n.is_canceled() {
                        canceled_keys.insert(ch);
                    }
                    if let Some(&idx) = shard_docs.get(ch) {
                        let doc = &mut shard_summaries[idx];
                        doc.valid |= n.is_valid();
                        if need_item_aggregations {
                            doc.items
                                .push((n.ncm.clone(), n.descricao.clone(), n.v_prod));
                        }
                    } else {
                        let idx = shard_summaries.len();
                        shard_docs.insert(ch, idx);
                        shard_summaries.push(NfeDocSummary {
                            nfe: ch.to_string(),
                            valid: n.is_valid(),
                            items: if need_item_aggregations {
                                vec![(n.ncm.clone(), n.descricao.clone(), n.v_prod)]
                            } else {
                                Vec::new()
                            },
                            cte_refs: if need_nfe_enrich {
                                n.ctes.clone()
                            } else {
                                Vec::new()
                            },
                            nfe_refs: if need_nfe_enrich {
                                n.nfes_relacionadas.clone()
                            } else {
                                Vec::new()
                            },
                        });
                    }
                }
            }
            counts.nfes_unicas += shard_docs.len();
            counts.nfes_canceladas += canceled_keys.len();
            nfe_summaries.extend(shard_summaries);

            let spill_out: Vec<SpillNfe> = nfes.into_iter().map(SpillNfe::from).collect();
            write_vec_zst(&spill.file(&format!("res_nfe_{p:03}.zst")), &spill_out)?;
        }

        let _ = std::fs::remove_file(spill.file(&format!("raw_nfe_{p:03}.zst")));
        let _ = std::fs::remove_file(spill.file(&format!("ev_nfe_{p:03}.zst")));
        let _ = std::fs::remove_file(spill.file(&format!("canc_nfe_{p:03}.zst")));
    }

    for p in 0..NUM_SHARDS {
        let raw_ef: Vec<SpillEFinanceira> =
            read_vec_zst(&spill.file(&format!("raw_efin_{p:03}.zst")))?;
        if !raw_ef.is_empty() {
            let efs: Vec<InfoEFinanceira> = raw_ef.into_iter().map(InfoEFinanceira::from).collect();
            let efs = efs.get_unique_id();
            counts.efin_total += efs.len();
            let spill_out: Vec<SpillEFinanceira> =
                efs.into_iter().map(SpillEFinanceira::from).collect();
            write_vec_zst(&spill.file(&format!("res_efin_{p:03}.zst")), &spill_out)?;
        }
        let _ = std::fs::remove_file(spill.file(&format!("raw_efin_{p:03}.zst")));
    }

    // =========================================================================
    // ESTÁGIO 3: Resolução do Grafo CT-e ↔ NF-e sobre Resumos Compactos
    // =========================================================================
    resolve_compact_graph(&spill, arguments, &cte_summaries, &nfe_summaries)?;
    drop(cte_summaries);
    drop(nfe_summaries);

    let contrib_canon = resolve_canonical_map(cte_contrib_votes);
    let part_canon = resolve_canonical_map(cte_part_votes);

    // =========================================================================
    // ESTÁGIO 4: Chaves (-l) + Ordenação Local + K-Way Merge Sort -> Saída
    // =========================================================================
    let mut generated_files: Vec<GeneratedFileSummary> = Vec::new();

    if let Some(size) = arguments.linhas {
        let size_u: usize = size.try_into()?;
        if counts.ctes_total > 0 {
            for (p, count) in write_keys_kway_ctes(&spill, "ctes", size_u)? {
                generated_files.push(GeneratedFileSummary::new(
                    "TXT",
                    "CTes (chaves)",
                    &p,
                    count,
                    None,
                ));
            }
        }
        if counts.nfes_unicas > 0 {
            for (p, count) in write_keys_kway_nfes(&spill, "nfes", size_u)? {
                generated_files.push(GeneratedFileSummary::new(
                    "TXT",
                    "NFes (chaves)",
                    &p,
                    count,
                    None,
                ));
            }
        }
    }

    if arguments.csv || !arguments.avoid {
        let total_datasets = usize::from(counts.ctes_total > 0)
            + usize::from(counts.nfes_itens > 0)
            + usize::from(counts.efin_total > 0);

        if arguments.csv {
            multi_progressbar.add_print_csv(&multi_progress, total_datasets)?;
        }
        if !arguments.avoid {
            multi_progressbar.add_print_xls(&multi_progress, total_datasets)?;
        }

        let output = OutputFilename::default();
        let number_format = NumberFormat::from(arguments.format_br);
        let ctes_label = if arguments.receita {
            "CTes (Receita)"
        } else {
            "CTes"
        };
        let nfes_label = if arguments.receita {
            "NFes (Receita)"
        } else {
            "NFes"
        };

        if counts.ctes_total > 0 {
            export_ctes_ooc(
                &spill,
                arguments,
                number_format,
                &contrib_canon,
                &part_canon,
                &output.ctes,
                ctes_label,
                &mut multi_progressbar,
                &mut generated_files,
            )?;
        }

        if counts.nfes_itens > 0 {
            export_nfes_ooc(
                &spill,
                arguments,
                number_format,
                &output.nfes,
                nfes_label,
                &mut multi_progressbar,
                &mut generated_files,
            )?;
        }

        if counts.efin_total > 0 {
            export_efin_ooc(
                &spill,
                arguments,
                &output.efin,
                &mut multi_progressbar,
                &mut generated_files,
            )?;
        }

        if arguments.csv {
            multi_progressbar.show_csval.finish();
        }
        if !arguments.avoid {
            multi_progressbar.show_excel.finish();
        }
    }

    print_execution_summary(&counts, &generated_files);
    Ok(())
}

fn run_sampled_out_of_core(
    zip_paths: &[PathBuf],
    arguments: &Arguments,
    sample_cfg: SampleConfig,
) -> XmlParserResult<()> {
    let (active_zips, skipped_zips, total_entries) = partition_sources_with_xml(zip_paths);
    if active_zips.is_empty() {
        eprintln!(
            "Aviso: nenhum arquivo .zip com XML ou .xml solto foi encontrado no caminho informado."
        );
        return Ok(());
    }
    if skipped_zips > 0 && arguments.verbose {
        eprintln!("Info: {skipped_zips} arquivo(s) .zip sem XML ignorado(s).");
    }

    let spill = SpillDir::create()?;
    let multi_progress = MultiProgress::new();
    let mut multi_progressbar = MultiProgressBar::default();
    multi_progressbar.add_parse_xml(&multi_progress, total_entries as usize)?;

    let cte_ev_w = ShardedZstdWriters::new(&spill, "ev_cte")?;
    let cte_canc_w = ShardedZstdWriters::new(&spill, "canc_cte")?;
    let nfe_ev_w = ShardedZstdWriters::new(&spill, "ev_nfe")?;
    let nfe_canc_w = ShardedZstdWriters::new(&spill, "canc_nfe")?;

    let xml_counter = AtomicUsize::new(0);
    let ev_cte_counter = AtomicUsize::new(0);
    let ev_nfe_counter = AtomicUsize::new(0);
    let canc_cte_counter = AtomicUsize::new(0);
    let canc_nfe_counter = AtomicUsize::new(0);

    struct SampleWorkerState {
        sampler: FiscalSampler,
        cte_ev: Vec<Vec<u8>>,
        cte_canc: Vec<Vec<u8>>,
        nfe_ev: Vec<Vec<u8>>,
        nfe_canc: Vec<Vec<u8>>,
    }

    impl SampleWorkerState {
        fn new(cfg: SampleConfig) -> Self {
            let mk = || (0..NUM_SHARDS).map(|_| Vec::new()).collect();
            Self {
                sampler: FiscalSampler::new(cfg),
                cte_ev: mk(),
                cte_canc: mk(),
                nfe_ev: mk(),
                nfe_canc: mk(),
            }
        }
    }

    thread_local! {
        static LOCAL_SAMPLED_STATE: RefCell<Option<SampleWorkerState>> = const { RefCell::new(None) };
    }

    for_each_xml(
        &active_zips,
        Some(&multi_progressbar.show_parse),
        |label, bytes| {
            let idx = xml_counter.fetch_add(1, Ordering::Relaxed) + 1;
            let Some(info) = get_xml_serialized_from_bytes(label, bytes, arguments) else {
                return;
            };
            if arguments.verbose {
                println!("xml {idx} ({label}): {info:#?}\n");
            }

            LOCAL_SAMPLED_STATE.with(|cell| {
                let mut opt = cell.borrow_mut();
                let st = opt.get_or_insert_with(|| SampleWorkerState::new(sample_cfg));
                if let Some(ev) = st.sampler.push_or_pass_event(info) {
                    match ev {
                        Information::EventoCte(e) => {
                            ev_cte_counter.fetch_add(1, Ordering::Relaxed);
                            let s = shard_index(&e.cte);
                            let _ = encode_framed(&*e, &mut st.cte_ev[s]);
                            let _ = flush_bucket(&mut st.cte_ev[s], &cte_ev_w, s, false);
                        }
                        Information::EventoNfe(e) => {
                            ev_nfe_counter.fetch_add(1, Ordering::Relaxed);
                            let s = shard_index(&e.nfe);
                            let _ = encode_framed(&*e, &mut st.nfe_ev[s]);
                            let _ = flush_bucket(&mut st.nfe_ev[s], &nfe_ev_w, s, false);
                        }
                        Information::CancelamentoCte(c) => {
                            canc_cte_counter.fetch_add(1, Ordering::Relaxed);
                            let s = shard_index(&c.cte);
                            let _ = encode_framed(&*c, &mut st.cte_canc[s]);
                            let _ = flush_bucket(&mut st.cte_canc[s], &cte_canc_w, s, false);
                        }
                        Information::CancelamentoNfe(c) => {
                            canc_nfe_counter.fetch_add(1, Ordering::Relaxed);
                            let s = shard_index(&c.nfe);
                            let _ = encode_framed(&*c, &mut st.nfe_canc[s]);
                            let _ = flush_bucket(&mut st.nfe_canc[s], &nfe_canc_w, s, false);
                        }
                        _ => {}
                    }
                }
            });
        },
    )?;

    let global_sampler = std::sync::Mutex::new(FiscalSampler::new(sample_cfg));
    let flush_and_collect = |mut st: SampleWorkerState| {
        for p in 0..NUM_SHARDS {
            let _ = flush_bucket(&mut st.cte_ev[p], &cte_ev_w, p, true);
            let _ = flush_bucket(&mut st.cte_canc[p], &cte_canc_w, p, true);
            let _ = flush_bucket(&mut st.nfe_ev[p], &nfe_ev_w, p, true);
            let _ = flush_bucket(&mut st.nfe_canc[p], &nfe_canc_w, p, true);
        }
        if let Ok(mut g) = global_sampler.lock() {
            g.merge(st.sampler);
        }
    };

    rayon::broadcast(|_| {
        LOCAL_SAMPLED_STATE.with(|cell| {
            if let Some(st) = cell.borrow_mut().take() {
                flush_and_collect(st);
            }
        });
    });
    LOCAL_SAMPLED_STATE.with(|cell| {
        if let Some(st) = cell.borrow_mut().take() {
            flush_and_collect(st);
        }
    });

    multi_progressbar.show_parse.finish();
    cte_ev_w.finish_all()?;
    cte_canc_w.finish_all()?;
    nfe_ev_w.finish_all()?;
    nfe_canc_w.finish_all()?;

    let mut docs_fiscais = global_sampler
        .into_inner()
        .map_err(|_| crate::XmlParserError::Export("Mutex poisoned on sampler merge".into()))?
        .into_docs();

    let cte_shards: BTreeSet<usize> = docs_fiscais
        .ctes
        .iter()
        .map(|c| shard_index(&c.cte))
        .collect();
    for p in cte_shards {
        docs_fiscais
            .eventos_cte
            .extend(read_vec_zst::<InfoCteEvento>(
                &spill.file(&format!("ev_cte_{p:03}.zst")),
            )?);
        docs_fiscais
            .cancel_cte
            .extend(read_vec_zst::<InfoCteCancel>(
                &spill.file(&format!("canc_cte_{p:03}.zst")),
            )?);
    }

    let nfe_shards: BTreeSet<usize> = docs_fiscais
        .nfes
        .iter()
        .map(|n| shard_index(&n.nfe))
        .collect();
    for p in nfe_shards {
        docs_fiscais
            .eventos_nfe
            .extend(read_vec_zst::<InfoNfeEvento>(
                &spill.file(&format!("ev_nfe_{p:03}.zst")),
            )?);
        docs_fiscais
            .cancel_nfe
            .extend(read_vec_zst::<InfoNfeCancel>(
                &spill.file(&format!("canc_nfe_{p:03}.zst")),
            )?);
    }

    crate::retain_events_for_sampled_docs(&mut docs_fiscais);

    process_and_export_docs(
        docs_fiscais,
        xml_counter.load(Ordering::Relaxed),
        (
            ev_cte_counter.load(Ordering::Relaxed),
            ev_nfe_counter.load(Ordering::Relaxed),
            canc_cte_counter.load(Ordering::Relaxed),
            canc_nfe_counter.load(Ordering::Relaxed),
        ),
        arguments,
        Some(sample_cfg),
        Some(ExcelMemoryMode::Constant),
    )
}

fn flush_enrich_shard<T: Serialize>(
    item: &T,
    shard: usize,
    buffers: &mut [Vec<u8>],
    writers: &ShardedZstdWriters,
) -> XmlParserResult<()> {
    encode_framed(item, &mut buffers[shard])?;
    if buffers[shard].len() >= 64 * 1024 {
        writers.write_chunk(shard, &buffers[shard])?;
        buffers[shard].clear();
    }
    Ok(())
}

fn finish_enrich_shards(buffers: Vec<Vec<u8>>, writers: ShardedZstdWriters) -> XmlParserResult<()> {
    for (p, buf) in buffers.into_iter().enumerate() {
        if !buf.is_empty() {
            writers.write_chunk(p, &buf)?;
        }
    }
    writers.finish_all()
}

fn resolve_compact_graph(
    spill: &SpillDir,
    arguments: &Arguments,
    cte_summaries: &[CteGraphSummary],
    nfe_summaries: &[NfeDocSummary],
) -> XmlParserResult<()> {
    if cte_summaries.is_empty() && nfe_summaries.is_empty() {
        return Ok(());
    }

    let need_item_aggregations = !arguments.receita || arguments.exibir_correlacoes;
    let need_nfe_enrich = !arguments.receita || !arguments.diversos || arguments.exibir_correlacoes;

    let mut cte_valid: HashMap<&str, bool> = HashMap::with_capacity(cte_summaries.len());
    let mut cte_by_key: HashMap<&str, &CteGraphSummary> = if need_item_aggregations {
        HashMap::with_capacity(cte_summaries.len())
    } else {
        HashMap::new()
    };
    for c in cte_summaries {
        cte_valid.insert(c.cte.as_str(), c.valid);
        if need_item_aggregations {
            cte_by_key.insert(c.cte.as_str(), c);
        }
    }

    let mut nfe_valid: HashMap<&str, bool> = HashMap::with_capacity(nfe_summaries.len());
    let mut nfe_by_key: HashMap<&str, &NfeDocSummary> = if need_item_aggregations {
        HashMap::with_capacity(nfe_summaries.len())
    } else {
        HashMap::new()
    };
    for n in nfe_summaries {
        nfe_valid.insert(n.nfe.as_str(), n.valid);
        if need_item_aggregations {
            nfe_by_key.insert(n.nfe.as_str(), n);
        }
    }

    let mut ctes_nao_encontrados = HashSet::new();
    let mut nfes_nao_encontrados = HashSet::new();
    let mut cte_ctes: HashMap<&String, Vec<&String>> = HashMap::new();
    let check_missing = arguments.exibir_chaves_nao_encontradas && arguments.filtro.is_none();

    for c in cte_summaries {
        if !c.valid {
            continue;
        }
        let mut valid_ctes: Vec<&String> = Vec::new();
        for ch in &c.cte_refs {
            match cte_valid.get(ch.as_str()) {
                Some(&true) if ch != &c.cte => valid_ctes.push(ch),
                None if check_missing => {
                    ctes_nao_encontrados.insert(ch.clone());
                }
                _ => {}
            }
        }
        if !valid_ctes.is_empty() {
            cte_ctes.insert(&c.cte, valid_ctes);
        }

        if check_missing {
            for ch in &c.nfe_refs {
                if !nfe_valid.contains_key(ch.as_str()) {
                    nfes_nao_encontrados.insert(ch.clone());
                }
            }
        }
    }

    if check_missing {
        show_docs("CTe", &ctes_nao_encontrados.to_vec_sorted());
        show_docs("NFe", &nfes_nao_encontrados.to_vec_sorted());
    }
    drop(ctes_nao_encontrados);
    drop(nfes_nao_encontrados);

    // DSU compacto O(V) sem explosão quadrática O(k²)
    let comps = CteComponents::from_adj(cte_ctes);

    let mut comp_nfes: Vec<HashSet<String>> = vec![HashSet::new(); comps.components.len()];
    for c in cte_summaries {
        if !c.valid {
            continue;
        }
        if let Some(&comp_id) = comps.cte_to_comp.get(&c.cte) {
            for ch in &c.nfe_refs {
                if nfe_valid.get(ch.as_str()).copied().unwrap_or(false) {
                    comp_nfes[comp_id].insert(ch.clone());
                }
            }
        }
    }

    let comp_cached: Vec<CachedCompNfeSummary> = comp_nfes
        .par_iter()
        .map(|nfes| {
            if nfes.is_empty() {
                None
            } else if need_item_aggregations {
                let (ncm_desc, total_val) = summarize_nfes_for_cte(nfes, &nfe_by_key, arguments);
                Some((nfes.to_vec_sorted(), ncm_desc, total_val))
            } else {
                Some((nfes.to_vec_sorted(), Vec::new(), None))
            }
        })
        .collect();

    let cte_writers = ShardedZstdWriters::new(spill, "enr_cte")?;
    let mut cte_bufs: Vec<Vec<u8>> = (0..NUM_SHARDS).map(|_| Vec::new()).collect();
    let mut cte_nfes_count = 0usize;

    if arguments.exibir_correlacoes {
        println!("cte_nfes:");
    }

    for (comp_id, members) in comps.components.iter().enumerate() {
        let (nfes_vec, ncm_desc, total_val) =
            comp_cached[comp_id].as_ref().cloned().unwrap_or_default();
        for cte in members {
            if !cte_valid.get(cte.as_str()).copied().unwrap_or(false) {
                continue;
            }
            if !nfes_vec.is_empty() {
                cte_nfes_count += 1;
                if arguments.exibir_correlacoes {
                    println!(
                        "{cte}: {:>2} {:?} ; Valor Total = {} ; NCMs: {:?}",
                        nfes_vec.len(),
                        nfes_vec,
                        total_val.unwrap_or_default(),
                        ncm_desc
                    );
                }
            }
            let enrich = CteEnrich {
                cte: cte.clone(),
                ctes_relacionados: members.clone(),
                nfes: nfes_vec.clone(),
                ncm_descricao: ncm_desc.clone(),
                valor_total_nfes: total_val,
            };
            let s = shard_index(&Some(cte.as_str()));
            flush_enrich_shard(&enrich, s, &mut cte_bufs, &cte_writers)?;
        }
    }

    let mut solo_nfes_set: HashSet<String> = HashSet::new();
    for c in cte_summaries {
        if !c.valid || comps.cte_to_comp.contains_key(&c.cte) {
            continue;
        }
        solo_nfes_set.clear();
        for ch in &c.nfe_refs {
            if nfe_valid.get(ch.as_str()).copied().unwrap_or(false) {
                solo_nfes_set.insert(ch.clone());
            }
        }
        if solo_nfes_set.is_empty() {
            continue;
        }
        let sorted = solo_nfes_set.to_vec_sorted();
        let (ncm_desc, total_val) = if need_item_aggregations {
            summarize_nfes_for_cte(&solo_nfes_set, &nfe_by_key, arguments)
        } else {
            (Vec::new(), None)
        };
        cte_nfes_count += 1;
        if arguments.exibir_correlacoes {
            println!(
                "{}: {:>2} {:?} ; Valor Total = {} ; NCMs: {:?}",
                c.cte,
                sorted.len(),
                sorted,
                total_val.unwrap_or_default(),
                ncm_desc
            );
        }
        let enrich = CteEnrich {
            cte: c.cte.clone(),
            ctes_relacionados: Vec::new(),
            nfes: sorted,
            ncm_descricao: ncm_desc,
            valor_total_nfes: total_val,
        };
        let s = shard_index(&Some(c.cte.as_str()));
        flush_enrich_shard(&enrich, s, &mut cte_bufs, &cte_writers)?;
    }

    finish_enrich_shards(cte_bufs, cte_writers)?;
    if arguments.exibir_correlacoes {
        println!(
            "cte_nfes.len(): {cte_nfes_count}
"
        );
    }

    drop(comp_cached);
    drop(nfe_by_key);

    if !need_nfe_enrich {
        return Ok(());
    }

    // Modo nativo: grafo reverso NF-e -> CTes e NF-e <-> NFes usando &str (zero clones de String)
    let mut nfe_ctes: HashMap<&str, Vec<&str>> = HashMap::new();
    for (comp_id, nfes) in comp_nfes.iter().enumerate() {
        if nfes.is_empty() {
            continue;
        }
        let members = &comps.components[comp_id];
        for nfe in nfes {
            let entry = nfe_ctes.entry(nfe.as_str()).or_default();
            entry.extend(members.iter().map(String::as_str));
        }
    }
    for c in cte_summaries {
        if !c.valid || comps.cte_to_comp.contains_key(&c.cte) {
            continue;
        }
        for ch in &c.nfe_refs {
            if nfe_valid.get(ch.as_str()).copied().unwrap_or(false) {
                nfe_ctes
                    .entry(ch.as_str())
                    .or_default()
                    .push(c.cte.as_str());
            }
        }
    }

    let mut nfe_nfes: HashMap<&str, Vec<&str>> = HashMap::new();
    for n in nfe_summaries {
        if !n.valid {
            continue;
        }
        for cte_ref in &n.cte_refs {
            if cte_ref != &n.nfe && cte_valid.get(cte_ref.as_str()).copied().unwrap_or(false) {
                nfe_ctes
                    .entry(n.nfe.as_str())
                    .or_default()
                    .push(cte_ref.as_str());
            }
        }
        for other_nfe in &n.nfe_refs {
            if other_nfe != &n.nfe && nfe_valid.get(other_nfe.as_str()).copied().unwrap_or(false) {
                nfe_nfes
                    .entry(n.nfe.as_str())
                    .or_default()
                    .push(other_nfe.as_str());
                nfe_nfes
                    .entry(other_nfe.as_str())
                    .or_default()
                    .push(n.nfe.as_str());
            }
        }
    }

    if arguments.exibir_correlacoes {
        println!("nfe_ctes:");
    }

    let all_nfe_keys: BTreeSet<&str> = nfe_ctes
        .keys()
        .copied()
        .chain(nfe_nfes.keys().copied())
        .collect();
    let nfe_ctes_count = nfe_ctes.len();

    let nfe_writers = ShardedZstdWriters::new(spill, "enr_nfe")?;
    let mut nfe_bufs: Vec<Vec<u8>> = (0..NUM_SHARDS).map(|_| Vec::new()).collect();
    let mut ctes_set: HashSet<String> = HashSet::new();

    for nfe in all_nfe_keys {
        if !nfe_valid.get(nfe).copied().unwrap_or(false) {
            continue;
        }
        let rel_nfes: Vec<String> = match nfe_nfes.get(nfe) {
            Some(v) => {
                let mut sorted: Vec<String> = v.iter().map(|s| (*s).to_string()).collect();
                sorted.sort();
                sorted.dedup();
                sorted
            }
            None => Vec::new(),
        };

        let (ctes_vec, tomadores, total_val) = match nfe_ctes.get(nfe) {
            Some(v) => {
                ctes_set.clear();
                ctes_set.extend(v.iter().map(|s| (*s).to_string()));
                let (tomadores, total_val) = if need_item_aggregations {
                    summarize_ctes_for_nfe(&ctes_set, &cte_by_key, arguments)
                } else {
                    (Vec::new(), None)
                };
                let sorted = ctes_set.to_vec_sorted();
                if arguments.exibir_correlacoes {
                    println!(
                        "{nfe}: {:>2} {:?} ; Valor Total = {} ; Tomadores: {:?}",
                        sorted.len(),
                        sorted,
                        total_val.unwrap_or_default(),
                        tomadores
                    );
                }
                (sorted, tomadores, total_val)
            }
            None => (Vec::new(), Vec::new(), None),
        };

        let enrich = NfeEnrich {
            nfe: nfe.to_string(),
            ctes: ctes_vec,
            nfes_relacionadas: rel_nfes,
            tomadores,
            valor_total_ctes: total_val,
        };
        let s = shard_index(&Some(nfe));
        flush_enrich_shard(&enrich, s, &mut nfe_bufs, &nfe_writers)?;
    }

    finish_enrich_shards(nfe_bufs, nfe_writers)?;
    if arguments.exibir_correlacoes {
        println!("nfe_ctes.len(): {nfe_ctes_count}\n");
    }

    Ok(())
}

fn summarize_nfes_for_cte(
    nfes: &HashSet<String>,
    nfe_by_key: &HashMap<&str, &NfeDocSummary>,
    arguments: &Arguments,
) -> (Vec<String>, Option<f64>) {
    let iter = nfes
        .iter()
        .filter_map(|nfe| nfe_by_key.get(nfe.as_str()))
        .filter(|doc| doc.valid)
        .flat_map(|doc| {
            doc.items
                .iter()
                .map(|(ncm, desc, v_prod)| (ncm.as_deref(), desc.as_deref(), *v_prod))
        });
    consolidate_ncm_items(iter, arguments.itens)
}

fn summarize_ctes_for_nfe(
    ctes: &HashSet<String>,
    cte_by_key: &HashMap<&str, &CteGraphSummary>,
    arguments: &Arguments,
) -> (Vec<String>, Option<f64>) {
    let iter = ctes
        .iter()
        .filter_map(|cte| cte_by_key.get(cte.as_str()))
        .filter(|info| info.valid)
        .map(|info| {
            (
                info.tomador_base.clone(),
                info.tomador_codigo,
                info.valor_total,
            )
        });
    consolidate_payer_items(iter, arguments.itens)
}

fn write_keys_kway_ctes(
    spill: &SpillDir,
    prefix: &str,
    chunk_size: usize,
) -> XmlParserResult<Vec<(PathBuf, usize)>> {
    let mut sorted_paths = Vec::with_capacity(NUM_SHARDS);
    for p in 0..NUM_SHARDS {
        let raw: Vec<SpillCte> = read_vec_zst(&spill.file(&format!("res_cte_{p:03}.zst")))?;
        if raw.is_empty() {
            continue;
        }
        let keys: Vec<String> = raw
            .into_iter()
            .filter_map(|c| c.cte)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let path = spill.file(&format!("keys_{prefix}_{p:03}.zst"));
        write_vec_zst(&path, &keys)?;
        sorted_paths.push(path);
    }
    write_kway_txt(&sorted_paths, prefix, chunk_size)
}

fn write_keys_kway_nfes(
    spill: &SpillDir,
    prefix: &str,
    chunk_size: usize,
) -> XmlParserResult<Vec<(PathBuf, usize)>> {
    let mut sorted_paths = Vec::with_capacity(NUM_SHARDS);
    for p in 0..NUM_SHARDS {
        let raw: Vec<SpillNfe> = read_vec_zst(&spill.file(&format!("res_nfe_{p:03}.zst")))?;
        if raw.is_empty() {
            continue;
        }
        let keys: Vec<String> = raw
            .into_iter()
            .filter_map(|n| n.nfe)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let path = spill.file(&format!("keys_{prefix}_{p:03}.zst"));
        write_vec_zst(&path, &keys)?;
        sorted_paths.push(path);
    }
    write_kway_txt(&sorted_paths, prefix, chunk_size)
}

fn write_kway_txt(
    paths: &[PathBuf],
    prefix: &str,
    chunk_size: usize,
) -> XmlParserResult<Vec<(PathBuf, usize)>> {
    let iter = KWayMergeIter::<String>::open(paths)?;
    let mut generated = Vec::new();
    let mut file_idx = 1usize;
    let mut count = 0usize;
    let mut current_path: Option<PathBuf> = None;
    let mut current_file: Option<File> = None;

    for key_res in iter {
        let key = key_res?;
        if current_file.is_none() || count >= chunk_size {
            if let Some(p) = current_path.take() {
                generated.push((p, count));
            }
            let p = PathBuf::from(format!("{prefix}-{file_idx:05}.txt"));
            current_file = Some(File::create(&p)?);
            current_path = Some(p);
            file_idx += 1;
            count = 0;
        }
        if let Some(f) = current_file.as_mut() {
            writeln!(f, "{key}")?;
            count += 1;
        }
    }

    if let Some(p) = current_path {
        generated.push((p, count));
    }
    for p in paths {
        let _ = std::fs::remove_file(p);
    }
    Ok(generated)
}

#[allow(clippy::too_many_arguments)]
fn export_ctes_ooc(
    spill: &SpillDir,
    arguments: &Arguments,
    number_format: NumberFormat,
    contrib_canon: &HashMap<String, String>,
    part_canon: &HashMap<String, String>,
    base_path: &Path,
    dataset_label: &str,
    pb: &mut MultiProgressBar,
    generated: &mut Vec<GeneratedFileSummary>,
) -> XmlParserResult<()> {
    let mut sorted_paths = Vec::with_capacity(NUM_SHARDS);

    for p in 0..NUM_SHARDS {
        let res_path = spill.file(&format!("res_cte_{p:03}.zst"));
        let raw: Vec<SpillCte> = read_vec_zst(&res_path)?;
        if raw.is_empty() {
            continue;
        }
        let mut ctes: Vec<InfoCte> = raw.into_iter().map(InfoCte::from).collect();
        let enrich: Vec<CteEnrich> = read_vec_zst(&spill.file(&format!("enr_cte_{p:03}.zst")))?;
        if !enrich.is_empty() {
            let map: HashMap<String, CteEnrich> =
                enrich.into_iter().map(|e| (e.cte.clone(), e)).collect();
            for c in &mut ctes {
                if c.is_valid()
                    && let Some(k) = &c.cte
                    && let Some(e) = map.get(k)
                {
                    if !e.ctes_relacionados.is_empty() {
                        let anteriores: HashSet<&str> =
                            c.cte_anteriores.iter().map(String::as_str).collect();
                        let mut merged: BTreeSet<String> =
                            c.cte_complementar.iter().cloned().collect();
                        for rel in &e.ctes_relacionados {
                            if rel != k && !anteriores.contains(rel.as_str()) {
                                merged.insert(rel.clone());
                            }
                        }
                        c.cte_complementar = merged.into_iter().collect();
                    }
                    c.nfes = e.nfes.clone();
                    c.ncm_descricao = e.ncm_descricao.clone();
                    c.valor_total_nfes = e.valor_total_nfes;
                }
            }
        }
        sanitize_slice(&mut ctes, arguments.clean_spaces);

        let out_path = spill.file(&format!("sorted_cte_{p:03}.zst"));
        if arguments.receita {
            let cte_options = ReceitaOptions::new(number_format, arguments.diversos);
            let mut rows: Vec<InfoReceita> = ctes
                .par_iter()
                .map(|c| {
                    let mut r = convert_cte_to_receita(c, cte_options);
                    if r.nome_contribuinte == RECEITA_ND
                        && let Some(cn) = contrib_canon.get(&r.cnpj_contribuinte)
                    {
                        r.nome_contribuinte = cn.clone();
                    }
                    if r.nome_participante == RECEITA_ND
                        && let Some(pn) = part_canon.get(&r.cpf_cnpj_participante)
                    {
                        r.nome_participante = pn.clone();
                    }
                    if arguments.clean_spaces {
                        r.clean_spaces();
                    }
                    r
                })
                .collect();
            sort_receita_rows(&mut rows);
            write_vec_zst(&out_path, &rows)?;
        } else {
            ctes.par_sort_by(cmp_info_cte);
            let spill_sorted: Vec<SpillCte> = ctes.into_iter().map(SpillCte::from).collect();
            write_vec_zst(&out_path, &spill_sorted)?;
        }
        sorted_paths.push(out_path);
        let _ = std::fs::remove_file(res_path);
        let _ = std::fs::remove_file(spill.file(&format!("enr_cte_{p:03}.zst")));
    }

    if arguments.receita {
        stream_merged_outputs::<InfoReceita, _>(
            &sorted_paths,
            arguments,
            "CTes",
            dataset_label,
            base_path,
            pb,
            generated,
            Ok,
        )?;
    } else {
        stream_merged_outputs::<SpillCte, InfoCte>(
            &sorted_paths,
            arguments,
            "CTes",
            dataset_label,
            base_path,
            pb,
            generated,
            |s| Ok(InfoCte::from(s)),
        )?;
    }

    for p in sorted_paths {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn export_nfes_ooc(
    spill: &SpillDir,
    arguments: &Arguments,
    number_format: NumberFormat,
    base_path: &Path,
    dataset_label: &str,
    pb: &mut MultiProgressBar,
    generated: &mut Vec<GeneratedFileSummary>,
) -> XmlParserResult<()> {
    let mut sorted_paths = Vec::with_capacity(NUM_SHARDS);

    for p in 0..NUM_SHARDS {
        let res_path = spill.file(&format!("res_nfe_{p:03}.zst"));
        let raw: Vec<SpillNfe> = read_vec_zst(&res_path)?;
        if raw.is_empty() {
            continue;
        }
        let mut nfes: Vec<InfoNfe> = raw.into_iter().map(InfoNfe::from).collect();
        let enrich: Vec<NfeEnrich> = read_vec_zst(&spill.file(&format!("enr_nfe_{p:03}.zst")))?;
        if !enrich.is_empty() {
            let map: HashMap<String, NfeEnrich> =
                enrich.into_iter().map(|e| (e.nfe.clone(), e)).collect();
            for n in &mut nfes {
                if n.is_valid()
                    && let Some(k) = &n.nfe
                    && let Some(e) = map.get(k)
                {
                    if !e.ctes.is_empty() {
                        let mut merged: BTreeSet<String> = n.ctes.iter().cloned().collect();
                        merged.extend(e.ctes.iter().cloned());
                        n.ctes = merged.into_iter().collect();
                        n.tomadores = e.tomadores.clone();
                        n.valor_total_ctes = e.valor_total_ctes;
                    }
                    if !e.nfes_relacionadas.is_empty() {
                        let mut merged: BTreeSet<String> =
                            n.nfes_relacionadas.iter().cloned().collect();
                        merged.extend(e.nfes_relacionadas.iter().cloned());
                        n.nfes_relacionadas = merged.into_iter().collect();
                    }
                }
            }
        }
        sanitize_slice(&mut nfes, arguments.clean_spaces);
        nfes.par_sort_by(cmp_info_nfe);

        let out_path = spill.file(&format!("sorted_nfe_{p:03}.zst"));
        if arguments.receita {
            let chunks: Vec<&[InfoNfe]> = nfes
                .chunk_by(|a, b| a.nfe.is_some() && a.nfe == b.nfe)
                .collect();
            let receita_options = ReceitaOptions::new(number_format, arguments.diversos);
            let mut rows: Vec<InfoReceita> = chunks
                .par_iter()
                .flat_map_iter(|chunk| convert_nfe_chunk_to_receita(chunk, receita_options))
                .collect();
            if arguments.clean_spaces {
                rows.par_iter_mut().for_each(SanitizeText::clean_spaces);
            }
            sort_receita_rows(&mut rows);
            write_vec_zst(&out_path, &rows)?;
        } else {
            let spill_sorted: Vec<SpillNfe> = nfes.into_iter().map(SpillNfe::from).collect();
            write_vec_zst(&out_path, &spill_sorted)?;
        }
        sorted_paths.push(out_path);
        let _ = std::fs::remove_file(res_path);
        let _ = std::fs::remove_file(spill.file(&format!("enr_nfe_{p:03}.zst")));
    }

    if arguments.receita {
        stream_merged_outputs::<InfoReceita, _>(
            &sorted_paths,
            arguments,
            "NFes",
            dataset_label,
            base_path,
            pb,
            generated,
            Ok,
        )?;
    } else {
        stream_merged_outputs::<SpillNfe, InfoNfe>(
            &sorted_paths,
            arguments,
            "NFes",
            dataset_label,
            base_path,
            pb,
            generated,
            |s| Ok(InfoNfe::from(s)),
        )?;
    }

    for p in sorted_paths {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

fn export_efin_ooc(
    spill: &SpillDir,
    arguments: &Arguments,
    base_path: &Path,
    pb: &mut MultiProgressBar,
    generated: &mut Vec<GeneratedFileSummary>,
) -> XmlParserResult<()> {
    let mut sorted_paths = Vec::with_capacity(NUM_SHARDS);

    for p in 0..NUM_SHARDS {
        let res_path = spill.file(&format!("res_efin_{p:03}.zst"));
        let raw: Vec<SpillEFinanceira> = read_vec_zst(&res_path)?;
        if raw.is_empty() {
            continue;
        }
        let mut efs: Vec<InfoEFinanceira> = raw.into_iter().map(InfoEFinanceira::from).collect();
        sanitize_slice(&mut efs, arguments.clean_spaces);
        efs.par_sort_by(cmp_info_efin);
        let spill_sorted: Vec<SpillEFinanceira> =
            efs.into_iter().map(SpillEFinanceira::from).collect();
        let out_path = spill.file(&format!("sorted_efin_{p:03}.zst"));
        write_vec_zst(&out_path, &spill_sorted)?;
        sorted_paths.push(out_path);
        let _ = std::fs::remove_file(res_path);
    }

    stream_merged_outputs::<SpillEFinanceira, InfoEFinanceira>(
        &sorted_paths,
        arguments,
        "eFinanceiras",
        "eFinanceiras",
        base_path,
        pb,
        generated,
        |s| Ok(InfoEFinanceira::from(s)),
    )?;

    for p in sorted_paths {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

impl MergeOrder for SpillCte {
    fn cmp_order(&self, other: &Self) -> std::cmp::Ordering {
        InfoCte::from_ref_cmp_fields(self, other)
    }
}

impl MergeOrder for SpillNfe {
    fn cmp_order(&self, other: &Self) -> std::cmp::Ordering {
        self.emitente_cnpj
            .cmp(&other.emitente_cnpj)
            .then_with(|| self.emitente_cpf.cmp(&other.emitente_cpf))
            .then_with(|| self.data_emissao.cmp(&other.data_emissao))
            .then_with(|| self.nfe.cmp(&other.nfe))
            .then_with(|| self.n_item.cmp(&other.n_item))
    }
}

impl MergeOrder for SpillEFinanceira {
    fn cmp_order(&self, other: &Self) -> std::cmp::Ordering {
        self.cnpj_do_declarante
            .cmp(&other.cnpj_do_declarante)
            .then_with(|| self.ni_do_declarado.cmp(&other.ni_do_declarado))
            .then_with(|| self.ano_mes_caixa.cmp(&other.ano_mes_caixa))
            .then_with(|| self.num_conta.cmp(&other.num_conta))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl InfoCte {
    fn from_ref_cmp_fields(a: &SpillCte, b: &SpillCte) -> std::cmp::Ordering {
        a.emitente_cnpj
            .cmp(&b.emitente_cnpj)
            .then_with(|| a.emitente_cpf.cmp(&b.emitente_cpf))
            .then_with(|| a.remetente_cnpj.cmp(&b.remetente_cnpj))
            .then_with(|| a.remetente_cpf.cmp(&b.remetente_cpf))
            .then_with(|| a.destinatario_cnpj.cmp(&b.destinatario_cnpj))
            .then_with(|| a.destinatario_cpf.cmp(&b.destinatario_cpf))
            .then_with(|| a.expedidor_cnpj.cmp(&b.expedidor_cnpj))
            .then_with(|| a.expedidor_cpf.cmp(&b.expedidor_cpf))
            .then_with(|| a.recebedor_cnpj.cmp(&b.recebedor_cnpj))
            .then_with(|| a.recebedor_cpf.cmp(&b.recebedor_cpf))
            .then_with(|| a.data_emissao.cmp(&b.data_emissao))
            .then_with(|| a.cte.cmp(&b.cte))
    }
}

#[allow(clippy::too_many_arguments)]
fn stream_merged_outputs<'de, S, T>(
    sorted_paths: &[PathBuf],
    arguments: &Arguments,
    sheet_name: &str,
    dataset: &str,
    base_path: &Path,
    pb: &mut MultiProgressBar,
    generated: &mut Vec<GeneratedFileSummary>,
    map_fn: impl Fn(S) -> XmlParserResult<T>,
) -> XmlParserResult<()>
where
    S: serde::de::DeserializeOwned + MergeOrder,
    T: serde::Serialize + serde::Deserialize<'de> + InfoExtension + struct_iterable::Iterable,
{
    if sorted_paths.is_empty() {
        return Ok(());
    }

    if arguments.csv {
        let csv_path = base_path.with_extension("csv");
        let csv_writer = CsvWriter::new(csv_path.clone(), arguments.delimiter);
        let mut writer = csv_writer.open_writer()?;
        let mut rows = 0usize;

        for item_res in KWayMergeIter::<S>::open(sorted_paths)? {
            let item = map_fn(item_res?)?;
            writer.serialize(&item)?;
            rows += 1;
        }
        writer.flush()?;
        pb.show_csval.inc(1);

        if rows > 0 {
            generated.push(GeneratedFileSummary::new(
                "CSV",
                dataset,
                &csv_path,
                rows,
                Some(T::get_headers().len()),
            ));
        }
    }

    if !arguments.avoid {
        let xlsx_path = base_path.with_extension("xlsx");
        let iter = KWayMergeIter::<S>::open(sorted_paths)?.map(|res| res.and_then(&map_fn));
        let (rows, logs) = write_xlsx_from_iter::<T, _, _>(
            iter,
            sheet_name,
            &xlsx_path,
            ExcelMemoryMode::Constant,
        )?;
        pb.show_excel.inc(1);
        if arguments.verbose {
            for line in logs {
                eprintln!("{line}");
            }
        }
        if rows > 0 {
            generated.push(GeneratedFileSummary::new(
                "XLSX",
                dataset,
                &xlsx_path,
                rows,
                Some(T::get_headers().len()),
            ));
        }
    }

    Ok(())
}
