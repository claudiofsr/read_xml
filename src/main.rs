use read_xml::{
    Information,
    MyResult,
    Arguments,
    DocsFiscais,
    get_progressbar,
    get_xml_entries,
    write_csv,
    write_xlsx,
    configure_the_environment,
    atualizar_nfes_cancelados,
    atualizar_ctes_cancelados,
    get_all_info,
};

use std::{time::Instant, thread};
use walkdir::DirEntry;

/**
    clear && cargo run -- -tcvm 1 > /tmp/xml
    cargo b -r && cargo install --path=.

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
    configure_the_environment();
    let arguments = Arguments::build()?;
    let xml_entries: Vec<DirEntry> = get_xml_entries(&arguments)?;
    let mut multi_progressbar = get_progressbar(xml_entries.len())?;

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
    rayon::join(
        || atualizar_nfes_cancelados(&mut docs_fiscais.nfes, &docs_fiscais.eventos_nfe),
        || atualizar_ctes_cancelados(&mut docs_fiscais.ctes, &docs_fiscais.eventos_cte),
    );

    docs_fiscais.sort();

    // By default, after parsing xml files, write the xlsx files.
    if !arguments.avoid {
        let results = thread::scope(|s| {
            let thread_write_nfes = s.spawn(|| -> MyResult<()> {
                write_xlsx(&docs_fiscais.ctes, "CTes", "read_xml-ctes.xlsx")
            });
            let thread_write_ctes = s.spawn(|| -> MyResult<()> {
                write_xlsx(&docs_fiscais.nfes, "NFes", "read_xml-nfes.xlsx")
            });
            let thread_write_efin = s.spawn(|| -> MyResult<()> {
                write_xlsx(&docs_fiscais.efinanceiras, "eFinanceiras", "read_xml-efinanceiras.xlsx")
            });

            // Wait for background thread to complete.
            // Call join() on each handle to make sure all the threads finish.
            // join() returns immediately when the associated thread completes.

            let threads: Vec<_> = [thread_write_nfes, thread_write_ctes, thread_write_efin]
                .into_iter()
                .map(|scoped_join_handle| scoped_join_handle.join())
                .collect();

            threads
        });

        results
            .into_iter()
            .for_each(|result| {
                if result.is_err() {
                    eprintln!("Error: {result:?}");
                    panic!("thread::scope failed to write xlsx files!")
                }
            });
    }

    // Optionally, after parsing xml files, write the csv files.
    if arguments.csv {
        rayon::scope(|s| {
            s.spawn(|_| {
                write_csv(&docs_fiscais.ctes, "read_xml-ctes.csv", arguments.delimiter).unwrap()
            });
            s.spawn(|_| {
                write_csv(&docs_fiscais.nfes, "read_xml-nfes.csv", arguments.delimiter).unwrap()
            });
            s.spawn(|_| {
                write_csv(&docs_fiscais.efinanceiras, "read_xml-eFinanceiras.csv", arguments.delimiter).unwrap()
            })
        });
    }

    if arguments.time {
        eprintln!("\nTotal Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}
