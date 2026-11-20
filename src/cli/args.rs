use clap::{
    ArgAction, CommandFactory, Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_complete::{Generator, Shell, generate};
use claudiofsr_lib::clear_terminal_screen;
use std::{io, path::PathBuf, process};

use crate::{ExcelMemoryMode, XmlParserResult, parse_xml_and_print_struct, print_nodes};

/// Custom Clap styling to mimic a beautiful colored help menu.
fn get_styles() -> Styles {
    let cyan = AnsiColor::Cyan.on_default();
    let green = AnsiColor::Green.on_default();
    let yellow = AnsiColor::Yellow.on_default();

    Styles::styled()
        .header(yellow | Effects::BOLD)
        .usage(yellow | Effects::BOLD)
        .literal(green)
        .placeholder(cyan)
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
    #[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
    pub avoid: bool,

    /// Collapse two or more consecutive spaces into a single space
    #[arg(long("clean-spaces"), default_value_t = false)]
    pub clean_spaces: bool,

    /// Write CSV (Comma Separated Values) files
    #[arg(
        short,
        long,
        value_parser,
        verbatim_doc_comment,
        default_value_t = false
    )]
    pub csv: bool,

    /// Set the field delimiter to use when writing CSV (default: ';')
    #[arg(
        short('d'),
        long,
        env("DELIMITER_CSV"),
        requires = "csv",
        required = false,
        default_value_t = ';'
    )]
    pub delimiter: char,

    /// Set the memory consumption mode when writing Excel files
    ///
    /// Available options: constant, low, memory
    #[arg(
        long = "memory-mode",
        value_enum,
        required = false,
        default_value_t = ExcelMemoryMode::default()
    )]
    pub memory_mode: ExcelMemoryMode,

    /// Display fiscal document keys (CTe and NFe) not found
    ///
    /// Example:
    ///
    /// read_xml -tae > output_missing_docs
    #[arg(short, long, default_value_t = false)]
    pub exibir_chaves_nao_encontradas: bool,

    /// Display correlated fiscal document keys (CTe and NFe)
    ///
    /// Examples:
    ///
    /// read_xml -ta --exibir-correlacoes > output_correlated_docs
    ///
    /// read_xml -taei 100 --exibir-correlacoes > output_missing_and_correlated_docs
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
    /// Items are collected in descending order of value
    #[arg(short('i'), long("itens"), required = false, default_value_t = 100)]
    pub itens: usize,

    /// Set the maximum number of lines in the output files with the found keys
    ///
    /// Each file will contain at most N 44-digit fiscal document keys
    /// (generated as nfes-00001.txt, ctes-00001.txt, etc.)
    #[arg(
        short,
        long,
        required = false,
        value_parser = clap::value_parser!(u64).range(900..)
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

    /// Transform CTe and NFe output to the Receita Federal standard layout (56 columns)
    ///
    /// Example:
    ///
    /// read_xml -catr
    #[arg(short('r'), long("receita"), default_value_t = false)]
    pub receita: bool,

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

    /// Verify XML tag consistency against mapped structs only (without generating spreadsheets)
    #[arg(short('k'), long("verificar-consistencia"), default_value_t = false)]
    pub verificar_consistencia: bool,

    /// Wipe (Clear) the terminal screen before processing XML files
    #[arg(short('w'), long("wipe_terminal"), default_value_t = false)]
    pub wipe_terminal: bool,

    /// Show intermediate runtime messages
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> XmlParserResult<Arguments> {
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
        process::exit(0);
    }
}
