//! # Pipeline Modular de Processamento de Fontes XML
//!
//! Fornece dois motores de execução sobre arquivos `.zip` (incluindo `.zip` aninhados)
//! e arquivos `.xml` soltos:
//! - [`memory`]: processamento integral em RAM com máxima velocidade.
//! - [`out_of_core`]: motor em disco (`-b` / `--buffer`) baseado em Hash Sharding (`postcard` + `zstd`)
//!   e *K-Way External Merge Sort* para suportar dezenas de milhões ou bilhões de XMLs com RAM limitada.

pub mod memory;
pub mod out_of_core;
pub mod spill_codec;
pub mod zip_source;

pub use memory::run_in_memory;
pub use out_of_core::run_out_of_core;
pub use zip_source::{
    count_source_entries, for_each_xml, get_xml_entries, partition_sources_with_xml,
};

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    CsvWriter, DocsFiscais, ExcelMemoryMode, InfoExtension, SanitizeSliceExt, SanitizeText,
    XmlParserResult, write_xlsx,
};

/// Metadados resumidos de cada arquivo gerado ao final da execução.
pub struct GeneratedFileSummary {
    pub format: &'static str,
    pub dataset: String,
    pub path: PathBuf,
    pub rows: usize,
    pub columns: Option<usize>,
    pub size_bytes: u64,
}

impl GeneratedFileSummary {
    pub fn new(
        format: &'static str,
        dataset: impl Into<String>,
        path: &Path,
        rows: usize,
        columns: Option<usize>,
    ) -> Self {
        let resolved_path = resolve_display_path(path);
        let size_bytes = std::fs::metadata(&resolved_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Self {
            format,
            dataset: dataset.into(),
            path: resolved_path,
            rows,
            columns,
            size_bytes,
        }
    }
}

/// Contadores consolidados para exibição do resumo final de execução.
#[derive(Debug, Default)]
pub struct ExecutionCounts {
    pub xml_total: usize,
    pub ctes_total: usize,
    pub ctes_cancelados: usize,
    pub nfes_itens: usize,
    pub nfes_unicas: usize,
    pub nfes_canceladas: usize,
    pub efin_total: usize,
    pub eventos_cte: usize,
    pub eventos_nfe: usize,
    pub cancel_cte: usize,
    pub cancel_nfe: usize,
}

impl ExecutionCounts {
    pub fn from_docs(
        xml_total: usize,
        docs: &DocsFiscais,
        eventos_cte: usize,
        eventos_nfe: usize,
        cancel_cte: usize,
        cancel_nfe: usize,
    ) -> Self {
        let ctes_cancelados = docs.ctes.iter().filter(|c| c.is_canceled()).count();
        let (nfes_unicas, nfes_canceladas) = {
            let mut unique_keys = HashSet::new();
            let mut canceled_keys = HashSet::new();
            for item in &docs.nfes {
                if let Some(chave) = item.nfe.as_deref() {
                    unique_keys.insert(chave);
                    if item.is_canceled() {
                        canceled_keys.insert(chave);
                    }
                }
            }
            (unique_keys.len(), canceled_keys.len())
        };

        Self {
            xml_total,
            ctes_total: docs.ctes.len(),
            ctes_cancelados,
            nfes_itens: docs.nfes.len(),
            nfes_unicas,
            nfes_canceladas,
            efin_total: docs.efinanceiras.len(),
            eventos_cte,
            eventos_nfe,
            cancel_cte,
            cancel_nfe,
        }
    }
}

pub fn sanitize_slice<T: SanitizeText + Send>(items: &mut [T], clean_spaces: bool) {
    if clean_spaces {
        items.clean_spaces_par();
    } else {
        items.strip_line_breaks_par();
    }
}

pub fn resolve_display_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}

pub fn format_file_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let b = bytes as f64;

    if b >= GIB {
        format!("{:.2} GiB ({bytes} bytes)", b / GIB)
    } else if b >= MIB {
        format!("{:.2} MiB ({bytes} bytes)", b / MIB)
    } else if b >= KIB {
        format!("{:.2} KiB ({bytes} bytes)", b / KIB)
    } else {
        format!("{bytes} bytes")
    }
}

pub fn print_execution_summary(counts: &ExecutionCounts, generated_files: &[GeneratedFileSummary]) {
    eprintln!("\n=== Resumo da Execução ===");
    eprintln!("Entradas XML analisadas : {}", counts.xml_total);
    eprintln!(
        "CT-es únicos            : {} (cancelados: {})",
        counts.ctes_total, counts.ctes_cancelados
    );
    eprintln!(
        "NF-es (itens / únicas)  : {} / {} (canceladas: {})",
        counts.nfes_itens, counts.nfes_unicas, counts.nfes_canceladas
    );
    eprintln!("e-Financeiras           : {}", counts.efin_total);
    eprintln!(
        "Eventos / Cancelamentos : CTe={}, NFe={}, CancCTe={}, CancNFe={}",
        counts.eventos_cte, counts.eventos_nfe, counts.cancel_cte, counts.cancel_nfe
    );

    if generated_files.is_empty() {
        eprintln!("\nNenhum arquivo de saída gerado.");
        return;
    }

    eprintln!("\nArquivos gerados ({}):", generated_files.len());
    for (idx, item) in generated_files.iter().enumerate() {
        let shape = match item.columns {
            Some(cols) => format!("{} linhas x {cols} colunas", item.rows),
            None => format!("{} chaves", item.rows),
        };
        let size = format_file_size(item.size_bytes);
        eprintln!("  {}. [{}] {}", idx + 1, item.format, item.dataset);
        eprintln!("     Path   : {}", item.path.display());
        eprintln!("     Resumo : {shape} | {size}");
    }
    eprintln!();
}

pub fn exportar_csv<'de, T>(
    caminho: &Path,
    dataset: &str,
    dados: &[T],
    delimitador: char,
    barra_progresso: &indicatif::ProgressBar,
) -> XmlParserResult<Option<GeneratedFileSummary>>
where
    T: serde::Serialize + serde::Deserialize<'de> + InfoExtension,
{
    if dados.is_empty() {
        return Ok(None);
    }
    let writer = CsvWriter::new(caminho.to_path_buf(), delimitador);
    writer.write(dados)?;
    barra_progresso.inc(1);
    Ok(Some(GeneratedFileSummary::new(
        "CSV",
        dataset,
        caminho,
        dados.len(),
        Some(T::get_headers().len()),
    )))
}

pub fn exportar_xlsx<'de, T>(
    dados: &[T],
    sheet_name: &str,
    dataset: &str,
    caminho: &Path,
    memory_mode: ExcelMemoryMode,
    barra_progresso: &indicatif::ProgressBar,
) -> XmlParserResult<(Vec<String>, Option<GeneratedFileSummary>)>
where
    T: serde::Serialize
        + serde::Deserialize<'de>
        + InfoExtension
        + struct_iterable::Iterable
        + Sync,
{
    if dados.is_empty() {
        return Ok((Vec::new(), None));
    }
    let logs = write_xlsx(dados, sheet_name, caminho, memory_mode)?;
    barra_progresso.inc(1);
    let summary = GeneratedFileSummary::new(
        "XLSX",
        dataset,
        caminho,
        dados.len(),
        Some(T::get_headers().len()),
    );
    Ok((logs, Some(summary)))
}
