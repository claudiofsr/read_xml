use indicatif::MultiProgress;
use read_xml::{
    adicionar_eventos_cte, 
    adicionar_eventos_nfe, 
    get_all_info, get_xml_entries, 
    CsvWriter, write_xlsx, 
    Arguments, DocsFiscais, 
    Information, MultiProgressBar, 
    MyResult, OuputFilename
};

use std::{thread, time::Instant};
use walkdir::DirEntry;

/**
    clear && cargo run -- -tcvm 1 > /tmp/xml
    cargo b -r && cargo install --path=.
    cargo run -- -h
    cargo doc --open
    cargo run -- -tacei 100 -p "/home/claudio/Documents/ ... /" > /tmp/output
    cargo test -- --show-output
    rustfmt src/xml_structs/agente.rs

    read_xml -tcei 100 -l 100000 --exibir-correlacoes > output_docs_nao_encontrados_e_docs_correlacionados

    To see the Rust Structures in an xml file:
    read_xml -s 35220412345678901234567890123456789012345678_NFe.xml > /tmp/structures.rs

    Analize xml files recursively:
    read_xml -tv > /tmp/xml

    # If some error:
    tail -n 10 /tmp/xml
    read_xml -s some_file.xml
*/

fn main() -> MyResult<()> {
    let time = Instant::now();
    let arguments = Arguments::build()?;
    let xml_entries: Vec<DirEntry> = get_xml_entries(&arguments)?;

    let multi_progress = MultiProgress::new();
    let mut multi_progressbar = MultiProgressBar::default();
    multi_progressbar.add_parse_xml(&multi_progress, xml_entries.len())?;
    multi_progressbar.add_print_xml(&multi_progress, xml_entries.len())?;

    let infos: Vec<Information> = get_all_info(
        &xml_entries,
        &mut multi_progressbar,
        &arguments
    );

    // Add informations to DocsFiscais
    let mut docs_fiscais = DocsFiscais::new();

    for (count, info) in (1u64..).zip(infos.iter()) {
        info.add_info_to_docs_fiscais(&mut docs_fiscais);
        if arguments.verbose {
            println!("xml {count}: {info:#?}\n");
        }
        multi_progressbar.show_print.inc(1);
    }

    multi_progressbar.show_print.finish();

    // Takes two closures and potentially runs them in parallel.
    thread::scope(|s| {
        s.spawn(|| adicionar_eventos_nfe(&mut docs_fiscais.nfes, &docs_fiscais.eventos_nfe, &docs_fiscais.cancel_nfe));
        s.spawn(|| adicionar_eventos_cte(&mut docs_fiscais.ctes, &docs_fiscais.eventos_cte, &docs_fiscais.cancel_cte));
    });

    docs_fiscais.unique();
    docs_fiscais.sort();
    docs_fiscais.get_correlations(&arguments);

    // Imprimir em arquivos com no máximo N linhas as chaves encontradas.
    if let Some(size) = arguments.linhas {
        docs_fiscais.print_ctes("ctes", size.try_into()?)?;
        docs_fiscais.print_nfes("nfes", size.try_into()?)?;
    }

    let mut output = OuputFilename::default();

    // By default, after parsing xml files, write the xlsx files.
    if !arguments.avoid {
        output.set_extension("xlsx");
        multi_progressbar.add_print_xls(&multi_progress, docs_fiscais.total())?;
        thread::scope(|s| {
            s.spawn(|| {
                if write_xlsx(&docs_fiscais.ctes, "CTes", &output.ctes).is_ok() {
                    multi_progressbar.show_excel.inc(1);
                }
            });
            s.spawn(|| {
                if write_xlsx(&docs_fiscais.nfes, "NFes", &output.nfes).is_ok() {
                    multi_progressbar.show_excel.inc(1);
                }
            });
            s.spawn(|| {
                if write_xlsx(&docs_fiscais.efinanceiras, "eFinanceiras",  &output.efin).is_ok() {
                    multi_progressbar.show_excel.inc(1);
                }
            });
        });
        multi_progressbar.show_excel.finish();
    }

    // Optionally, after parsing xml files, write the csv files.
    if arguments.csv {
        output.set_extension("csv");
        multi_progressbar.add_print_csv(&multi_progress, docs_fiscais.total())?;
        thread::scope(|s| {
            s.spawn(|| {
                let csv_writer = CsvWriter::new(output.ctes, arguments.delimiter);
                if csv_writer.write(&docs_fiscais.ctes).is_ok() {
                    multi_progressbar.show_csval.inc(1);
                }
            });
            s.spawn(|| {
                let csv_writer = CsvWriter::new(output.nfes, arguments.delimiter);
                if csv_writer.write(&docs_fiscais.nfes).is_ok() {
                    multi_progressbar.show_csval.inc(1);
                }
            });
            s.spawn(|| {
                let csv_writer = CsvWriter::new(output.efin, arguments.delimiter);
                if csv_writer.write(&docs_fiscais.efinanceiras).is_ok() {
                    multi_progressbar.show_csval.inc(1);
                }
            });
        });
        multi_progressbar.show_csval.finish();
    }

    if arguments.time {
        eprintln!("\nTotal Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}
