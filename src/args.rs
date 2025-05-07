use clap::{ArgAction, CommandFactory, Parser};
use clap_complete::{Generator, Shell, generate};
use claudiofsr_lib::clear_terminal_screen;
use std::{io, path::PathBuf, process};

use crate::{MyResult, nodes::print_nodes, parse_xml_and_print_struct};

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
fn get_styles() -> clap::builder::Styles {
    let cyan = anstyle::Color::Ansi(anstyle::AnsiColor::Cyan);
    let green = anstyle::Color::Ansi(anstyle::AnsiColor::Green);
    let yellow = anstyle::Color::Ansi(anstyle::AnsiColor::Yellow);

    clap::builder::Styles::styled()
        .placeholder(anstyle::Style::new().fg_color(Some(yellow)))
        .usage(anstyle::Style::new().fg_color(Some(cyan)).bold())
        .header(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
                .underline(),
        )
        .literal(anstyle::Style::new().fg_color(Some(green)))
}

// https://docs.rs/clap/latest/clap/struct.Command.html#method.help_template
const APPLET_TEMPLATE: &str = "\
{before-help}
{about-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}";

#[derive(Parser, Debug)]
#[command(
    // Read from `Cargo.toml`
    author, version, about,
    long_about = None,
    next_line_help = true,
    help_template = APPLET_TEMPLATE,
    styles=get_styles(),
)]
pub struct Arguments {
    /// Avoid creating XLSX files after parsing XML files
    ///
    /// Para evitar a criação de arquivos XLSX, adicione a opção: --avoid ou -a
    #[arg(short, long, default_value_t = false, action=ArgAction::SetTrue)]
    pub avoid: bool,

    /// Write CSV (Comma Separated Values) files
    ///
    /// Para imprimir o arquivo CSV, adicione a opção: --csv ou -c
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub csv: bool,

    /// Set the field delimiter to use when writing CSV
    ///
    /// The default is b';'
    #[arg(
        short('d'),
        long,
        env("DELIMITER_CSV"),
        requires = "csv",
        required = false,
        default_value_t = ';'
    )]
    pub delimiter: char,

    /// Exibir chaves de documentos fiscais (CTe e NFe) não encontradas
    ///
    /// Exemplo:
    ///
    /// read_xml -tae > output_docs_nao_encotrados
    #[arg(short, long, default_value_t = false)]
    pub exibir_chaves_nao_encontradas: bool,

    /// Exibir chaves de documentos fiscais (CTe e NFe) correlacionadas
    ///
    /// Exemplos:
    ///
    /// read_xml -ta --exibir-correlacoes > output_docs_correlacionados
    ///
    /// read_xml -taei 100 --exibir-correlacoes > output_docs_nao_encontrados_e_docs_correlacionados
    #[arg(long, default_value_t = false)]
    pub exibir_correlacoes: bool,

    /**
    If provided, outputs the completion file for given shell.

    ### How to generate shell completions for Z-shell:

    #### Example (as root):

    Generate completions to read_xml.

    Visible to all system users.

    ```console

        mkdir -p /usr/local/share/zsh/site-functions

        read_xml --generate=zsh > /usr/local/share/zsh/site-functions/_read_xml

        compinit && zsh

    ```

    See `rustup completions` for detailed help.

    <https://github.com/clap-rs/clap/blob/master/clap_complete/examples/completion-derive.rs>
    */
    #[arg(short('g'), long("generate"), value_enum)]
    pub generator: Option<Shell>,

    /// Set maximum number of items with NCM information to be collected
    ///
    /// Apresentar no máximo N itens com informaçoes de NCM e descrição
    ///
    /// Os itens são coletados em ordem decrescente de valores
    ///
    /// Ver função: `fn get_info_de_nfes()` em lib.rs
    #[arg(short('i'), long("itens"), required = false, default_value_t = 100)]
    pub itens: usize,

    /// Set the maximum number of lines in the files with the found keys.
    ///
    /// Definir o nº máximo de linhas nos arquivos contendo as chaves encontradas.
    ///
    /// A Chave é uma sequência alfanumérica de 44 dígitos que identifica de
    /// forma única cada documento fiscal eletrônico (CTe e NFe).
    ///
    /// Cada arquivo conterá no máximo N chaves.
    ///
    /// Nomes dos arquivos gerados: nfes-00001.txt e ctes-00001.txt
    ///
    /// Exemplo: suponha que forem encontradas 1807 chaves de NFe e N = 900.
    ///
    /// read_xml -tal 900
    ///
    /// Portanto, serão gerados 3 arquivos:
    ///
    /// nfes-00001.txt com 900 chaves nfe;
    ///
    /// nfes-00002.txt com 900 chaves nfe;
    ///
    /// nfes-00003.txt com 7 chaves nfe.
    #[arg(
        short,
        long,
        required = false,
        //default_value_t = 10_000,
        //default_missing_value = "10000",
        value_parser = clap::value_parser!(u64).range(900..)
        //value_parser = |value: &str| value.parse::<usize>()
    )]
    pub linhas: Option<u64>,

    /// Set maximum depth to recursively search XML files
    ///
    /// Avoid descending into directories when the depth is exceeded
    #[arg(short('m'), long("max_depth"), required = false)]
    pub max_depth: Option<usize>,

    /// Print nodes from XML files
    #[arg(short('n'), long("nodes"), required = false)]
    pub nodes: Option<PathBuf>,

    /// Set the XML file path, otherwise recursively search
    /// for XML files in the current directory
    #[arg(short('p'), long("path"), required = false)]
    pub path: Option<PathBuf>,

    /// Parse CTe or NFe XML file and print Rust struct
    ///
    /// Save this result as a file.rs and use it to (de)serialize an XML document with serde
    ///
    /// read_xml -s cte.xml > file.rs
    #[arg(short('s'), long("structure"), required = false)]
    pub structure: Option<PathBuf>,

    /// Show total execution time
    #[arg(short('t'), long("time"), default_value_t = false)]
    pub time: bool,

    /// Wipe (Clear) the terminal screen before listing the identical files.
    ///
    /// On Linux, to clear use the command:
    ///
    /// tput reset
    ///
    /// Unlike the clear command, the reset command does more than just clear the terminal screen.
    ///
    /// It also resets the terminal to its default settings.
    #[arg(short('w'), long("wipe_terminal"), default_value_t = false)]
    // action = ArgAction::SetTrue
    pub wipe_terminal: bool,

    /// Show intermediate runtime messages.
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> MyResult<Arguments> {
        let args: Arguments = Arguments::parse();

        if let Some(generator) = args.generator {
            args.print_completions(generator);
        }

        if args.wipe_terminal {
            clear_terminal_screen();
        }

        if let Some(xml_path) = &args.structure {
            parse_xml_and_print_struct(xml_path)?;
            process::exit(0); // success
        }

        if let Some(xml_path) = &args.nodes {
            print_nodes(xml_path)?;
            process::exit(0); // success
        }

        Ok(args)
    }

    /// Print shell completions to standard output
    fn print_completions<G>(&self, gnt: G)
    where
        G: Generator + std::fmt::Debug,
    {
        let mut cmd = Arguments::command();
        let cmd_name = cmd.get_name().to_string();
        let mut stdout = io::stdout();

        eprintln!("Generating completion file for {gnt:?}...");
        generate(gnt, &mut cmd, cmd_name, &mut stdout);
        process::exit(1);
    }
}
