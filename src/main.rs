use execution_time::ExecutionTime;
use indicatif::MultiProgress;
use std::{path::PathBuf, process};
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

    // 8. Opcional: Gravação de arquivos particionados com as chaves encontradas (-l)
    if let Some(size) = arguments.linhas {
        let size_u = size.try_into()?;
        docs_fiscais.print_ctes("ctes", size_u)?;
        docs_fiscais.print_nfes("nfes", size_u)?;
    }

    let mut output = OutputFilename::default();

    // 9. Opcional: Exportação tabular em formato CSV (-c / --csv)
    if arguments.csv {
        output.set_extension("csv");
        multi_progressbar.add_print_csv(&multi_progress, docs_fiscais.total())?;

        /// Função interna auxiliar para gravação genérica de CSV com feedback na barra.
        fn exportar_tabela_csv<T: serde::Serialize>(
            caminho: &std::path::Path,
            dados: &[T],
            delimitador: char,
            barra_progresso: &indicatif::ProgressBar,
        ) {
            let writer = CsvWriter::new(caminho.to_path_buf(), delimitador);
            if writer.write(dados).is_ok() {
                barra_progresso.inc(1);
            }
        }

        let delimiter = arguments.delimiter;
        let pb = &multi_progressbar.show_csval;

        // Escrita concorrente dos três arquivos CSV
        rayon::scope(|s| {
            s.spawn(|_| exportar_tabela_csv(&output.ctes, &docs_fiscais.ctes, delimiter, pb));
            s.spawn(|_| exportar_tabela_csv(&output.nfes, &docs_fiscais.nfes, delimiter, pb));
            s.spawn(|_| {
                exportar_tabela_csv(&output.efin, &docs_fiscais.efinanceiras, delimiter, pb)
            });
        });

        pb.finish();
    }

    // 10. Exportação padrão em formato Excel XLSX (a menos que `-a` esteja ativo)
    if !arguments.avoid {
        output.set_extension("xlsx");
        multi_progressbar.add_print_xls(&multi_progress, docs_fiscais.total())?;

        let memory_mode = arguments.memory_mode;
        let mut ctes_res = Ok(Vec::new());
        let mut nfes_res = Ok(Vec::new());
        let mut efin_res = Ok(Vec::new());

        // Geração concorrente das planilhas na thread-pool do Rayon
        rayon::scope(|s| {
            s.spawn(|_| {
                ctes_res = write_xlsx(&docs_fiscais.ctes, "CTes", &output.ctes, memory_mode);
                if ctes_res.is_ok() {
                    multi_progressbar.show_excel.inc(1);
                }
            });
            s.spawn(|_| {
                nfes_res = write_xlsx(&docs_fiscais.nfes, "NFes", &output.nfes, memory_mode);
                if nfes_res.is_ok() {
                    multi_progressbar.show_excel.inc(1);
                }
            });
            s.spawn(|_| {
                efin_res = write_xlsx(
                    &docs_fiscais.efinanceiras,
                    "eFinanceiras",
                    &output.efin,
                    memory_mode,
                );
                if efin_res.is_ok() {
                    multi_progressbar.show_excel.inc(1);
                }
            });
        });

        multi_progressbar.show_excel.finish();

        // Propaga o primeiro erro eventual antes de imprimir mensagens de sucesso
        let ctes_logs = ctes_res?;
        let nfes_logs = nfes_res?;
        let efin_logs = efin_res?;

        // Descarrega no stderr os logs descritivos de criação dos arquivos
        for line in ctes_logs.into_iter().chain(nfes_logs).chain(efin_logs) {
            eprintln!("{line}");
        }
    }

    // 11. Exibição do tempo total de execução (-t / --time)
    if arguments.time {
        timer.print_elapsed_time();
    }

    Ok(())
}
