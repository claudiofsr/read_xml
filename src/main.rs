use execution_time::ExecutionTime;
use indicatif::MultiProgress;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process,
};
use walkdir::DirEntry;

use read_xml::*;

/*
    clear && cargo run -- -tcvm 1 > /tmp/xml
    cargo b -r && cargo install --path=.
    cargo run -- -h
    cargo doc --open
    cargo run -- -tacei 100 -p "/home/claudio/Documents/ ... /" > /tmp/output
    cargo test -- --show-output
    rustfmt src/xml_structs/agente.rs

    read_xml -taei 200 -l 500000 > output_docs_nao_encontrados
    read_xml -tai 200 --exibir-correlacoes > output_correlacoes_new
    read_xml -m 1 -tcei 200 --exibir-correlacoes > output_docs_nao_encontrados_e_docs_correlacionados
    b3sum read_xml-*.csv && b3sum output_*

    To see the Rust Structures in an xml file:
    read_xml -s 35220412345678901234567890123456789012345678_NFe.xml > /tmp/structures.rs

    Analize xml files recursively:
    read_xml -tv > /tmp/xml

    # Encontrar tags presentes nos arquivos XML, porém ausentes em Structs.
    read_xml -k > /tmp/tags_ausentes.txt

    # If some error:
    tail -n 10 /tmp/xml
    read_xml -s some_file.xml
*/

/// Ponto de entrada do executável.
///
/// Atua como uma barreira de captura de falhas (*error boundary*).
/// Não contém lógica de negócio complexa: seu papel restringe-se a inicializar
/// os argumentos, delegar o processamento para a função [`run`] e garantir
/// o código de saída (*exit code*) adequado no encerramento do processo.
fn main() {
    // 1. Parse e validação dos argumentos de linha de comando
    let arguments = match Arguments::build() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("\nErro ao processar argumentos da linha de comando:\n  {err}\n");
            process::exit(1);
        }
    };

    // 2. Execução do pipeline de negócios delegada para `run`
    if let Err(error) = run(&arguments) {
        eprintln!("\nFalha durante a execução do programa:\n  {error}\n");
        process::exit(1);
    }

    // Ao alcançar o final do main com sucesso, o Rust encerra automaticamente com código 0.
}

/// Orquestrador do pipeline de processamento de documentos fiscais eletrônicos.
///
/// Retorna [`XmlParserResult`] para que o operador `?` possa propagar qualquer
/// falha de I/O, parsing de XML ou geração de planilhas de forma segura, garantindo
/// a execução de todos os destrutores ([`Drop`]) antes do retorno.
///
/// # Etapas Executadas:
///
/// 1. **Coleta de Arquivos**: Identifica os arquivos `.xml` elegíveis no diretório.
/// 2. **Validação Estrutural (-k)**: Se solicitada, executa apenas o check de tags não mapeadas.
/// 3. **Parsing Concorrente**: Desserializa NF-es, CT-es e e-Financeiras em paralelo.
/// 4. **Vinculação de Eventos**: Aplica cancelamentos e complementos aos documentos raiz.
/// 5. **Deduplicação e Ordenação**: Remove duplicidades e ordena os registros contábeis.
/// 6. **Cruzamento de Grafos**: Executa a correlação entre fretes (CT-e) e cargas (NF-e).
/// 7. **Exportação de Dados**: Gera arquivos `.txt` (chaves), `.csv` e `.xlsx`.
///
/// # Erros
///
/// Retorna [`XmlParserError`](read_xml::XmlParserError) caso ocorram erros de acesso
/// ao sistema de arquivos, inconsistências na escrita de planilhas ou falhas de formatação.
fn run(arguments: &Arguments) -> XmlParserResult<()> {
    let timer = ExecutionTime::start();

    // 1. Varredura recursiva de arquivos XML no diretório configurado
    let xml_entries: Vec<DirEntry> = get_xml_entries(arguments)?;

    // 2. Rota de Validação Rápida de Esquemas (-k / --verificar-consistencia)
    if arguments.verificar_consistencia {
        let paths: Vec<PathBuf> = xml_entries
            .iter()
            .map(|entry| entry.path().to_path_buf())
            .collect();

        verificar_inconsistencias_de_xml(&paths);

        if arguments.time {
            timer.print_elapsed_time();
        }
        return Ok(());
    }

    // 3. Inicialização e configuração visual das barras de progresso concorrentes
    let multi_progress = MultiProgress::new();
    let mut multi_progressbar = MultiProgressBar::default();
    multi_progressbar.add_parse_xml(&multi_progress, xml_entries.len())?;
    multi_progressbar.add_print_xml(&multi_progress, xml_entries.len())?;

    // 4. Parsing concorrente dos arquivos XML físicos
    let infos = get_all_info(&xml_entries, &mut multi_progressbar, arguments);

    // 5. Agregação sequencial das variantes no repositório de documentos
    let mut docs_fiscais = DocsFiscais::new();
    for (i, info) in infos.into_iter().enumerate() {
        if arguments.verbose {
            println!("xml {}: {info:#?}\n", i + 1);
        }
        docs_fiscais.add_information(info);
        multi_progressbar.show_print.inc(1);
    }
    multi_progressbar.show_print.finish();

    // 6. Vinculação concorrente de Eventos e Cancelamentos aos documentos raiz
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

    // 7. Deduplicação, ordenação contábil e resolução do grafo de correlações
    docs_fiscais.unique();
    docs_fiscais.sort();
    docs_fiscais.get_correlations(arguments);

    let mut generated_files: Vec<GeneratedFileSummary> = Vec::new();

    // 8. Opcional: Gravação de arquivos particionados com as chaves encontradas (-l)
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

    if arguments.clean_spaces {
        docs_fiscais.clean_spaces();
    } else {
        docs_fiscais.strip_line_breaks();
    }

    let mut output = OutputFilename::default();

    // Transformação opcional para o layout padrão de 56 colunas da Receita Federal (-r / --receita)
    let mut receita_data = if arguments.receita && (arguments.csv || !arguments.avoid) {
        Some(rayon::join(
            || ctes_to_receita(&docs_fiscais.ctes),
            || nfes_to_receita(&docs_fiscais.nfes),
        ))
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

    // 9. Opcional: Exportação tabular em formato CSV (-c / --csv)
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
        multi_progressbar.add_print_csv(&multi_progress, docs_fiscais.total())?;

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

    // 10. Exportação padrão em formato Excel XLSX (a menos que `-a` esteja ativo)
    if !arguments.avoid {
        output.set_extension("xlsx");
        multi_progressbar.add_print_xls(&multi_progress, docs_fiscais.total())?;

        let memory_mode = arguments.memory_mode;
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

    // 11. Resumo final dos documentos analisados e arquivos gerados
    print_execution_summary(xml_entries.len(), &docs_fiscais, &generated_files);

    // 12. Exibição do tempo total de execução (-t / --time)
    if arguments.time {
        timer.print_elapsed_time();
    }

    Ok(())
}

/// Metadados resumidos de cada arquivo gerado ao final da execução.
struct GeneratedFileSummary {
    format: &'static str,
    dataset: String,
    path: PathBuf,
    rows: usize,
    columns: Option<usize>,
    size_bytes: u64,
}

impl GeneratedFileSummary {
    fn new(
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

fn resolve_display_path(path: &Path) -> PathBuf {
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

fn format_file_size(bytes: u64) -> String {
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

fn print_execution_summary(
    xml_total: usize,
    docs: &DocsFiscais,
    generated_files: &[GeneratedFileSummary],
) {
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

    eprintln!("\n=== Resumo da Execução ===");
    eprintln!("Arquivos XML analisados : {xml_total}");
    eprintln!(
        "CT-es únicos            : {} (cancelados: {ctes_cancelados})",
        docs.ctes.len()
    );
    eprintln!(
        "NF-es (itens / únicas)  : {} / {nfes_unicas} (canceladas: {nfes_canceladas})",
        docs.nfes.len()
    );
    eprintln!("e-Financeiras           : {}", docs.efinanceiras.len());
    eprintln!(
        "Eventos / Cancelamentos : CTe={}, NFe={}, CancCTe={}, CancNFe={}",
        docs.eventos_cte.len(),
        docs.eventos_nfe.len(),
        docs.cancel_cte.len(),
        docs.cancel_nfe.len()
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

/// Exporta uma fatia genérica para CSV, atualizando a barra de progresso apenas se houver dados.
fn exportar_csv<'de, T>(
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

/// Exporta uma fatia genérica para XLSX, atualizando a barra de progresso apenas se houver dados.
fn exportar_xlsx<'de, T>(
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

#[cfg(test)]
mod tests_main {
    use super::format_file_size;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 bytes");
        assert_eq!(format_file_size(2048), "2.00 KiB (2048 bytes)");
        assert_eq!(
            format_file_size(5 * 1024 * 1024),
            "5.00 MiB (5242880 bytes)"
        );
    }
}
