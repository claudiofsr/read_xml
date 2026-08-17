//! # Utilitários Gerais de I/O, Datas e Serialização
//!
//! Funções auxiliares sem efeitos colaterais para sanitização, parsing temporal
//! e formatação customizada de tipos de dados fiscais.

use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use quick_xml::reader::Reader;
use serde::Serializer;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    str,
};
use walkdir::{DirEntry, WalkDir};
use xml_schema_generator::{Options, into_struct};

use crate::{Arguments, XmlParserError, XmlParserResult};

/// Representa os caminhos de saída para os arquivos gerados (planilhas e CSVs).
#[derive(Debug, Clone)]
pub struct OutputFilename {
    pub ctes: PathBuf,
    pub nfes: PathBuf,
    pub efin: PathBuf,
}

impl OutputFilename {
    /// Altera em lote a extensão dos três arquivos de saída.
    #[inline]
    pub fn set_extension(&mut self, extension: &str) {
        self.ctes.set_extension(extension);
        self.nfes.set_extension(extension);
        self.efin.set_extension(extension);
    }
}

impl Default for OutputFilename {
    fn default() -> Self {
        Self {
            ctes: PathBuf::from("documentos_fiscais-ctes"),
            nfes: PathBuf::from("documentos_fiscais-nfes"),
            efin: PathBuf::from("documentos_fiscais-efinanceiras"),
        }
    }
}

/// Serializa um slice de strings em formato textual envolto por colchetes: `[item1, item2]`.
///
/// Caso a coleção esteja vazia, retorna uma string em branco.
pub fn serialize_vec_string<S>(vec_string: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if vec_string.is_empty() {
        serializer.collect_str("")
    } else {
        serializer.collect_str(&format!("[{}]", vec_string.join(", ")))
    }
}

/// Converte datas no formato textual `YYYY-MM-DD` ou `YYYY-MM-DDTHH:MM:SS` para [`NaiveDate`].
pub fn get_naive_date_from_yyyy_mm_dd<T>(date: &Option<T>) -> Option<NaiveDate>
where
    T: std::ops::Deref<Target = str> + std::fmt::Debug,
{
    // date: YYYY-MM-DD
    // "2020-09-04T10:48:18-03:00" -> "202009041048180300"
    let digits = date.as_ref()?.remove_non_digits();

    if digits.chars_count() >= 8 {
        match NaiveDate::parse_from_str(&digits[..8], "%Y%-m%-d") {
            Ok(dt) => Some(dt),
            Err(why) => {
                eprintln!("Error parsing date: '{date:?}'");
                eprintln!("Failed to parse date: {why}");
                None
            }
        }
    } else {
        None
    }
}

/// Converte referências de ano/mês no formato `YYYYMM` (comum no SPED/e-Financeira) para [`NaiveDate`].
pub fn get_naive_date_from_yyyymm<T>(date: &Option<T>) -> Option<NaiveDate>
where
    T: std::ops::Deref<Target = str>,
{
    let digits = date.as_ref()?.remove_non_digits();
    if digits.chars_count() >= 6 {
        let yyyymmdd = format!("{}01", &digits[..6]); // adicionar dia 01
        match NaiveDate::parse_from_str(&yyyymmdd, "%Y%-m%-d") {
            Ok(dt) => Some(dt),
            Err(why) => {
                eprintln!("fn get_naive_date_from_yyyymm()");
                eprintln!("Data inválida ou inexistente!");
                eprintln!("Erro: {why}");
                eprintln!("\t'{digits}'");
                None
            }
        }
    } else {
        None
    }
}

/// Valida e retorna o diretório informado via CLI ou o diretório atual como fallback.
pub fn get_path(opt_path: &Option<PathBuf>) -> XmlParserResult<PathBuf> {
    match opt_path {
        Some(path) => {
            if path.try_exists()? {
                Ok(path.clone())
            } else {
                Err(XmlParserError::PathNotFound { path: path.clone() })
            }
        }
        None => Ok(PathBuf::from(".")),
    }
}

/// Realiza a varredura recursiva de arquivos com extensão `.xml` no diretório configurado.
pub fn get_xml_entries(arguments: &Arguments) -> XmlParserResult<Vec<DirEntry>> {
    let dir_path = get_path(&arguments.path)?;
    let max_depth = arguments.max_depth.unwrap_or(usize::MAX);

    let entries: Vec<DirEntry> = WalkDir::new(dir_path)
        .max_depth(max_depth)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("XML"))
        })
        .collect();

    Ok(entries)
}

/// Imprime a estrutura Rust gerada a partir da inferência de um arquivo XML.
pub fn parse_xml_and_print_struct(xml_path: &Path) -> XmlParserResult<()> {
    let file = File::open(xml_path).map_err(|err| XmlParserError::IoContext {
        source: err,
        path: xml_path.to_path_buf(),
    })?;
    let mut reader = Reader::from_reader(BufReader::new(file));

    if let Ok(root) = into_struct(&mut reader) {
        // Options: quick_xml_de(), serde_xml_rs(), derive()
        let struct_as_string = root.to_serde_struct(&Options::quick_xml_de());
        // save this result as a .rs file and use it to (de)serialize an XML document with serde
        println!("{}", struct_as_string.trim());
    }

    Ok(())
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output tests_utils
#[cfg(test)]
mod tests_utils {
    use super::*;

    // google: cte xml 4.0
    // CTe Esquemas XML
    // https://www.cte.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=0xlG1bdBass=

    // NFe Esquemas XML
    // https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=BMPFMBoln3w=

    #[test]
    /// https://docs.rs/xml_schema_generator/latest/xml_schema_generator/
    ///
    /// `cargo test -- --show-output parse_xml_and_get_struct_from_str`
    fn parse_xml_and_get_struct_from_str() -> XmlParserResult<()> {
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
    fn parse_xml_and_get_struct_from_file() -> XmlParserResult<()> {
        let _xml_cte = PathBuf::from(r"35220998765432101234567894741048320396789012_CTe.xml");
        let _xml_nfe = PathBuf::from(r"35220412345678901234567890123456789012345678_NFe.xml");

        parse_xml_and_print_struct(&_xml_cte)?;

        Ok(())
    }
}
