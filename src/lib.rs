mod args;
mod csv;
mod example01;
mod nodes;
mod regex;
mod xml_structs;

mod excel;
//mod excel_with_polars;

/// A module that exports the `ExcelWriter` struct which implements the Polars
/// `SerWriter` trait to serialize a dataframe to an Excel Xlsx file.
///
/// authors = ["John McNamara <jmcnamara@cpan.org>"]
///
/// repository = "https://github.com/jmcnamara/polars_excel_writer"
// mod write;

/// A module that exports the `PolarsXlsxWriter` struct which provides an Excel
/// Xlsx serializer that works with Polars dataframes and which can also
/// interact with the [`rust_xlsxwriter`] writing engine that it wraps.
// mod xlsx_writer;

pub use self::{
    args::*,
    csv::write_csv,
    excel::write_xlsx,
    xml_structs::cte_version_3_00::{CteProc, InfoCte},
    xml_structs::cte_evento::{ProcEventoCte, InfoCteEvento},
    xml_structs::nfe_version_4_00::{NfeProc,InfoNfe},
    xml_structs::nfe_evento::{ProcEventoNfe, InfoNfeEvento},
    xml_structs::efinanceira::{EFinanceira, InfoEFinanceira},
    regex::*,
    // write::ExcelWriter,
    // xlsx_writer::PolarsXlsxWriter,
};

use indicatif::{
    ProgressBar,
    ProgressStyle,
    MultiProgress,
};

use quick_xml::{
    events::Event,
    reader::Reader,
    de::from_reader,
};

use std::{
    env,
    str,
    fs::File,
    fmt::Debug,
    ops::Deref,
    io::{BufReader, Write},
    string::ToString,
    process::exit,
    path::{PathBuf, Path},
    collections::{BTreeMap, HashSet},
    thread,
};

use claudiofsr_lib::StrExtension;
use chrono::NaiveDate;
use rayon::prelude::*;
use serde::{Serialize, Deserialize, Serializer, de::DeserializeOwned};
use xml_schema_generator::{into_struct, Options};
use walkdir::{WalkDir, DirEntry};

pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;

/**
To serialize a field in a struct, do the following:
```ignore
    #[derive(Debug, Default, Serialize, Clone)]
    pub struct StructName {
        field_a: Option<String>,
        field_b: Option<u32>,
        #[serde(serialize_with = "serialize_vec_string")]
        field_c: Vec<String>,
    }
```
<https://serde.rs/impl-serialize.html>
*/
pub fn serialize_vec_string<S>(vec_string: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if vec_string.is_empty() {
        serializer.collect_str("")
    } else {
        let text = vec_string.join(", ");
        serializer.collect_str(&format!("[{text}]"))
    }
}

/// Polar arguments with ENV vars
pub fn configure_the_environment() {
    // https://stackoverflow.com/questions/70830241/rust-polars-how-to-show-all-columns/75675569#75675569
    // https://docs.rs/polars/latest/polars/#config-with-env-vars
    env::set_var("POLARS_FMT_TABLE_ROUNDED_CORNERS", "1"); // apply rounded corners to UTF8-styled tables.
    env::set_var("POLARS_FMT_TABLE_INLINE_COLUMN_DATA_TYPE", "0"); // put column data type on the same line as the column name.
    //env::set_var("POLARS_FMT_MAX_COLS", "20"); // maximum number of columns shown when formatting DataFrames.
    env::set_var("POLARS_FMT_MAX_ROWS", "10");   // maximum number of rows shown when formatting DataFrames.
    env::set_var("POLARS_FMT_STR_LEN", "52");    // maximum number of characters printed per string value.
}

pub struct MultiProgressBar {
    pub show_parse: ProgressBar,
    pub show_print: ProgressBar,
}

/// Add some methods
///
/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
pub trait StructExtension {
    /// Parse XML File into struct T
    fn xml_parse(path: &Path) -> MyResult<Self>
    where
        Self: DeserializeOwned,
    {
        // Attempts to open a file in read-only mode.
        let file = File::open(path)?;
        let mut bufreader: BufReader<File> = BufReader::new(file);

        // Try to deserialize the XML file into struct T
        Ok(from_reader(&mut bufreader)?)
    }

    /// struct T to Information
    fn struct_to_info(xml_path: &Path, arguments: &Arguments) -> Information
    where
        Self: DeserializeOwned,
    {
        match Self::xml_parse(xml_path) {
            Ok(proc) => proc.get_information(xml_path, arguments),
            Err(err) => Self::print_error_msgs(&err, xml_path),
        }
    }

    fn get_information(&self, xml_path: &std::path::Path, arguments: &crate::Arguments) -> Information;

    /**
    Print error messages. Examples:

    Structure Name: `read_xml::xml_structs::cte_version_3_00::CteProc`
    missing field `CTe`

    Structure Name: `read_xml::xml_structs::nfe_version_4_00::NfeProc`
    duplicate field `NFe`

    Structure Name: `read_xml::xml_structs::cte_evento::ProcEventoCte`
    missing field `eventoCTe`

    Structure Name: `read_xml::xml_structs::nfe_evento::ProcEventoNfe`
    missing field `evento`

    Structure Name: `read_xml::xml_structs::efinanceira::EFinanceira`
    missing field `evtMovOpFin`
    */
    fn print_error_msgs(err: &MyError, xml_path: &Path) -> Information
    where
        Self: Sized
    {

        let error_str: String = format!("{err}");

        if !REGEX_FIELDS.is_match(&error_str) {

            let mut buffer: Vec<u8> = Vec::new();
            let write: Box<&mut dyn Write> = Box::new(&mut buffer);

            let typename = std::any::type_name::<Self>();

            writeln!(*write, "\n").unwrap();
            writeln!(*write, "Structure Name: {typename:?}\n").unwrap();
            writeln!(*write, "{error_str}\n").unwrap();

            if REGEX_ERROR_MISSING_FIELD.is_match(&error_str) {
                writeln!(*write, "Faça a correção de").unwrap();
                writeln!(*write, "\tfield `XXX`: Type_Y").unwrap();
                writeln!(*write, "para").unwrap();
                writeln!(*write, "\tfield `XXX`: Option<Type_Y>\n").unwrap();
            }

            if REGEX_ERROR_DUPLICATE_FIELD.is_match(&error_str) {
                writeln!(*write, "Faça a correção de").unwrap();
                writeln!(*write, "\tfield `XXX`: Type_Y").unwrap();
                writeln!(*write, "para").unwrap();
                writeln!(*write, "\tfield `XXX`: Vec<Type_Y>\n").unwrap();
            }

            writeln!(*write, "Para solucionar este erro, veja os campos da estrutura com o comando:\n").unwrap();
            writeln!(*write, "read_xml -s {xml_path:?}\n").unwrap();

            my_print(&buffer);

            exit(1);
        }

        Information::None
    }
}

// https://doc.rust-lang.org/book/ch10-01-syntax.html?highlight=option#in-enum-definitions
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
//#[enum_delegate::implement(ParseXML)]
pub enum Information {
    Cte(Box<InfoCte>),
    Nfe(Vec<InfoNfe>),
    EventoCte(Box<InfoCteEvento>),
    EventoNfe(Box<InfoNfeEvento>),
    EFinanceira(Vec<InfoEFinanceira>),
    #[default]
    None,
}

impl Information {
    /// Returns `true` if the Information is a [`None`] value
    ///
    /// <https://doc.rust-lang.org/src/core/option.rs.html#642>
    pub const fn is_none(&self) -> bool {
        matches!(*self, Self::None)
    }

    pub const fn is_cte(&self) -> bool {
        matches!(*self, Self::Cte(_))
    }

    pub const fn is_nfe(&self) -> bool {
        matches!(*self, Self::Nfe(_))
    }

    pub const fn is_evento_cte(&self) -> bool {
        matches!(*self, Self::EventoCte(_))
    }

    pub const fn is_evendo_nfe(&self) -> bool {
        matches!(*self, Self::EventoNfe(_))
    }

    pub const fn is_efinanceira(&self) -> bool {
        matches!(*self, Self::EFinanceira(_))
    }

    pub fn add_info_to_docs_fiscais(&self, docs_fiscais: &mut DocsFiscais) {
        match self {
            Information::Cte(info_cte) => docs_fiscais.ctes.push(*info_cte.clone()),
            Information::Nfe(info_nfe) => docs_fiscais.nfes.extend(info_nfe.clone()),
            Information::EventoCte(info_cte_evento) => docs_fiscais.eventos_cte.push(*info_cte_evento.clone()),
            Information::EventoNfe(info_nfe_evento) => docs_fiscais.eventos_nfe.push(*info_nfe_evento.clone()),
            Information::EFinanceira(info_efinanceira) => docs_fiscais.efinanceiras.extend(info_efinanceira.clone()),
            Information::None => (),
        }
    }
}

#[derive(Debug, Default)]
pub struct DocsFiscais {
    pub ctes: Vec<InfoCte>,
    pub nfes: Vec<InfoNfe>,
    pub eventos_cte: Vec<InfoCteEvento>,
    pub eventos_nfe: Vec<InfoNfeEvento>,
    pub efinanceiras: Vec<InfoEFinanceira>,
}

impl DocsFiscais {
    // https://doc.rust-lang.org/stable/rust-by-example/fn/methods.html
    pub fn new() -> Self {
        DocsFiscais::default()
    }

    /*
    /// Sort `Vec<InfoNfe>` by key
    pub fn sort_nfes(&mut self) {
        // rayon: Sorts the slice in parallel with a key extraction function.
        self.nfes.par_sort_by_key(|info_nfe| (
            info_nfe.emitente_cnpj.clone(),
            info_nfe.emitente_cpf.clone(),
            info_nfe.dh_emi,
            info_nfe.nfe.clone(),
            info_nfe.n_item
        ));
    }
    
    /// Sort `Vec<InfoCte>` by key
    pub fn sort_ctes(&mut self) {
        // rayon: Sorts the slice in parallel with a key extraction function.
        self.ctes.par_sort_by_key(|info_cte| (
            info_cte.emitente_cnpj.clone(),
            info_cte.emitente_cpf.clone(),
            info_cte.remetente_cnpj.clone(),
            info_cte.remetente_cpf.clone(),
            info_cte.destinatario_cnpj.clone(),
            info_cte.destinatario_cpf.clone(),
            info_cte.expedidor_cnpj.clone(),
            info_cte.expedidor_cpf.clone(),
            info_cte.recebedor_cnpj.clone(),
            info_cte.recebedor_cpf.clone(),
            info_cte.dh_emi,
            info_cte.cte.clone()
        ));
    }
    */

    /// Sort `DocsFiscais` by key
    pub fn sort(&mut self) {

        //self.sort_nfes();
        //self.sort_ctes();
        
        let results = thread::scope(|s| {
            // Sort `Vec<InfoNfe>` by key
            let thread_sort_nfes = s.spawn(|| {
                // rayon: Sorts the slice in parallel with a key extraction function.
                self.nfes.par_sort_by_key(|info_nfe| (
                    info_nfe.emitente_cnpj.clone(),
                    info_nfe.emitente_cpf.clone(),
                    info_nfe.dh_emi,
                    info_nfe.nfe.clone(),
                    info_nfe.n_item
                ));

            });
            // Sort `Vec<InfoCte>` by key
            let thread_sort_ctes = s.spawn(|| {
                // rayon: Sorts the slice in parallel with a key extraction function.
                self.ctes.par_sort_by_key(|info_cte| (
                    info_cte.emitente_cnpj.clone(),
                    info_cte.emitente_cpf.clone(),
                    info_cte.remetente_cnpj.clone(),
                    info_cte.remetente_cpf.clone(),
                    info_cte.destinatario_cnpj.clone(),
                    info_cte.destinatario_cpf.clone(),
                    info_cte.expedidor_cnpj.clone(),
                    info_cte.expedidor_cpf.clone(),
                    info_cte.recebedor_cnpj.clone(),
                    info_cte.recebedor_cpf.clone(),
                    info_cte.dh_emi,
                    info_cte.cte.clone()
                ));
            });

            // Wait for background thread to complete.
            // Call join() on each handle to make sure all the threads finish.
            // join() returns immediately when the associated thread completes.

            let threads: Vec<_> = [thread_sort_nfes, thread_sort_ctes]
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
                    panic!("thread::scope failed to sort!")
                }
            });
    }
}

/// Get path from arguments or from default (current directory).
pub fn get_path(opt_path: &Option<PathBuf>) -> MyResult<PathBuf> {
    let relative_path: PathBuf = match opt_path {
        Some(path) => {
            if std::path::Path::new(&path).try_exists()? {
                path.to_path_buf()
            } else {
                eprintln!("The path {path:?} was not found!");
                panic!("fn get_path()");
            }
        }
        None => PathBuf::from("."),
    };

    Ok(relative_path)
}

pub fn get_xml_entries(arguments: &Arguments) -> MyResult<Vec<DirEntry>> {

    let dir_path = get_path(&arguments.path)?;

    let max_depth: usize = match arguments.max_depth {
        Some(depth) => depth,
        None => std::usize::MAX,
    };

    let entries: Vec<DirEntry> = WalkDir::new(dir_path)
        .max_depth(max_depth)
        .into_iter()
        .map_while(|result| {
            match result {
                Ok(dir_entry) => Some(dir_entry),
                Err(why) => {
                    eprintln!("Error: {why}");
                    exit(2); // No such file or directory
                }
            }
        })
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry|
            entry.path().extension().is_some_and(|ext|
                ext.to_ascii_uppercase() == "XML"
            )
        )
        .collect();

    Ok(entries)
}

pub fn get_progressbar(total_size: usize) -> MyResult<MultiProgressBar> {

    let num_char = total_size.to_string().chars().count();

    let template =
        "{prefix:.bold.dim} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>nchar}/{len:>nchar} ({eta})"
        .replace("nchar", &num_char.to_string());

    let style: ProgressStyle = ProgressStyle::default_bar()
        .template(&template)?
        .progress_chars("#>-");

    let multi_progressbar = MultiProgress::new();

    let pb_parse = multi_progressbar.add(ProgressBar::new(total_size as u64));
    pb_parse.set_style(style.clone());

    let pb_print = multi_progressbar.add(ProgressBar::new(total_size as u64));
    pb_print.set_style(style);

    pb_parse.set_prefix("parse xml");
    pb_print.set_prefix("print xml");

    Ok(MultiProgressBar {
        show_parse: pb_parse,
        show_print: pb_print,
    })
}

/// Get all information from files and show progress bar
pub fn get_all_info(
    xml_entries: &[DirEntry],
    multi_progressbar: &mut MultiProgressBar,
    arguments: &Arguments
) -> Vec<Information>
{
    let infos: Vec<Information> = xml_entries
        .into_par_iter() // rayon parallel iterator
        //.iter()
        .flat_map(|entry| {
            multi_progressbar.show_parse.inc(1);
            let xml_path = entry.path();
            get_xml_serialized(xml_path, arguments)
        })
        .collect();

    multi_progressbar.show_parse.finish();

    infos
}

fn get_xml_serialized(xml_path: &Path, arguments: &Arguments) -> Vec<Information> {
    vec![
        CteProc::struct_to_info(xml_path, arguments),
        NfeProc::struct_to_info(xml_path, arguments),
        ProcEventoCte::struct_to_info(xml_path, arguments),
        ProcEventoNfe::struct_to_info(xml_path, arguments),
        EFinanceira::struct_to_info(xml_path, arguments),
    ]
    //.into_par_iter()
    .into_iter()
    .filter(|info| !info.is_none())
    .collect()
}

/// Print buffer to stdout
pub fn my_print(buffer: &[u8]) {
    // Converts a slice of bytes to a string slice
    let print_msg = match str::from_utf8(buffer) {
        Ok(valid_uft8) => valid_uft8,
        Err(error) => {
            eprintln!("fn my_print()");
            eprintln!("Invalid UTF-8 sequence!");
            panic!("{error}");
        }
    };

    // Print to stdout
    print!("{print_msg}");
}

pub fn parse_xml_and_print_struct(xml_path: &PathBuf) -> MyResult<()> {

    let mut reader = Reader::from_file(xml_path)?;

    if let Ok(root) = into_struct(&mut reader) {
        // Options: quick_xml_de(), serde_xml_rs(), derive()
        let struct_as_string = root
            .to_serde_struct(&Options::quick_xml_de());
        // save this result as a .rs file and use it to (de)serialize an XML document with serde
        println!("{}", struct_as_string.trim());
    }

    Ok(())
}

pub fn get_naive_date_from_yyyy_mm_dd<T>(date: &Option<T>) -> Option<NaiveDate>
    where
        T: Deref<Target=str> + std::fmt::Display,
{
    // date: YYYY-MM-DD
    // "2020-09-04T10:48:18-03:00" -> "202009041048180300"
    let digits: Option<String> = date
        .as_ref()
        .map(|d| d.remove_non_digits());

    let yyyymmdd: &str = match &digits {
        Some(d) if d.chars_count() >= 8 => &d[..8],
        _ => return None,
    };

    match NaiveDate::parse_from_str(yyyymmdd, "%Y%-m%-d") {
        Ok(dt) => Some(dt),
        Err(why) => {
            eprintln!("fn get_naive_date()");
            eprintln!("Data inválida ou inexistente!");
            eprintln!("Erro: {why}");
            eprintln!("\t'{yyyymmdd}'");
            None
        }
    }
}

pub fn get_naive_date_from_yyyymm<T>(date: &Option<T>) -> Option<NaiveDate>
    where
        T: Deref<Target=str> + std::fmt::Display,
{
    // date: YYYYMM
    // ano_mes_caixa: "201901"
    let digits: Option<String> = date
        .as_ref()
        .map(|d| d.remove_non_digits());

    let yyyymm: &str = match &digits {
        Some(d) if d.chars_count() >= 6 => &d[..6],
        _ => return None,
    };

    let yyyymmdd = [yyyymm, "01"].concat(); // adicionar dia 01

    match NaiveDate::parse_from_str(&yyyymmdd, "%Y%-m%-d") {
        Ok(dt) => Some(dt),
        Err(why) => {
            eprintln!("fn get_naive_date_from_yyyymm()");
            eprintln!("Data inválida ou inexistente!");
            eprintln!("Erro: {why}");
            eprintln!("\t'{yyyymm}'");
            None
        }
    }
}

pub fn atualizar_nfes_cancelados(nfes: &mut [InfoNfe], eventos_nfe: &[InfoNfeEvento]) {

    let mut nfes_canceladas: HashSet<String> = HashSet::new();

    eventos_nfe
        .iter()
        .filter(|evento_nfe| evento_nfe.cancelado)
        .for_each(|evento_nfe| {
            if let Some(nfe) = &evento_nfe.nfe {
                //println!("NFe: {nfe} cancelada");
                nfes_canceladas.insert(nfe.to_string());
            }
        });

    //println!("nfes_canceladas: {nfes_canceladas:#?}");

    nfes
        .iter_mut()
        .for_each(|info_nfe| {
            if let Some(nfe) = &info_nfe.nfe {
                if nfes_canceladas.contains(nfe) {
                    info_nfe.cancelado = Some("Sim".to_string());
                }
            }
        });
}

pub fn atualizar_ctes_cancelados(ctes: &mut [InfoCte], eventos_cte: &[InfoCteEvento]) {

    let mut ctes_cancelados: HashSet<String> = HashSet::new();

    eventos_cte
        .iter()
        .filter(|evento_cte| evento_cte.cancelado)
        .for_each(|evento_cte| {
            if let Some(cte) = &evento_cte.cte {
                //println!("NFe: {cte} cancelada");
                ctes_cancelados.insert(cte.to_string());
            }
        });

    //println!("ctes_canceladas: {ctes_canceladas:#?}");

    ctes
        .iter_mut()
        .for_each(|info_cte| {
            if let Some(cte) = &info_cte.cte {
                if ctes_cancelados.contains(cte) {
                    info_cte.cancelado = Some("Sim".to_string());
                }
            }
        });
}

// Not used!
pub fn deep_keys(reader: &mut Reader<BufReader<File>>, filter: bool) -> BTreeMap<String, Vec<String>> {

    // if filter, capture information only from these fields:
    const FIELDS: [&str; 3] = [
        "chCTe",
        "chave",
        "chNFe", // "refNFe",
    ];

    let mut buf: Vec<u8> = Vec::new();
    let mut key: String = String::new();
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(node)) => {
                let bytes: Vec<u8> = node.name().as_ref().to_vec();
                key = match String::from_utf8(bytes) {
                    Ok(string) => string,
                    Err(why) => panic!("Failed to convert bytes to UTF-8!: {why}"),
                };
                //println!("key: {key}");
            },
            Ok(Event::Text(node)) => {
                let value = node.unescape().expect("Invalid UTF-8!").into_owned();
                //println!("value: {value}");

                if !filter || FIELDS.iter().any(|field| key.contains(field)) {
                    map.entry(key.clone()).or_default().push(value);
                }
            },
            Ok(Event::Eof) => break,
            Err(why) => panic!("Error at position {}: {why}", reader.buffer_position()),
            _ => (),
        }
        buf.clear();
    }

    map
}

#[cfg(test)]
mod lib_functions {
    use super::*;

    // cargo test -- --help
    // cargo test -- --show-output
    // cargo test -- --show-output multiple_values

    #[test]
    /// `cargo test -- --show-output deep_keys_test`
    fn deep_keys_test() -> MyResult<()> {

        let filter = false;

        let _xml_cte = PathBuf::from(r"35220998765432101234567894741048320396789012_CTe.xml");
        let _xml_nfe = PathBuf::from(r"35220412345678901234567890123456789012345678_NFe.xml");

        let mut reader = Reader::from_file(_xml_cte)?;

        let map: BTreeMap<String, Vec<String>> = deep_keys(&mut reader, filter);

        for (index, xml) in map.iter().enumerate() {
            println!("field {}: {xml:#?}\n", index + 1);
        }

        Ok(())
    }

    // google: cte xml 4.0
    // CTe Esquemas XML
    // https://www.cte.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=0xlG1bdBass=

    // NFe Esquemas XML
    // https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=BMPFMBoln3w=

    #[test]
    /// https://docs.rs/xml_schema_generator/latest/xml_schema_generator/
    ///
    /// `cargo test -- --show-output parse_xml_and_get_struct_from_str`
    fn parse_xml_and_get_struct_from_str() -> MyResult<()> {

        use quick_xml::reader::Reader;
        use xml_schema_generator::into_struct;

        let xml = "<a b=\"c\">d</a>";
        let mut reader = Reader::from_str(xml);

        if let Ok(root) = into_struct(&mut reader) {
            let struct_as_string = root.to_serde_struct(&Options::quick_xml_de());
            // save this result as a .rs file and use it to (de)serialize an XML document with serde
            println!("struct_as_string: {struct_as_string}");
        }

        Ok(())
    }

    #[test]
    /// https://docs.rs/xml_schema_generator/latest/xml_schema_generator/
    ///
    /// `cargo test -- --show-output parse_xml_and_get_struct_from_file`
    ///
    /// `cargo test -- --show-output parse_xml_and_get_struct_from_file > /tmp/xml_struct.rs`
    fn parse_xml_and_get_struct_from_file() -> MyResult<()> {

        let _xml_cte = PathBuf::from(r"35220998765432101234567894741048320396789012_CTe.xml");
        let _xml_nfe = PathBuf::from(r"35220412345678901234567890123456789012345678_NFe.xml");

        parse_xml_and_print_struct(&_xml_cte)?;

        Ok(())
    }
}