mod args;
mod csv;
mod example01;
mod excel;
mod group_by_hashmap;
mod my_traits;
mod nodes;
mod regex;
mod unique_with_cows;
mod xml_structs;

pub use self::{
    args::*,
    csv::CsvWriter,
    excel::write_xlsx,
    group_by_hashmap::{GetKey, GroupByHashMapExt},
    my_traits::{GetFirst, GroupBy, OptExt},
    regex::*,
    unique_with_cows::UniqueIdentification,
    xml_structs::{
        agente::TOMADOR_DO_SERVICO,
        cancelamento_cte::{InfoCteCancel, ProcCancCte},
        cancelamento_nfe::{InfoNfeCancel, ProcCancNfe},
        cte_evento::{InfoCteEvento, ProcEventoCte},
        cte_version_3_00::{CteProc, InfoCte},
        efinanceira::{EFinanceira, InfoEFinanceira},
        nfe_evento::{InfoNfeEvento, ProcEventoNfe},
        nfe_version_4_00::{InfoNfe, NfeProc},
    },
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use quick_xml::{de::from_reader, events::Event, reader::Reader};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::{self, Debug},
    fs::File,
    hash::Hash,
    io::{BufReader, Write},
    ops::Deref,
    path::{Path, PathBuf},
    process::exit,
    str,
    string::ToString,
    thread,
};

use chrono::NaiveDate;
use claudiofsr_lib::{BTreeSetExtension, HashSetExtension, RoundFloat, StrExtension};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned};
use walkdir::{DirEntry, WalkDir};
use xml_schema_generator::{Options, into_struct};

pub type MyError = Box<dyn std::error::Error + Send + Sync + 'static>;
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

pub struct OuputFilename {
    pub ctes: PathBuf,
    pub nfes: PathBuf,
    pub efin: PathBuf,
}

impl Default for OuputFilename {
    fn default() -> Self {
        OuputFilename {
            ctes: "read_xml-ctes".into(),
            nfes: "read_xml-nfes".into(),
            efin: "read_xml-efinanceiras".into(),
        }
    }
}

impl OuputFilename {
    pub fn set_extension(&mut self, extension: &str) {
        self.ctes.set_extension(extension);
        self.nfes.set_extension(extension);
        self.efin.set_extension(extension);
    }
}

#[derive(Debug)]
pub struct MultiProgressBar {
    pub show_parse: ProgressBar,
    pub show_print: ProgressBar,
    pub show_excel: ProgressBar,
    pub show_csval: ProgressBar,
}

impl Default for MultiProgressBar {
    fn default() -> Self {
        MultiProgressBar {
            show_parse: ProgressBar::new(0),
            show_print: ProgressBar::new(0),
            show_excel: ProgressBar::new(0),
            show_csval: ProgressBar::new(0),
        }
    }
}

impl MultiProgressBar {
    fn add_progressbar(
        &mut self,
        multi_progress: &MultiProgress,
        name: &str,
        total: usize,
    ) -> MyResult<ProgressBar> {
        let num_char = total.to_string().chars().count();

        let template =
            "{prefix:<10.bold.dim} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>nchar}/{len:>nchar} ({eta})"
            .replace("nchar", &num_char.to_string());

        let style: ProgressStyle = ProgressStyle::default_bar()
            .template(&template)?
            .progress_chars("#>-");

        let progressbar = multi_progress.add(ProgressBar::new(total as u64));
        progressbar.set_style(style);

        progressbar.set_prefix(name.to_string());

        Ok(progressbar)
    }

    pub fn add_parse_xml(&mut self, multi_progress: &MultiProgress, total: usize) -> MyResult<()> {
        let progressbar = self.add_progressbar(multi_progress, "parse xml", total)?;
        self.show_parse = progressbar;
        Ok(())
    }

    pub fn add_print_xml(&mut self, multi_progress: &MultiProgress, total: usize) -> MyResult<()> {
        let progressbar = self.add_progressbar(multi_progress, "print xml", total)?;
        self.show_print = progressbar;
        Ok(())
    }

    pub fn add_print_xls(&mut self, multi_progress: &MultiProgress, total: usize) -> MyResult<()> {
        let progressbar = self.add_progressbar(multi_progress, "write xlsx", total)?;
        self.show_excel = progressbar;
        Ok(())
    }

    pub fn add_print_csv(&mut self, multi_progress: &MultiProgress, total: usize) -> MyResult<()> {
        let progressbar = self.add_progressbar(multi_progress, "write csv", total)?;
        self.show_csval = progressbar;
        Ok(())
    }
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

    fn get_information(
        &self,
        xml_path: &std::path::Path,
        arguments: &crate::Arguments,
    ) -> Information;

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
        Self: Sized,
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

            writeln!(
                *write,
                "Para solucionar este erro, veja os campos da estrutura com o comando:\n"
            )
            .unwrap();
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
    CancelamentoCte(Box<InfoCteCancel>),
    CancelamentoNfe(Box<InfoNfeCancel>),
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
            Information::EventoCte(info_cte_evento) => {
                docs_fiscais.eventos_cte.push(*info_cte_evento.clone())
            }
            Information::EventoNfe(info_nfe_evento) => {
                docs_fiscais.eventos_nfe.push(*info_nfe_evento.clone())
            }
            Information::CancelamentoCte(info_cte_cancel) => {
                docs_fiscais.cancel_cte.push(*info_cte_cancel.clone())
            }
            Information::CancelamentoNfe(info_nfe_cancel) => {
                docs_fiscais.cancel_nfe.push(*info_nfe_cancel.clone())
            }
            Information::EFinanceira(info_efinanceira) => {
                docs_fiscais.efinanceiras.extend(info_efinanceira.clone())
            }
            Information::None => (),
        }
    }
}

pub trait KeysExtension {
    fn get_chaves(&self) -> BTreeSet<String>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyDoc {
    pub chave: String,
    pub valido: bool,
}

impl KeyDoc {
    fn new<T>(chave: T, valido: bool) -> Self
    where
        T: ToString,
    {
        KeyDoc {
            chave: chave.to_string(),
            valido,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Correlacoes {
    pub cte_info: BTreeMap<KeyDoc, Vec<InfoCte>>,
    pub nfe_info: BTreeMap<KeyDoc, Vec<InfoNfe>>,
    pub cte_nfes: HashMap<String, HashSet<String>>,
    pub nfe_ctes: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Default)]
pub struct DocsFiscais {
    pub ctes: Vec<InfoCte>,
    pub nfes: Vec<InfoNfe>,
    pub eventos_cte: Vec<InfoCteEvento>,
    pub eventos_nfe: Vec<InfoNfeEvento>,
    pub cancel_cte: Vec<InfoCteCancel>,
    pub cancel_nfe: Vec<InfoNfeCancel>,
    pub efinanceiras: Vec<InfoEFinanceira>,
}

impl DocsFiscais {
    // https://doc.rust-lang.org/stable/rust-by-example/fn/methods.html
    pub fn new() -> Self {
        DocsFiscais::default()
    }

    /// Nº de Documentos Fiscais distintos
    pub fn total(&self) -> usize {
        [
            self.ctes.is_empty(),
            self.nfes.is_empty(),
            self.efinanceiras.is_empty(),
        ]
        .into_iter()
        .map(|is_empty| if is_empty { 0 } else { 1 })
        .sum()
    }

    // https://blog.logrocket.com/using-cow-rust-efficient-memory-utilization/
    pub fn unique(&mut self) {
        thread::scope(|s| {
            s.spawn(|| self.nfes = self.nfes.get_unique_id());
            s.spawn(|| self.ctes = self.ctes.get_unique_id());
        });

        self.ctes
            .par_iter_mut()
            .for_each(|info| info.get_unique_elements());
    }

    /// Sort `DocsFiscais` by key
    pub fn sort(&mut self) {
        //self.sort_nfes();
        //self.sort_ctes();

        let results = thread::scope(|s| {
            // Sort `Vec<InfoNfe>` by key
            let thread_sort_nfes = s.spawn(|| {
                // rayon: Sorts the slice in parallel with a key extraction function.
                self.nfes.par_sort_by_key(|info_nfe| {
                    (
                        info_nfe.emitente_cnpj.clone(),
                        info_nfe.emitente_cpf.clone(),
                        info_nfe.data_emissao,
                        info_nfe.nfe.clone(),
                        info_nfe.n_item,
                    )
                });
            });

            // Sort `Vec<InfoCte>` by key
            let thread_sort_ctes = s.spawn(|| {
                // rayon: Sorts the slice in parallel with a key extraction function.
                self.ctes.par_sort_by_key(|info_cte| {
                    (
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
                        info_cte.data_emissao,
                        info_cte.cte.clone(),
                    )
                });
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

        results.into_iter().for_each(|result| {
            if result.is_err() {
                eprintln!("Error: {result:?}");
                panic!("thread::scope failed to sort!")
            }
        });
    }

    /// Relacionar KeyDoc com InfoCte
    ///
    /// keydoc.chave = cte
    fn groupby_cte_info(&self) -> BTreeMap<KeyDoc, Vec<InfoCte>> {
        self.ctes
            .par_iter() // rayon parallel iterator
            .flat_map(|info| {
                info.cte.as_ref().map(|cte| {
                    let keydoc = KeyDoc::new(cte, info.is_valid());
                    (keydoc, info.clone()) // (key, value)
                })
            })
            .fold(
                BTreeMap::new,
                |mut acc: BTreeMap<KeyDoc, Vec<InfoCte>>, (key, value)| {
                    acc.entry(key).or_default().push(value);
                    acc
                },
            )
            .reduce(BTreeMap::new, |mut acc, map| {
                map.into_iter().for_each(|(key, values)| {
                    acc.entry(key).or_default().extend(values);
                });
                acc
            })
    }

    /// Relacionar KeyDoc com InfoNfe
    ///
    /// keydoc.chave = nfe
    ///
    /// 1 NFe pode conter vários InfoNfe, tal que cada InfoNfe contém informação de 1 Item.
    fn groupby_nfe_info(&self) -> BTreeMap<KeyDoc, Vec<InfoNfe>> {
        self.nfes
            .par_iter() // rayon parallel iterator
            .flat_map(|info| {
                info.nfe.as_ref().map(|nfe| {
                    let keydoc = KeyDoc::new(nfe, info.is_valid());
                    (keydoc, info.clone()) // (key, value)
                })
            })
            .fold(
                BTreeMap::new,
                |mut acc: BTreeMap<KeyDoc, Vec<InfoNfe>>, (key, value)| {
                    acc.entry(key).or_default().push(value);
                    acc
                },
            )
            .reduce(BTreeMap::new, |mut acc, map| {
                map.into_iter().for_each(|(key, values)| {
                    acc.entry(key).or_default().extend(values);
                });
                acc
            })
    }

    fn get_ctes_nao_encontrados(
        &self,
        cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>,
    ) -> HashSet<String> {
        self.ctes
            .par_iter()
            .flat_map(|info| {
                info.get_correlated_ctes()
                    .into_iter()
                    .filter(|cte| {
                        let keydoc_val = KeyDoc::new(cte, true); // chave válida
                        let keydoc_can = KeyDoc::new(cte, false); // chave cancelada
                        !cte_info.contains_key(&keydoc_val) && !cte_info.contains_key(&keydoc_can)
                    })
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    fn get_nfes_nao_encontrados(
        &self,
        nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>,
    ) -> HashSet<String> {
        self.ctes
            .par_iter()
            .flat_map(|info| {
                info.get_correlated_nfes()
                    .into_iter()
                    .filter(|nfe| {
                        let keydoc_val = KeyDoc::new(nfe, true); // chave válida
                        let keydoc_can = KeyDoc::new(nfe, false); // chave cancelada
                        !nfe_info.contains_key(&keydoc_val) && !nfe_info.contains_key(&keydoc_can)
                    })
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    /// CTe relacionado a CTes
    ///
    /// HashMap<chave_cte, HashSet<chave_cte>
    fn groupby_cte_ctes(
        &self,
        cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>,
    ) -> HashMap<String, HashSet<String>> {
        self.ctes
            .par_iter()
            .filter_map(|info| {
                if let Some(cte_a) = &info.cte {
                    let ctes: HashSet<String> = info
                        .get_correlated_ctes()
                        .into_iter()
                        .filter(|cte_b| {
                            let chave_valida = KeyDoc::new(cte_b, true);
                            cte_b != cte_a && cte_info.contains_key(&chave_valida)
                        })
                        .collect();

                    Some((cte_a.clone(), ctes))
                } else {
                    None
                }
            })
            .collect()
    }

    /// CTe relacionado a NFes
    ///
    /// HashMap<chave_cte, HashSet<chave_nfe>
    fn groupby_cte_nfes(
        &self,
        nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>,
    ) -> HashMap<String, HashSet<String>> {
        self.ctes
            .par_iter()
            .filter_map(|info| {
                if let Some(cte) = &info.cte {
                    let nfes: HashSet<String> = info
                        .get_correlated_nfes()
                        .into_iter()
                        .filter(|nfe| {
                            let chave_valida = KeyDoc::new(nfe, true);
                            nfe != cte && nfe_info.contains_key(&chave_valida)
                        })
                        .collect();

                    Some((cte.clone(), nfes))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Obtain correlations between CTe and NFe
    pub fn get_correlations(&mut self, arguments: &Arguments) {
        let mut cte_info: BTreeMap<KeyDoc, Vec<InfoCte>> = BTreeMap::new(); // cte contém 1 item
        let mut nfe_info: BTreeMap<KeyDoc, Vec<InfoNfe>> = BTreeMap::new(); // nfe contém vários itens

        thread::scope(|s| {
            s.spawn(|| cte_info = self.groupby_cte_info());
            s.spawn(|| nfe_info = self.groupby_nfe_info());
        });

        let mut ctes_nao_encontrados: HashSet<String> = HashSet::new();
        let mut nfes_nao_encontrados: HashSet<String> = HashSet::new();

        let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::new();
        let mut cte_nfes: HashMap<String, HashSet<String>> = HashMap::new();

        thread::scope(|s| {
            s.spawn(|| ctes_nao_encontrados = self.get_ctes_nao_encontrados(&cte_info));
            s.spawn(|| nfes_nao_encontrados = self.get_nfes_nao_encontrados(&nfe_info));
            s.spawn(|| cte_ctes = self.groupby_cte_ctes(&cte_info));
            s.spawn(|| cte_nfes = self.groupby_cte_nfes(&nfe_info));
        });

        if arguments.exibir_chaves_nao_encontradas {
            show_docs("CTe", &ctes_nao_encontrados.to_vec_sorted());
            show_docs("NFe", &nfes_nao_encontrados.to_vec_sorted());
        }

        cte_ctes.expand_ctes(false);

        cte_nfes.expand_nfes(&cte_ctes);

        let cte_nfes = cte_nfes.filtrar_nfes_validos(&nfe_info);

        let nfe_ctes = cte_nfes.get_nfe_ctes();

        let nfe_ctes = nfe_ctes.filtrar_ctes_validos(&cte_info);

        let correlacoes = Correlacoes {
            cte_info,
            nfe_info,
            cte_nfes,
            nfe_ctes,
        };

        if arguments.exibir_correlacoes {
            print_cte_nfes(&correlacoes, arguments);
            print_nfe_ctes(&correlacoes, arguments);
        }

        self.add_info_nfes_to_cte(&correlacoes, arguments);
        self.add_info_ctes_to_nfe(&correlacoes, arguments);
    }

    /// Adicionar informações de NFes em CTe
    pub fn add_info_nfes_to_cte(&mut self, correlacoes: &Correlacoes, arguments: &Arguments) {
        self.ctes
            .par_iter_mut() // rayon parallel iterator
            .filter(|info| info.is_valid()) // remover cte cancelado
            .for_each(|info| {
                if let Some(cte) = &info.cte
                    && let Some(nfes) = correlacoes.cte_nfes.get(cte)
                {
                    info.nfes = nfes.to_vec_sorted();
                    info.ncm_descricao =
                        get_nfes_grouped_by_ncm_description(nfes, &correlacoes.nfe_info, arguments);
                    info.valor_total_nfes = get_total_value_nfes(nfes, &correlacoes.nfe_info);
                }
            });
    }

    /// Adicionar informações de CTes em NFe
    pub fn add_info_ctes_to_nfe(&mut self, correlacoes: &Correlacoes, arguments: &Arguments) {
        self.nfes
            .par_iter_mut() // rayon parallel iterator
            .filter(|info| info.is_valid()) // remover nfe cancelado
            .for_each(|info| {
                if let Some(nfe) = &info.nfe
                    && let Some(ctes) = correlacoes.nfe_ctes.get(nfe)
                {
                    info.ctes = ctes.to_vec_sorted();
                    info.tomadores =
                        get_ctes_grouped_by_payer(ctes, &correlacoes.cte_info, arguments);
                    info.valor_total_ctes = get_total_value_ctes(ctes, &correlacoes.cte_info);
                }
            });
    }

    // find . \( -iname "ctes-*.txt" \) | xargs wc -l
    // find . \( -iname "ctes-*.txt" -o -iname "nfes-*.txt" \) | xargs wc -l
    pub fn print_ctes(&self, filename: &str, size: usize) -> MyResult<()> {
        let chaves: BTreeSet<String> = self.ctes.get_chaves();
        print_chaves(&chaves, filename, size)?;
        Ok(())
    }

    // find . \( -iname "nfes-*.txt" \) | xargs wc -l
    // find . \( -iname "ctes-*.txt" -o -iname "nfes-*.txt" \) | xargs wc -l
    pub fn print_nfes(&self, filename: &str, size: usize) -> MyResult<()> {
        let chaves: BTreeSet<String> = self.nfes.get_chaves();
        print_chaves(&chaves, filename, size)?;
        Ok(())
    }
}

pub fn print_chaves(all_keys: &BTreeSet<String>, filename: &str, size: usize) -> MyResult<()> {
    all_keys
        .to_vec() // unique and ordered keys
        .chunks(size)
        .enumerate()
        .try_for_each(|(index, chaves)| -> MyResult<()> {
            let file_txt = format!("{filename}-{:05}.txt", index + 1);
            let mut output = File::create(file_txt)?;
            for chave in chaves {
                writeln!(output, "{chave}")?;
            }
            Ok(())
        })?;

    Ok(())
}

pub fn print_cte_nfes(correlacoes: &Correlacoes, arguments: &Arguments) {
    println!("cte_nfes:");
    correlacoes
        .cte_nfes
        .iter()
        .sorted_by_key(|tuple| tuple.0)
        .for_each(|(cte, nfes)| {
            println!(
                "{cte}: {:>2} {:?} ; Valor Total = {} ; NCMs: {:?}",
                nfes.len(),
                nfes.to_vec_sorted(),
                get_total_value_nfes(nfes, &correlacoes.nfe_info).unwrap_or_default(),
                get_nfes_grouped_by_ncm_description(nfes, &correlacoes.nfe_info, arguments),
            )
        });
    println!("cte_nfes.len(): {}\n", &correlacoes.cte_nfes.len());
}

pub fn print_nfe_ctes(correlacoes: &Correlacoes, arguments: &Arguments) {
    println!("nfe_ctes:");
    correlacoes
        .nfe_ctes
        .iter()
        .sorted_by_key(|tuple| tuple.0)
        .for_each(|(nfe, ctes)| {
            println!(
                "{nfe}: {:>2} {:?} ; Valor Total = {} ; Tomadores: {:?}",
                ctes.len(),
                ctes.to_vec_sorted(),
                get_total_value_ctes(ctes, &correlacoes.cte_info).unwrap_or_default(),
                get_ctes_grouped_by_payer(ctes, &correlacoes.cte_info, arguments),
            )
        });
    println!("nfe_ctes.len(): {}\n", &correlacoes.nfe_ctes.len());
}

pub trait HashMapExtension {
    fn count_recursively(&self) -> usize;
    fn expand_ctes(&mut self, verbose: bool);
    fn expand_nfes(&mut self, cte_ctes: &Self);
    fn get_nfe_ctes(&self) -> Self;
    fn filtrar_nfes_validos(&self, nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>) -> Self;
    fn filtrar_ctes_validos(&self, cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>) -> Self;
}

impl HashMapExtension for HashMap<String, HashSet<String>> {
    /// contador de chaves distintas
    fn count_recursively(&self) -> usize {
        self.len() + self.values().flatten().count()
    }

    /// `HashMap<chave_cte, HashSet<chave_cte>>`
    ///
    /// Usar HashMap temporário:
    ///
    /// to_add: `HashMap<String, HashSet<String>>`
    fn expand_ctes(&mut self, verbose: bool) {
        loop {
            let count_keys = self.count_recursively();
            let mut to_add: HashMap<String, HashSet<String>> = HashMap::new();

            for (cte_a, ctes) in self.iter() {
                for cte_complementar in ctes.iter().filter(|&cte| cte != cte_a) {
                    // reflexividade
                    // to_add.entry(cte_a.clone()).or_default().insert(cte_complementar.clone());
                    to_add
                        .entry(cte_complementar.clone())
                        .or_default()
                        .insert(cte_a.clone());

                    // transitividade: procurar ctes complementares do cte complementar
                    if let Some(complementares) = self.get(cte_complementar) {
                        let mut comp = complementares.clone();
                        comp.remove(cte_a);
                        to_add.entry(cte_a.clone()).or_default().extend(comp);
                    }
                }
            }

            to_add.into_iter().for_each(|(cte, ctes)| {
                self.entry(cte).or_default().extend(ctes);
            });

            if verbose {
                println!("self:");
                self.iter()
                    .for_each(|(k, chaves)| println!(" {k}: {chaves:?}"));
                println!("count_keys: {count_keys} -> {}\n", self.count_recursively());
            }

            // If no (key, value) is added to Self, exit the loop:
            if count_keys >= self.count_recursively() {
                break;
            }
        }
    }

    /// HashMap<chave_cte, HashSet<chave_nfe>>
    fn expand_nfes(&mut self, cte_ctes: &Self) {
        for (cte_a, nfes) in self.clone() {
            if let Some(ctes) = cte_ctes.get(&cte_a) {
                for cte_b in ctes {
                    self.entry(cte_b.clone())
                        // If there's no entry for the key cte, create a new Vec and return a mutable ref to it
                        .or_default()
                        // and insert the item onto the Vec
                        .extend(nfes.clone());
                }
            }
        }
    }

    /// Filtrar NFes válidos de cte_nfes
    fn filtrar_nfes_validos(&self, nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>) -> Self {
        self.par_iter()
            .map(|(cte, nfes)| {
                let docs_validos: HashSet<String> = nfes
                    .iter()
                    .filter(|&nfe| {
                        let chave_valida = KeyDoc::new(nfe, true);
                        nfe_info.contains_key(&chave_valida)
                    })
                    .cloned()
                    .collect();

                (cte.clone(), docs_validos)
            })
            .filter(|(_, docs_validos)| !docs_validos.is_empty())
            .collect()
    }

    /// Filtrar CTes válidos de nfe_ctes
    fn filtrar_ctes_validos(&self, cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>) -> Self {
        self.par_iter()
            .map(|(cte, ctes)| {
                let docs_validos: HashSet<String> = ctes
                    .iter()
                    .filter(|&cte| {
                        let chave_valida = KeyDoc::new(cte, true);
                        cte_info.contains_key(&chave_valida)
                    })
                    .cloned()
                    .collect();

                (cte.clone(), docs_validos)
            })
            .filter(|(_, docs_validos)| !docs_validos.is_empty())
            .collect()
    }

    /// Get nfe_ctes from cte_nfes
    ///
    /// Inverter a ordem: (cte, nfes) -> (nfe, ctes)
    ///
    /// HashMap<chave_nfe, HashSet<chave_cte>>
    fn get_nfe_ctes(&self) -> Self {
        let mut nfe_ctes: HashMap<String, HashSet<String>> = HashMap::new();

        for (cte, nfes) in self {
            for nfe in nfes {
                nfe_ctes
                    .entry(nfe.clone())
                    // If there's no entry for the key cte, create a new Vec and return a mutable ref to it
                    .or_default()
                    // and insert the item onto the Vec
                    .insert(cte.clone());
            }
        }

        nfe_ctes
    }
}

/// nfe: [cte1, cte2, cte3, ,,,, cteN]
fn get_total_value_ctes(
    ctes: &HashSet<String>,
    cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>,
) -> Option<f64> {
    ctes.par_iter()
        .map(|cte| {
            let chave_valida = KeyDoc::new(cte, true);
            cte_info // Soma dos Itens do CTe
                .get(&chave_valida)
                .and_then(|infos| {
                    infos
                        .iter()
                        .map(|info| info.valor_total)
                        .sum::<Option<f64>>()
                })
        })
        .sum::<Option<f64>>()
        .map(|sum| sum.round_float(2))
}

/// cte: [nfe1, nfe2, nfe3, ,,,, nfeN]
fn get_total_value_nfes(
    nfes: &HashSet<String>,
    nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>,
) -> Option<f64> {
    nfes.par_iter()
        .map(|nfe| {
            let chave_valida = KeyDoc::new(nfe, true);
            nfe_info // Soma dos Itens da NFe
                .get(&chave_valida)
                .and_then(|infos| infos.iter().map(|info| info.v_prod).sum::<Option<f64>>())
        })
        .sum::<Option<f64>>()
        .map(|sum| sum.round_float(2))
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
struct NFeKeys<'a> {
    ncm: &'a str,
    descricao: &'a str,
}

#[derive(Debug, Default, Clone, Serialize)]
struct NFeItens<'a> {
    keys: NFeKeys<'a>,
    valor: f64,
    pct: f64,
}

impl fmt::Display for NFeItens<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(ncm: {}, descricao: {}, valor: {}, pct: {})",
            self.keys.ncm, self.keys.descricao, self.valor, self.pct
        )
    }
}

/// Get nfes grouped by ncm and description
///
/// Informaçẽs de NFes agrupadas por NCM e Descrição dos Itens
fn get_nfes_grouped_by_ncm_description(
    nfes: &HashSet<String>,
    nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>,
    arguments: &Arguments,
) -> Vec<String> {
    // Vec<(NFeKeys, valor_do_item)>
    let tuples: Vec<(NFeKeys, f64)> = nfes
        .into_par_iter() // rayon: parallel iterator
        .flat_map(|nfe| {
            let chave_valida = KeyDoc::new(nfe, true);
            nfe_info.get(&chave_valida).map(|infos| {
                let keys: Vec<(NFeKeys, f64)> = infos
                    .iter()
                    .flat_map(|info| match (&info.ncm, &info.descricao, &info.v_prod) {
                        (Some(ncm), Some(descricao), Some(valor)) if *valor > 0.0 => {
                            Some((NFeKeys { ncm, descricao }, *valor))
                        }
                        _ => None,
                    })
                    .collect();

                keys
            })
        })
        .flatten()
        .collect();

    let map_reduce: HashMap<NFeKeys, f64> = tuples.group_by_key();
    let opt_valor_total: Option<f64> = get_total_value_nfes(nfes, nfe_info);

    match opt_valor_total {
        Some(valor_total) if valor_total > 0.0 => {
            let ncms_descricoes: Vec<String> = map_reduce
                .into_iter()
                .map(|(keys, valor)| {
                    let porcentagem = valor / valor_total * 100.0;
                    NFeItens {
                        keys,
                        valor: valor.round_float(2),
                        pct: porcentagem.round_float(2),
                    }
                })
                // sort in descending order of values
                .sorted_by_key(|itens| {
                    (
                        Reverse((itens.valor * 100.0) as i64), // 10 ^ 2 = 100
                        itens.keys.ncm,
                        itens.keys.descricao,
                    )
                })
                // Collect at most the N largest items
                .take(arguments.itens)
                .map(|item| item.to_string())
                //.flat_map(|item| serde_json::to_string(&item))
                .collect();

            ncms_descricoes
        }
        _ => Vec::new(),
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
struct CTeKeys {
    tomador_cnpj_cpf: String,
    tomador_atributo: String,
}

#[derive(Debug, Default, Clone, Serialize)]
struct CTeItens {
    keys: CTeKeys,
    valor: f64,
    pct: f64,
}

impl fmt::Display for CTeItens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(cnpj_cpf: {}, atributo: {}, valor: {}, pct: {})",
            self.keys.tomador_cnpj_cpf, self.keys.tomador_atributo, self.valor, self.pct
        )
    }
}

/// Get ctes grouped by payer
///
/// Informaçẽs de CTes agrupadas por Tomador/Pagador (CNPJ_CPF e Atributo)
fn get_ctes_grouped_by_payer(
    ctes: &HashSet<String>,
    cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>,
    arguments: &Arguments,
) -> Vec<String> {
    // Vec<(CTeKeys, valor_do_item)>
    let tuples: Vec<(CTeKeys, f64)> = ctes
        .into_par_iter() // rayon: parallel iterator
        .flat_map(|cte| {
            let chave_valida = KeyDoc::new(cte, true);
            cte_info.get(&chave_valida).map(|infos| {
                let keys: Vec<(CTeKeys, f64)> = infos
                    .iter()
                    .flat_map(|info| {
                        let opt_tomador_cnpj_cpf = info.get_cnpj_cpf_base_do_tomador();
                        let opt_tomador_atributo = TOMADOR_DO_SERVICO
                            .get(&info.tomador_codigo)
                            .map(|&t| t.to_string());
                        match (
                            opt_tomador_cnpj_cpf,
                            opt_tomador_atributo,
                            &info.valor_total,
                        ) {
                            (Some(tomador_cnpj_cpf), Some(tomador_atributo), Some(valor))
                                if *valor > 0.0 =>
                            {
                                let ctekeys = CTeKeys {
                                    tomador_cnpj_cpf,
                                    tomador_atributo,
                                };
                                Some((ctekeys, *valor))
                            }
                            _ => None,
                        }
                    })
                    .collect();

                keys
            })
        })
        .flatten()
        .collect();

    let map_reduce: HashMap<CTeKeys, f64> = tuples.group_by_key();
    let opt_valor_total: Option<f64> = get_total_value_ctes(ctes, cte_info);

    match opt_valor_total {
        Some(valor_total) if valor_total > 0.0 => {
            let tomadores: Vec<String> = map_reduce
                .into_iter()
                .map(|(keys, valor)| {
                    let porcentagem = valor / valor_total * 100.0;
                    CTeItens {
                        keys,
                        valor: valor.round_float(2),
                        pct: porcentagem.round_float(2),
                    }
                })
                // sort in descending order of values
                .sorted_by_key(|itens| {
                    (
                        Reverse((itens.valor * 100.0) as i64), // 10 ^ 2 = 100
                        itens.keys.tomador_cnpj_cpf.clone(),
                        itens.keys.tomador_atributo.clone(),
                    )
                })
                // Collect at most the N largest items
                .take(arguments.itens)
                .map(|item| item.to_string())
                //.flat_map(|item| serde_json::to_string(&item))
                .collect();

            tomadores
        }
        _ => Vec::new(),
    }
}

fn show_docs(doc_tipo: &str, docs: &[String]) {
    let size = docs.len();

    if size > 1 {
        println!("{size} {doc_tipo}s não encontrados:");
    } else {
        println!("{size} {doc_tipo} não encontrado:");
    }

    docs.iter().for_each(|doc| {
        println!("{doc}");
    });

    println!();
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
        None => usize::MAX,
    };

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

/// Get all information from files and show progress bar
pub fn get_all_info(
    xml_entries: &[DirEntry],
    multi_progressbar: &mut MultiProgressBar,
    arguments: &Arguments,
) -> Vec<Information> {
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
        ProcCancCte::struct_to_info(xml_path, arguments),
        ProcCancNfe::struct_to_info(xml_path, arguments),
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
        let struct_as_string = root.to_serde_struct(&Options::quick_xml_de());
        // save this result as a .rs file and use it to (de)serialize an XML document with serde
        println!("{}", struct_as_string.trim());
    }

    Ok(())
}

pub fn get_naive_date_from_yyyy_mm_dd<T>(date: &Option<T>) -> Option<NaiveDate>
where
    T: Deref<Target = str> + std::fmt::Debug + std::fmt::Display,
{
    // date: YYYY-MM-DD
    // "2020-09-04T10:48:18-03:00" -> "202009041048180300"
    let formatted_digits = date.as_ref().map(|d| d.remove_non_digits());

    if let Some(digits) = formatted_digits {
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
    } else {
        None
    }
}

pub fn get_naive_date_from_yyyymm<T>(date: &Option<T>) -> Option<NaiveDate>
where
    T: Deref<Target = str> + std::fmt::Display,
{
    // date: YYYYMM
    // ano_mes_caixa: "201901"
    let digits: Option<String> = date.as_ref().map(|d| d.remove_non_digits());

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

pub fn adicionar_eventos_nfe(
    nfes: &mut [InfoNfe],
    eventos_nfe: &[InfoNfeEvento],
    cancelamentos_nfe: &[InfoNfeCancel],
) {
    // Em NFe pode conter vários eventos independentes.
    // HashMap<chave_nfe, Vec<&InfoNfeEvento>>

    let eventos_agrupados_por_chaves: HashMap<String, Vec<&InfoNfeEvento>> =
        eventos_nfe.group_by_hashmap_key_vector_v2();

    let cancela_agrupados_por_chaves: HashMap<String, Vec<&InfoNfeCancel>> =
        cancelamentos_nfe.group_by_hashmap_key_vector_v2();

    nfes.par_iter_mut() // rayon parallel iterator
        .for_each(|info_nfe| {
            if let Some(nfe) = &info_nfe.nfe {
                if let Some(eventos) = eventos_agrupados_por_chaves.get(nfe) {
                    for evento in eventos {
                        if evento.cancelado {
                            info_nfe.cancelado = Some("Sim".to_string());
                            break;
                        }
                    }
                }

                if let Some(cancelamentos) = cancela_agrupados_por_chaves.get(nfe) {
                    for cancelamento in cancelamentos {
                        if cancelamento.cancelado {
                            info_nfe.cancelado = Some("Sim".to_string());
                            return;
                        }
                    }
                }
            }
        });
}

pub fn adicionar_eventos_cte(
    ctes: &mut [InfoCte],
    eventos_cte: &[InfoCteEvento],
    cancelamentos_cte: &[InfoCteCancel],
) {
    // Em CTe pode conter vários eventos independentes.
    // HashMap<chave_cte, Vec<&InfoCteEvento>>

    let eventos_agrupados_por_chaves: HashMap<String, Vec<&InfoCteEvento>> =
        eventos_cte.group_by_hashmap_key_vector_v2();

    let cancela_agrupados_por_chaves: HashMap<String, Vec<&InfoCteCancel>> =
        cancelamentos_cte.group_by_hashmap_key_vector_v2();

    ctes.par_iter_mut() // rayon parallel iterator
        .for_each(|info_cte| {
            if let Some(cte) = &info_cte.cte {
                if let Some(eventos) = eventos_agrupados_por_chaves.get(cte) {
                    for evento in eventos {
                        if evento.cancelado {
                            info_cte.cancelado = Some("Sim".to_string());
                        }

                        // append() or extend() or concat()
                        // https://rustjobs.dev/blog/vector-concatenation-in-rust/
                        info_cte.cte_complementar.extend(evento.get_ctes());
                    }
                }

                if let Some(cancelamentos) = cancela_agrupados_por_chaves.get(cte) {
                    for cancelamento in cancelamentos {
                        if cancelamento.cancelado {
                            info_cte.cancelado = Some("Sim".to_string());
                            return;
                        }
                    }
                }
            }
        });
}

// Not used!
pub fn deep_keys(
    reader: &mut Reader<BufReader<File>>,
    filter: bool,
) -> BTreeMap<String, Vec<String>> {
    // if filter, capture information only from these fields:
    const FIELDS: [&str; 3] = [
        "chCTe", "chave", "chNFe", // "refNFe",
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
            }
            Ok(Event::Text(node)) => {
                let value = node.decode().expect("Invalid UTF-8!").into_owned();
                //println!("value: {value}");

                if !filter || FIELDS.iter().any(|field| key.contains(field)) {
                    map.entry(key.clone()).or_default().push(value);
                }
            }
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
    /// `cargo test -- --show-output hashset_btreeset`
    fn hashset_btreeset() {
        // Obter lista com elementos únicos não necessariamente ordenados
        let hashset = HashSet::from([3, 1, 2, 3, 2]);
        println!("hashset: {hashset:?}");

        // Obter lista com elementos únicos ordenados
        let btreeset = BTreeSet::from([3, 1, 2, 3, 2]);
        println!("btreeset: {btreeset:?}");

        assert_eq!(hashset, HashSet::from([1, 2, 3]));
        assert_eq!(btreeset, BTreeSet::from([1, 2, 3]));
    }

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

    /**
        cte_a: ["cte_1", "cte_2", "cte_3"]

        cte_b: ["cte_3", "cte_4"]

        cte_c: ["cte_5", "cte_6"]

        cte_3 ∈ cte_a && cte_3 ∈ cte_b

        Portanto:
        cte_a e cte_b são correlacionados pois contém "cte_3" em comum.

        `cargo test -- --show-output correlacionar_ctes`
    */
    #[test]
    fn correlacionar_ctes() -> MyResult<()> {
        // Start
        let array_a = ["cte_1", "cte_2", "cte_3"];
        let array_b = ["cte_3", "cte_4"];
        let array_c = ["cte_5", "cte_6"];
        let array_d = ["cte_7"];

        let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "cte_a".to_string(),
                HashSet::from(array_a.map(String::from)),
            ),
            (
                "cte_b".to_string(),
                HashSet::from(array_b.map(String::from)),
            ),
            (
                "cte_c".to_string(),
                HashSet::from(array_c.map(String::from)),
            ),
            (
                "cte_d".to_string(),
                HashSet::from(array_d.map(String::from)),
            ),
        ]);

        println!("cte_ctes: [Start]");
        cte_ctes
            .iter()
            .for_each(|(k, chaves)| println!("{k}: {chaves:?}"));
        println!("cte_ctes.len(): {}\n", cte_ctes.len());

        cte_ctes.expand_ctes(true);

        println!("cte_ctes: [Final]");
        cte_ctes
            .iter()
            .for_each(|(k, chaves)| println!("{k}: {chaves:?}"));
        println!("cte_ctes.len(): {}\n", cte_ctes.len());

        // Final
        let array_1 = ["cte_2", "cte_3", "cte_4", "cte_a", "cte_b"];
        let array_2 = ["cte_1", "cte_3", "cte_4", "cte_a", "cte_b"];
        let array_3 = ["cte_1", "cte_2", "cte_4", "cte_a", "cte_b"];
        let array_4 = ["cte_1", "cte_2", "cte_3", "cte_a", "cte_b"];
        let array_5 = ["cte_6", "cte_c"];
        let array_6 = ["cte_5", "cte_c"];
        let array_7 = ["cte_d"];
        let array_a = ["cte_1", "cte_2", "cte_3", "cte_4", "cte_b"];
        let array_b = ["cte_1", "cte_2", "cte_3", "cte_4", "cte_a"];
        let array_c = ["cte_5", "cte_6"];
        let array_d = ["cte_7"];

        assert_eq!(
            cte_ctes,
            HashMap::from([
                (
                    "cte_1".to_string(),
                    HashSet::from(array_1.map(String::from))
                ),
                (
                    "cte_2".to_string(),
                    HashSet::from(array_2.map(String::from))
                ),
                (
                    "cte_3".to_string(),
                    HashSet::from(array_3.map(String::from))
                ),
                (
                    "cte_4".to_string(),
                    HashSet::from(array_4.map(String::from))
                ),
                (
                    "cte_5".to_string(),
                    HashSet::from(array_5.map(String::from))
                ),
                (
                    "cte_6".to_string(),
                    HashSet::from(array_6.map(String::from))
                ),
                (
                    "cte_7".to_string(),
                    HashSet::from(array_7.map(String::from))
                ),
                (
                    "cte_a".to_string(),
                    HashSet::from(array_a.map(String::from))
                ),
                (
                    "cte_b".to_string(),
                    HashSet::from(array_b.map(String::from))
                ),
                (
                    "cte_c".to_string(),
                    HashSet::from(array_c.map(String::from))
                ),
                (
                    "cte_d".to_string(),
                    HashSet::from(array_d.map(String::from))
                ),
            ])
        );

        Ok(())
    }

    /// `cargo test -- --show-output expand_cte_nfes`
    #[test]
    fn expand_cte_nfes() -> MyResult<()> {
        // Start
        let array_a = ["cte_1", "cte_2", "cte_3"];
        let array_b = ["cte_3", "cte_4"];
        let array_c = ["cte_5", "cte_6"];
        let array_d = ["cte_7"];

        let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "cte_a".to_string(),
                HashSet::from(array_a.map(String::from)),
            ),
            (
                "cte_b".to_string(),
                HashSet::from(array_b.map(String::from)),
            ),
            (
                "cte_c".to_string(),
                HashSet::from(array_c.map(String::from)),
            ),
            (
                "cte_d".to_string(),
                HashSet::from(array_d.map(String::from)),
            ),
        ]);

        let array_1 = ["nfe_1", "nfe_2"];
        let array_2 = ["nfe_2", "nfe_3"];
        let array_3 = ["nfe_4"];

        let mut cte_nfes: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "cte_1".to_string(),
                HashSet::from(array_1.map(String::from)),
            ),
            (
                "cte_3".to_string(),
                HashSet::from(array_2.map(String::from)),
            ),
            (
                "cte_8".to_string(),
                HashSet::from(array_3.map(String::from)),
            ),
        ]);

        cte_ctes.expand_ctes(false);

        cte_nfes.expand_nfes(&cte_ctes);

        println!("cte_nfes:");
        cte_nfes
            .iter()
            .for_each(|(cte, nfes)| println!("{cte}: {:>2} {nfes:?}", nfes.len()));
        println!("cte_nfes.len(): {}\n", cte_nfes.len());

        let array_x = ["nfe_1", "nfe_2", "nfe_3"];
        let array_y = ["nfe_4"];

        // Final
        assert_eq!(
            cte_nfes,
            HashMap::from([
                (
                    "cte_1".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_2".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_3".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_4".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_8".to_string(),
                    HashSet::from(array_y.map(String::from))
                ),
                (
                    "cte_a".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_b".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
            ])
        );

        Ok(())
    }
}
