use read_xml::{
    Information,
    MyResult,
    Arguments,
    InfoCte,
    InfoNfe,
    InfoCteEvento,
    InfoNfeEvento,
    InfoEFinanceira,
    get_progressbar,
    get_xml_entries,
    print_csv_file,
    write_xlsx,
    configure_the_environment,
    atualizar_nfes_cancelados,
    atualizar_ctes_cancelados,
    get_all_info,
};

use std::time::Instant;
use walkdir::DirEntry;

// type Map = BTreeMap<String, Information>;

/**
    clear && cargo run -- -tvm 1 > /tmp/xml
    cargo b -r && cargo install --path=.

    To see the Rust Structures in an xml file:
    read_xml -s 35220412345678901234567890123456789012345678_NFe.xml > /tmp/output.rs

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

    multi_progressbar.a.finish();

    let mut ctes: Vec<InfoCte> = Vec::new();
    let mut nfes: Vec<InfoNfe> = Vec::new();
    let mut eventos_cte: Vec<InfoCteEvento> = Vec::new();
    let mut eventos_nfe: Vec<InfoNfeEvento> = Vec::new();
    let mut efinanceiras: Vec<InfoEFinanceira> = Vec::new();

    for (count, info) in (1u64..).zip(infos.iter()) {

        match info {
            Information::Cte(info_cte) => ctes.push(*info_cte.clone()),
            Information::Nfe(info_nfe) => nfes.extend(info_nfe.clone()),
            Information::EventoCte(info_cte_evento) => eventos_cte.push(*info_cte_evento.clone()),
            Information::EventoNfe(info_nfe_evento) => eventos_nfe.push(*info_nfe_evento.clone()),
            Information::EFinanceira(info_efinanceira) => efinanceiras.extend(info_efinanceira.clone()),
            Information::None => continue,
        }

        if arguments.verbose {
            println!("xml {count}: {info:#?}\n");
        }

        multi_progressbar.b.inc(1);
    }

    multi_progressbar.b.finish();

    atualizar_nfes_cancelados(&mut nfes, &eventos_nfe);
    atualizar_ctes_cancelados(&mut ctes, &eventos_cte);

    if arguments.csv {
        print_csv_file(&ctes, "read_xml-ctes")?;
        print_csv_file(&nfes, "read_xml-nfes")?;
        print_csv_file(&efinanceiras, "read_xml-eFinanceiras")?;
    }

    write_xlsx(&ctes, "read_xml-ctes")?;
    write_xlsx(&nfes, "read_xml-nfes")?;
    write_xlsx(&efinanceiras, "read_xml-eFinanceiras")?;

    if arguments.time {
        eprintln!("\nTotal Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}
