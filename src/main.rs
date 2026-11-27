use execution_time::ExecutionTime;
use std::process;

use read_xml::*;

/*
    clear && cargo run -- -tcvm 1 > /tmp/xml
    cargo b -r && cargo install --path=.
    cargo run -- -h
    cargo test -- --show-output

    cargo fmt && cargo clippy --all-targets && cargo test && cargo install --path=.

    # Ler .zip e .xml soltos do diretório atual (em RAM, máxima velocidade):
    read-xml -catr

    # Filtrar documentos pelas chaves de 44 caracteres de um arquivo texto:
    read-xml -catr -f chaves_extraidas.txt

    # Ler arquivos .zip em modo out-of-core (-b / --buffer) para milhões/bilhões de XMLs:
    read-xml -catrb

    # Validar tags não mapeadas diretamente dentro dos .zip:
    read-xml -k
*/

fn main() {
    // 8 MiB por thread worker do Rayon para suportar XMLs profundamente aninhados.
    let _ = rayon::ThreadPoolBuilder::new()
        .stack_size(8 * 1024 * 1024)
        .build_global();

    let arguments = match Arguments::build() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("\nErro ao processar argumentos da linha de comando:\n  {err}\n");
            process::exit(1);
        }
    };

    if let Err(error) = run(&arguments) {
        eprintln!("\nFalha durante a execução do programa:\n  {error}\n");
        process::exit(1);
    }
}

fn run(arguments: &Arguments) -> XmlParserResult<()> {
    let timer = ExecutionTime::start();

    init_key_filter(arguments)?;

    let xml_entries = get_xml_entries(arguments)?;
    if xml_entries.is_empty() {
        eprintln!("Aviso: nenhum arquivo .zip ou .xml encontrado no caminho informado.");
        return Ok(());
    }

    if arguments.verificar_consistencia {
        verificar_inconsistencias(&xml_entries)?;
        if arguments.time {
            eprintln!("Elapsed time: {}", timer.get_elapsed_time());
        }
        return Ok(());
    }

    if arguments.buffer {
        run_out_of_core(&xml_entries, arguments)?;
    } else {
        run_in_memory(&xml_entries, arguments)?;
    }

    print_key_filter_report(arguments);

    if arguments.time {
        eprintln!("Elapsed time: {}", timer.get_elapsed_time());
    }

    Ok(())
}

#[cfg(test)]
mod tests_main {
    use read_xml::format_file_size;

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
