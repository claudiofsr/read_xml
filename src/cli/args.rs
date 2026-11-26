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
    /// Retain at most N fiscal documents per type during XML extraction (streaming sampling)
    #[arg(long("amostragem"), required = false)]
    pub amostragem: Option<usize>,

    /// Retain X fiscal documents with the highest values when --amostragem is enabled
    #[arg(
        long("maiores"),
        alias("amostra-maiores"),
        requires = "amostragem",
        required = false
    )]
    pub amostra_maiores: Option<usize>,

    /// Retain Y random fiscal documents uniformly when --amostragem is enabled
    #[arg(
        long("random"),
        alias("amostra-aleatoria"),
        requires = "amostragem",
        required = false
    )]
    pub amostra_aleatoria: Option<usize>,

    /// Optional deterministic seed for reproducible random sampling (--amostragem)
    #[arg(long("seed"), requires = "amostragem", required = false)]
    pub sample_seed: Option<u64>,

    /// Avoid creating XLSX files after parsing XML files
    #[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
    pub avoid: bool,

    /// Free intermediate buffers early and limit peak RAM usage
    ///
    /// Parses XML files in bounded batches, releases intermediate buffers early,
    /// exports datasets sequentially, and forces Excel constant-memory mode.
    #[arg(short('b'), long("buffer"), default_value_t = false)]
    pub buffer: bool,

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

    /// Match original Receita Federal CSVs strictly: *DIVERSOS* on CTe rows and <N/D> on NFe rows (-r)
    #[arg(long("diversos"), requires = "receita", default_value_t = false)]
    pub diversos: bool,

    /// Display fiscal document keys (CTe and NFe) not found
    ///
    /// With --filtro, lists the requested keys not found in the batch instead:
    /// pure 44-character keys, one per line on stdout (no headers), ready to be
    /// redirected to a file and reused as another --filtro input.
    ///
    /// Examples:
    ///
    /// read-xml -tae > output_missing_docs
    ///
    /// read-xml -catr -f chaves_extraidas.txt -e > chaves_nao_encontradas.txt
    #[arg(short, long, default_value_t = false)]
    pub exibir_chaves_nao_encontradas: bool,

    /// Display correlated fiscal document keys (CTe and NFe)
    ///
    /// Examples:
    ///
    /// read-xml -ta --exibir-correlacoes > output_correlated_docs
    ///
    /// read-xml -taei 100 --exibir-correlacoes > output_missing_and_correlated_docs
    #[arg(long, default_value_t = false)]
    pub exibir_correlacoes: bool,

    /// Filter fiscal documents by 44-character access keys listed in a text file
    ///
    /// One key per line; blank lines and `#` comments are ignored.
    /// NF-e items are kept when their key is listed; CT-e documents are kept when
    /// their own key or any referenced NF-e key is listed; events/cancellations are
    /// kept when the affected document key is listed. e-Financeira is discarded
    /// while filtering (no 44-character access key).
    ///
    /// Example:
    ///
    /// read-xml -catr -f chaves_extraidas.txt
    #[arg(
        short('f'),
        long("filtro"),
        visible_alias("chaves"),
        required = false,
        conflicts_with = "verificar_consistencia"
    )]
    pub filtro: Option<PathBuf>,

    /// Format float columns in Brazilian style (thousands '.', decimal ',')
    ///
    /// By default the Receita layout (-r) keeps the native XML/f64 format
    /// (123456.78 ; 0.3456 ; 3.0 ; 0.0), matching the native -c output.
    /// Use this flag only to compare against Receita Federal extracts (1.234,56).
    ///
    /// Integer codes (CRT, note/item number, CFOP, NCM, DI) and 44-digit keys are
    /// never affected. Only the representation changes, never the magnitude.
    #[arg(long("format-br"), requires = "receita", default_value_t = false)]
    pub format_br: bool,

    /**
    If provided, outputs the completion file for given shell.

    ### How to generate shell completions for Z-shell:

    #### Example (as root):

    Generate completions to read-xml.

    Visible to all system users.

    ```console

        mkdir -p /usr/local/share/zsh/site-functions

        read-xml --generate=zsh > /usr/local/share/zsh/site-functions/_read_xml

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

    /// Print nodes from XML files
    #[arg(short('n'), long("nodes"), required = false)]
    pub nodes: Option<PathBuf>,

    /// Set a .zip/.xml file or a directory path, otherwise recursively search
    /// for .zip and .xml files in the current directory
    #[arg(short('p'), long("path"), required = false)]
    pub path: Option<PathBuf>,

    /// Transform CTe and NFe output to the Receita Federal standard layout (56 columns)
    ///
    /// Example:
    ///
    /// read-xml -catr
    #[arg(short('r'), long("receita"), default_value_t = false)]
    pub receita: bool,

    /// Parse CTe or NFe XML file and print Rust struct
    ///
    /// Save this result as a file.rs and use it to (de)serialize an XML document with serde
    ///
    /// read-xml -s cte.xml > file.rs
    #[arg(short('s'), long("structure"), required = false)]
    pub structure: Option<PathBuf>,

    /// Show total execution time
    #[arg(short('t'), long("time"), default_value_t = false)]
    pub time: bool,

    /// Verify XML tag consistency in .zip files and loose .xml files (without generating spreadsheets)
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
