//! # Coleção Unificada de Documentos Fiscais
//!
//! Gerencia a consolidação global de notas fiscais, conhecimentos de transporte e e-Financeira.

use claudiofsr_lib::HashSetExtension;
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::File,
    io::Write,
};

use crate::{
    Arguments, Correlacoes, GraphExtension, Information, KeyDoc, UniqueIdentification,
    XmlParserResult, get_ctes_grouped_by_payer, get_nfes_grouped_by_ncm_description,
    get_total_value_ctes, get_total_value_nfes,
    xml_structs::{
        cancelamento_cte::InfoCteCancel, cancelamento_nfe::InfoNfeCancel, cte::InfoCte,
        cte_evento::InfoCteEvento, efinanceira::InfoEFinanceira, nfe::InfoNfe,
        nfe_evento::InfoNfeEvento,
    },
};

pub trait KeysExtension {
    fn get_chaves(&self) -> BTreeSet<String>;
}

/// Repositório de documentos fiscais eletrônicos apurados durante a execução.
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Retorna a quantidade de tipos distintos de documentos populados no lote.
    pub fn total(&self) -> usize {
        usize::from(!self.ctes.is_empty())
            + usize::from(!self.nfes.is_empty())
            + usize::from(!self.efinanceiras.is_empty())
    }

    /// Adiciona a variante de informação ao repositório correspondente.
    pub fn add_information(&mut self, info: Information) {
        match info {
            Information::Cte(c) => self.ctes.push(*c),
            Information::Nfe(n) => self.nfes.extend(n),
            Information::EventoCte(e) => self.eventos_cte.push(*e),
            Information::EventoNfe(e) => self.eventos_nfe.push(*e),
            Information::CancelamentoCte(c) => self.cancel_cte.push(*c),
            Information::CancelamentoNfe(c) => self.cancel_nfe.push(*c),
            Information::EFinanceira(ef) => self.efinanceiras.extend(ef),
            Information::None => (),
        }
    }

    /// Remove duplicatas em paralelo.
    pub fn unique(&mut self) {
        rayon::scope(|s| {
            s.spawn(|_| self.nfes = self.nfes.get_unique_id());
            s.spawn(|_| self.ctes = self.ctes.get_unique_id());
        });

        self.ctes
            .par_iter_mut()
            .for_each(|info| info.get_unique_elements());
    }

    /// Ordena os documentos fiscais em paralelo com base em critérios contábeis estáveis.
    pub fn sort(&mut self) {
        rayon::join(
            || {
                self.nfes.par_sort_by_key(|info_nfe| {
                    (
                        info_nfe.emitente_cnpj.clone(),
                        info_nfe.emitente_cpf.clone(),
                        info_nfe.data_emissao,
                        info_nfe.nfe.clone(),
                        info_nfe.n_item,
                    )
                });
            },
            || {
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
            },
        );
    }

    /// Relacionar KeyDoc com InfoCte
    ///
    /// keydoc.chave = cte
    fn groupby_cte_info(&self) -> BTreeMap<KeyDoc, Vec<InfoCte>> {
        self.ctes
            .par_iter()
            .flat_map(|info| {
                info.cte
                    .as_ref()
                    .map(|cte| (KeyDoc::new(cte, info.is_valid()), info.clone()))
            })
            .fold(
                BTreeMap::new,
                |mut acc: BTreeMap<KeyDoc, Vec<InfoCte>>, (k, v)| {
                    acc.entry(k).or_default().push(v);
                    acc
                },
            )
            .reduce(BTreeMap::new, |mut acc, map| {
                for (k, v) in map {
                    acc.entry(k).or_default().extend(v);
                }
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
            .par_iter()
            .flat_map(|info| {
                info.nfe.as_ref().map(|nfe| {
                    // (key, value)
                    (KeyDoc::new(nfe, info.is_valid()), info.clone())
                })
            })
            .fold(
                BTreeMap::new,
                |mut acc: BTreeMap<KeyDoc, Vec<InfoNfe>>, (k, v)| {
                    acc.entry(k).or_default().push(v);
                    acc
                },
            )
            .reduce(BTreeMap::new, |mut acc, map| {
                for (k, v) in map {
                    acc.entry(k).or_default().extend(v);
                }
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
                        .filter(|cte_complementar| {
                            let chave_valida = KeyDoc::new(cte_complementar, true);
                            cte_complementar != cte_a && cte_info.contains_key(&chave_valida)
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
    ///
    /// CTe contém 1 item
    ///
    /// NFe contém vários itens
    pub fn get_correlations(&mut self, arguments: &Arguments) {
        // 1. Primeira etapa: Agrupamento inicial
        let (cte_info, nfe_info) =
            rayon::join(|| self.groupby_cte_info(), || self.groupby_nfe_info());

        // 2. Segunda etapa: Cruzamento de dados
        // Aqui usamos rayon::scope pois temos 4 tarefas independentes.
        let mut ctes_nao_encontrados: HashSet<String> = HashSet::new();
        let mut nfes_nao_encontrados: HashSet<String> = HashSet::new();

        let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::new();
        let mut cte_nfes: HashMap<String, HashSet<String>> = HashMap::new();

        rayon::scope(|s| {
            s.spawn(|_| ctes_nao_encontrados = self.get_ctes_nao_encontrados(&cte_info));
            s.spawn(|_| nfes_nao_encontrados = self.get_nfes_nao_encontrados(&nfe_info));
            s.spawn(|_| cte_ctes = self.groupby_cte_ctes(&cte_info));
            s.spawn(|_| cte_nfes = self.groupby_cte_nfes(&nfe_info));
        });

        // 3. Processamento Sequencial de Lógica de Negócio
        if arguments.exibir_chaves_nao_encontradas {
            show_docs("CTe", &ctes_nao_encontrados.to_vec_sorted());
            show_docs("NFe", &nfes_nao_encontrados.to_vec_sorted());
        }

        // Estas expansões são iterativas e dependentes
        cte_ctes.expand_ctes(false);
        cte_nfes.expand_nfes(&cte_ctes);

        // Filtros e Inversão de Mapas
        let cte_nfes = cte_nfes.filtrar_nfes_validos(&nfe_info);
        let nfe_ctes = cte_nfes.get_nfe_ctes().filtrar_ctes_validos(&cte_info);

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

    /// Salva as chaves de CT-e apuradas em arquivos particionados de texto.
    pub fn print_ctes(&self, filename: &str, size: usize) -> XmlParserResult<()> {
        let chaves = self.ctes.get_chaves();
        print_chaves(&chaves, filename, size)
    }

    /// Salva as chaves de NF-e apuradas em arquivos particionados de texto.
    pub fn print_nfes(&self, filename: &str, size: usize) -> XmlParserResult<()> {
        let chaves = self.nfes.get_chaves();
        print_chaves(&chaves, filename, size)
    }
}

/// Salva um conjunto de chaves únicas ordenadas em blocos de até `size` linhas.
pub fn print_chaves(
    all_keys: &BTreeSet<String>,
    filename: &str,
    size: usize,
) -> XmlParserResult<()> {
    for (index, chaves) in all_keys.iter().chunks(size).into_iter().enumerate() {
        let file_txt = format!("{filename}-{:05}.txt", index + 1);
        let mut output = File::create(file_txt)?;
        for chave in chaves {
            writeln!(output, "{chave}")?;
        }
    }
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
    println!("cte_nfes.len(): {}\n", correlacoes.cte_nfes.len());
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
    println!("nfe_ctes.len(): {}\n", correlacoes.nfe_ctes.len());
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
