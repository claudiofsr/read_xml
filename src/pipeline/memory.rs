//! # Motor de Execução em Memória (`In-Memory`)
//!
//! Lê todas as entradas `.xml` (de `.zip`s e de arquivos soltos) em paralelo para `DocsFiscais`
//! e executa o pipeline com máxima velocidade em RAM.

use indicatif::MultiProgress;
use std::{cell::RefCell, path::PathBuf, sync::Mutex};

use crate::{
    Arguments, DocsFiscais, ExcelMemoryMode, FiscalSampler, MultiProgressBar, NumberFormat,
    OutputFilename, ReceitaOptions, SampleConfig, SanitizeSliceExt, XmlParserResult,
    adicionar_eventos_cte, adicionar_eventos_nfe, ctes_to_receita, for_each_xml,
    get_xml_serialized_from_bytes, nfes_to_receita, partition_sources_with_xml,
    pipeline::{
        ExecutionCounts, GeneratedFileSummary, exportar_csv, exportar_xlsx,
        print_execution_summary, sanitize_slice,
    },
    retain_events_for_sampled_docs,
};

pub fn run_in_memory(zip_paths: &[PathBuf], arguments: &Arguments) -> XmlParserResult<()> {
    let sample_cfg = SampleConfig::from_args(arguments)?;
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

    let multi_progress = MultiProgress::new();
    let mut multi_progressbar = MultiProgressBar::default();
    multi_progressbar.add_parse_xml(&multi_progress, total_entries as usize)?;

    let docs_mutex = Mutex::new(DocsFiscais::new());
    let xml_counter = std::sync::atomic::AtomicUsize::new(0);
    let ev_cte_total = std::sync::atomic::AtomicUsize::new(0);
    let ev_nfe_total = std::sync::atomic::AtomicUsize::new(0);
    let canc_cte_total = std::sync::atomic::AtomicUsize::new(0);
    let canc_nfe_total = std::sync::atomic::AtomicUsize::new(0);

    thread_local! {
        static LOCAL_SAMPLER: RefCell<Option<FiscalSampler>> = const { RefCell::new(None) };
        static LOCAL_DOCS_BUF: RefCell<Vec<crate::Information>> = const { RefCell::new(Vec::new()) };
    }
    const BATCH_FLUSH: usize = 2048;

    let flush_local_docs = |buf: &mut Vec<crate::Information>| {
        if !buf.is_empty()
            && let Ok(mut guard) = docs_mutex.lock()
        {
            for item in buf.drain(..) {
                guard.add_information(item);
            }
        }
    };

    for_each_xml(
        &active_zips,
        Some(&multi_progressbar.show_parse),
        |label, bytes| {
            let idx = xml_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let Some(info) = get_xml_serialized_from_bytes(label, bytes, arguments) else {
                return;
            };
            if arguments.verbose {
                println!("xml {idx} ({label}): {info:#?}\n");
            }

            let to_store = if let Some(cfg) = sample_cfg {
                let maybe_event = LOCAL_SAMPLER.with(|cell| {
                    let mut opt = cell.borrow_mut();
                    let sampler = opt.get_or_insert_with(|| FiscalSampler::new(cfg));
                    sampler.push_or_pass_event(info)
                });
                if let Some(ev) = &maybe_event {
                    match ev {
                        crate::Information::EventoCte(_) => {
                            ev_cte_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                        crate::Information::EventoNfe(_) => {
                            ev_nfe_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                        crate::Information::CancelamentoCte(_) => {
                            canc_cte_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                        crate::Information::CancelamentoNfe(_) => {
                            canc_nfe_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
                maybe_event
            } else {
                Some(info)
            };

            if let Some(item) = to_store {
                LOCAL_DOCS_BUF.with(|cell| {
                    let mut buf = cell.borrow_mut();
                    buf.push(item);
                    if buf.len() >= BATCH_FLUSH {
                        flush_local_docs(&mut buf);
                    }
                });
            }
        },
    )?;

    rayon::broadcast(|_| {
        LOCAL_DOCS_BUF.with(|cell| flush_local_docs(&mut cell.borrow_mut()));
    });
    LOCAL_DOCS_BUF.with(|cell| flush_local_docs(&mut cell.borrow_mut()));

    multi_progressbar.show_parse.finish();
    let xml_total = xml_counter.load(std::sync::atomic::Ordering::Relaxed);
    let mut docs_fiscais = docs_mutex
        .into_inner()
        .map_err(|_| crate::XmlParserError::Export("Mutex poisoned in run_in_memory".into()))?;

    if let Some(cfg) = sample_cfg {
        let global_sampler = Mutex::new(FiscalSampler::new(cfg));
        rayon::broadcast(|_| {
            LOCAL_SAMPLER.with(|cell| {
                if let Some(local) = cell.borrow_mut().take()
                    && let Ok(mut g) = global_sampler.lock()
                {
                    g.merge(local);
                }
            });
        });
        LOCAL_SAMPLER.with(|cell| {
            if let Some(local) = cell.borrow_mut().take()
                && let Ok(mut g) = global_sampler.lock()
            {
                g.merge(local);
            }
        });

        let sampled = global_sampler
            .into_inner()
            .map_err(|_| crate::XmlParserError::Export("Mutex poisoned on sampler merge".into()))?
            .into_docs();
        docs_fiscais.ctes = sampled.ctes;
        docs_fiscais.nfes = sampled.nfes;
        docs_fiscais.efinanceiras = sampled.efinanceiras;
        retain_events_for_sampled_docs(&mut docs_fiscais);
    }

    let (ev_cte, ev_nfe, canc_cte, canc_nfe) = if sample_cfg.is_some() {
        (
            ev_cte_total.load(std::sync::atomic::Ordering::Relaxed),
            ev_nfe_total.load(std::sync::atomic::Ordering::Relaxed),
            canc_cte_total.load(std::sync::atomic::Ordering::Relaxed),
            canc_nfe_total.load(std::sync::atomic::Ordering::Relaxed),
        )
    } else {
        (
            docs_fiscais.eventos_cte.len(),
            docs_fiscais.eventos_nfe.len(),
            docs_fiscais.cancel_cte.len(),
            docs_fiscais.cancel_nfe.len(),
        )
    };

    process_and_export_docs(
        docs_fiscais,
        xml_total,
        (ev_cte, ev_nfe, canc_cte, canc_nfe),
        arguments,
        sample_cfg,
        None,
    )
}

/// Executa vinculação de eventos, deduplicação, ordenação, correlação e exportação tabular.
pub fn process_and_export_docs(
    mut docs_fiscais: DocsFiscais,
    xml_total: usize,
    event_counts: (usize, usize, usize, usize),
    arguments: &Arguments,
    sample_cfg: Option<SampleConfig>,
    memory_mode_override: Option<ExcelMemoryMode>,
) -> XmlParserResult<()> {
    let (ev_cte, ev_nfe, canc_cte, canc_nfe) = event_counts;

    rayon::scope(|s| {
        s.spawn(|_| {
            adicionar_eventos_nfe(
                &mut docs_fiscais.nfes,
                &docs_fiscais.eventos_nfe,
                &docs_fiscais.cancel_nfe,
            );
        });
        s.spawn(|_| {
            adicionar_eventos_cte(
                &mut docs_fiscais.ctes,
                &docs_fiscais.eventos_cte,
                &docs_fiscais.cancel_cte,
            );
        });
    });

    docs_fiscais.clear_events();

    docs_fiscais.unique();
    docs_fiscais.sort();
    docs_fiscais.get_correlations(arguments);

    let counts =
        ExecutionCounts::from_docs(xml_total, &docs_fiscais, ev_cte, ev_nfe, canc_cte, canc_nfe);

    let mut generated_files: Vec<GeneratedFileSummary> = Vec::new();

    if let Some(size) = arguments.linhas {
        let size_u = size.try_into()?;
        for (txt_path, count) in docs_fiscais.print_ctes("ctes", size_u)? {
            generated_files.push(GeneratedFileSummary::new(
                "TXT",
                "CTes (chaves)",
                &txt_path,
                count,
                None,
            ));
        }
        for (txt_path, count) in docs_fiscais.print_nfes("nfes", size_u)? {
            generated_files.push(GeneratedFileSummary::new(
                "TXT",
                "NFes (chaves)",
                &txt_path,
                count,
                None,
            ));
        }
    }

    sanitize_slice(&mut docs_fiscais.ctes, arguments.clean_spaces);
    sanitize_slice(&mut docs_fiscais.nfes, arguments.clean_spaces);
    sanitize_slice(&mut docs_fiscais.efinanceiras, arguments.clean_spaces);

    let multi_progress = MultiProgress::new();
    let mut multi_progressbar = MultiProgressBar::default();
    let total_datasets = docs_fiscais.total();
    let mut output = OutputFilename::default();

    let mut receita_data = if arguments.receita && (arguments.csv || !arguments.avoid) {
        let number_format = NumberFormat::from(arguments.format_br);
        let receita_options = ReceitaOptions::new(number_format, arguments.diversos);
        let data = rayon::join(
            || ctes_to_receita(&docs_fiscais.ctes, receita_options),
            || nfes_to_receita(&docs_fiscais.nfes, receita_options),
        );
        docs_fiscais.ctes = Vec::new();
        docs_fiscais.nfes = Vec::new();
        Some(data)
    } else {
        None
    };

    if arguments.clean_spaces
        && let Some((ctes_rec, nfes_rec)) = &mut receita_data
    {
        rayon::join(
            || ctes_rec.clean_spaces_par(),
            || nfes_rec.clean_spaces_par(),
        );
    }

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

    if arguments.csv {
        output.set_extension("csv");
        multi_progressbar.add_print_csv(&multi_progress, total_datasets)?;

        let delimiter = arguments.delimiter;
        let pb = &multi_progressbar.show_csval;
        let (mut ctes_res, mut nfes_res, mut efin_res) = (Ok(None), Ok(None), Ok(None));

        rayon::scope(|s| {
            s.spawn(|_| {
                ctes_res = match &receita_data {
                    Some((ctes_rec, _)) => {
                        exportar_csv(&output.ctes, ctes_label, ctes_rec, delimiter, pb)
                    }
                    None => {
                        exportar_csv(&output.ctes, ctes_label, &docs_fiscais.ctes, delimiter, pb)
                    }
                };
            });
            s.spawn(|_| {
                nfes_res = match &receita_data {
                    Some((_, nfes_rec)) => {
                        exportar_csv(&output.nfes, nfes_label, nfes_rec, delimiter, pb)
                    }
                    None => {
                        exportar_csv(&output.nfes, nfes_label, &docs_fiscais.nfes, delimiter, pb)
                    }
                };
            });
            s.spawn(|_| {
                efin_res = exportar_csv(
                    &output.efin,
                    "eFinanceiras",
                    &docs_fiscais.efinanceiras,
                    delimiter,
                    pb,
                );
            });
        });

        pb.finish();
        for summary in [ctes_res?, nfes_res?, efin_res?].into_iter().flatten() {
            generated_files.push(summary);
        }
    }

    if !arguments.avoid {
        output.set_extension("xlsx");
        multi_progressbar.add_print_xls(&multi_progress, total_datasets)?;

        let memory_mode = memory_mode_override.unwrap_or(arguments.memory_mode);
        let pb = &multi_progressbar.show_excel;
        let (mut ctes_res, mut nfes_res, mut efin_res) = (
            Ok((Vec::new(), None)),
            Ok((Vec::new(), None)),
            Ok((Vec::new(), None)),
        );

        rayon::scope(|s| {
            s.spawn(|_| {
                ctes_res = match &receita_data {
                    Some((ctes_rec, _)) => {
                        exportar_xlsx(ctes_rec, "CTes", ctes_label, &output.ctes, memory_mode, pb)
                    }
                    None => exportar_xlsx(
                        &docs_fiscais.ctes,
                        "CTes",
                        ctes_label,
                        &output.ctes,
                        memory_mode,
                        pb,
                    ),
                };
            });
            s.spawn(|_| {
                nfes_res = match &receita_data {
                    Some((_, nfes_rec)) => {
                        exportar_xlsx(nfes_rec, "NFes", nfes_label, &output.nfes, memory_mode, pb)
                    }
                    None => exportar_xlsx(
                        &docs_fiscais.nfes,
                        "NFes",
                        nfes_label,
                        &output.nfes,
                        memory_mode,
                        pb,
                    ),
                };
            });
            s.spawn(|_| {
                efin_res = exportar_xlsx(
                    &docs_fiscais.efinanceiras,
                    "eFinanceiras",
                    "eFinanceiras",
                    &output.efin,
                    memory_mode,
                    pb,
                );
            });
        });

        pb.finish();
        for (logs, summary) in [ctes_res?, nfes_res?, efin_res?] {
            if arguments.verbose {
                for line in logs {
                    eprintln!("{line}");
                }
            }
            if let Some(item) = summary {
                generated_files.push(item);
            }
        }
    }

    if let Some(cfg) = sample_cfg {
        eprintln!(
            "Amostragem ativa        : max={} (maiores={}, random={}, seed={})",
            cfg.total_limit, cfg.top_k, cfg.random_k, cfg.seed
        );
    }
    print_execution_summary(&counts, &generated_files);
    Ok(())
}
