use clap::{Parser, ArgAction};
use std::{
    path::PathBuf,
    process::exit,
};

use crate::{
    MyResult,
    nodes::print_nodes,
    parse_xml_and_print_struct,
};

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
fn get_styles() -> clap::builder::Styles {

    let cyan   = anstyle::Color::Ansi(anstyle::AnsiColor::Cyan);
    let green  = anstyle::Color::Ansi(anstyle::AnsiColor::Green);
    let yellow = anstyle::Color::Ansi(anstyle::AnsiColor::Yellow);

    clap::builder::Styles::styled()
        .placeholder(
            anstyle::Style::new()
                .fg_color(Some(yellow))
        )
        .usage(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
        )
        .header(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
                .underline()
        )
        .literal(
            anstyle::Style::new()
                .fg_color(Some(green))
        )
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

    /// Avoid creating .xlsx after parsing xml files
    ///
    /// Para evitar a criação de arquivos .xlsx, adicione a opção: --avoid ou -a
    #[arg(short, long, default_value_t = false, action=ArgAction::SetTrue)]
    pub avoid: bool,

    /// Write CSV (Comma Separated Values) files
    ///
    /// Para imprimir o arquivo .csv, adicione a opção: --csv ou -c
    #[arg(short, long, value_parser, verbatim_doc_comment, default_value_t = false)]
    pub csv: bool,

    /// Set the field delimiter to use when writing CSV
    ///
    /// The default is b';'
    #[arg(short('d'), long, env("DELIMITER_CSV"), requires = "csv", required = false, default_value_t = ';')]
    pub delimiter: char,

    /// Set maximum depth to recursively search xml files
    ///
    /// Avoid descending into directories when the depth is exceeded
    #[arg(short('m'), long("max_depth"), required = false)]
    pub max_depth: Option<usize>,

    /// Print nodes from xml files
    #[arg(short('n'), long("nodes"), required = false)]
    pub nodes: Option<PathBuf>,

    /// Set the xml file path, otherwise recursively search
    /// for xml files in the current directory
    #[arg(short('p'), long("path"), required = false)]
    pub path: Option<PathBuf>,

    /// Parse CTe or NFe xml file and print Rust struct
    ///
    /// Save this result as a file.rs and use it to (de)serialize an XML document with serde
    ///
    /// read_xml -s cte.xml > file.rs
    #[arg(short('s'), long("structure"), required = false)]
    pub structure: Option<PathBuf>,

    /// Show total execution time
    #[arg(short('t'), long("time"), default_value_t = false)]
    pub time: bool,

    /// Show intermediate runtime messages.
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> MyResult<Arguments> {
        let args: Arguments = Arguments::parse();

        if let Some(xml_path) = &args.structure {
            parse_xml_and_print_struct(xml_path)?;
            exit(0); // success
        }

        if let Some(xml_path) = &args.nodes {
            print_nodes(xml_path)?;
            exit(0); // success
        }

        if args.avoid && !args.csv {
            // https://doc.rust-lang.org/cargo/reference/environment-variables.html
            let app_name = env!("CARGO_PKG_NAME");
            eprintln!("Requires at least CSV or XLSX output files!");
            eprintln!("See: {app_name} --help");
            exit(-1); // failure
        }

        Ok(args)
    }
}
