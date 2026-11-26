//! # Coleção Unificada de Documentos Fiscais
//!
//! Gerencia a consolidação global de notas fiscais, conhecimentos de transporte e e-Financeira.

use claudiofsr_lib::HashSetExtension;
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
};

use crate::{
    Arguments, CachedCompNfeSummary, CteComponents, Information, KeyDoc, SanitizeSliceExt,
    UniqueIdentification, XmlParserResult, get_ctes_grouped_by_payer,
    get_nfes_grouped_by_ncm_description, get_total_value_ctes, get_total_value_nfes,
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
            s.spawn(|_| self.efinanceiras = self.efinanceiras.get_unique_id());
        });

        rayon::join(
            || {
                self.ctes
                    .par_iter_mut()
                    .for_each(|info| info.get_unique_elements());
            },
            || {
                self.nfes
                    .par_iter_mut()
                    .for_each(|info| info.get_unique_elements());
            },
        );
    }

    /// Ordena os documentos fiscais em paralelo com base em critérios contábeis estáveis.
    pub fn sort(&mut self) {
        rayon::scope(|s| {
            s.spawn(|_| self.nfes.par_sort_by(cmp_info_nfe));
            s.spawn(|_| self.ctes.par_sort_by(cmp_info_cte));
            s.spawn(|_| self.efinanceiras.par_sort_by(cmp_info_efin));
        });
    }

    /// Remove quebras de linha e tabulações em paralelo (comportamento padrão).
    pub fn strip_line_breaks(&mut self) {
        rayon::scope(|s| {
            s.spawn(|_| self.ctes.strip_line_breaks_par());
            s.spawn(|_| self.nfes.strip_line_breaks_par());
            s.spawn(|_| self.efinanceiras.strip_line_breaks_par());
        });
    }

    /// Colapsa múltiplos espaços consecutivos para um único espaço em paralelo.
    pub fn clean_spaces(&mut self) {
        rayon::scope(|s| {
            s.spawn(|_| self.ctes.clean_spaces_par());
            s.spawn(|_| self.nfes.clean_spaces_par());
            s.spawn(|_| self.efinanceiras.clean_spaces_par());
        });
    }

    /// Libera da RAM os buffers de eventos e cancelamentos após a vinculação.
    pub fn clear_events(&mut self) {
        self.eventos_cte = Vec::new();
        self.eventos_nfe = Vec::new();
        self.cancel_cte = Vec::new();
        self.cancel_nfe = Vec::new();
    }

    /// Agrupa referências `&T` em paralelo sem clonar as structs de domínio.
    fn groupby_info_ref<T, F>(items: &[T], key_fn: F) -> BTreeMap<KeyDoc, Vec<&T>>
    where
        T: Sync,
        F: Fn(&T) -> Option<KeyDoc> + Sync,
    {
        items
            .par_iter()
            .filter_map(|info| key_fn(info).map(|key| (key, info)))
            .fold(
                BTreeMap::new,
                |mut acc: BTreeMap<KeyDoc, Vec<&T>>, (k, v)| {
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

    /// Coleta chaves referenciadas pelos CT-es que não constam (nem válidas nem canceladas) no índice informado.
    fn get_chaves_nao_encontradas<V: Sync, I, F>(
        &self,
        docs_info: &BTreeMap<KeyDoc, V>,
        extractor: F,
    ) -> HashSet<String>
    where
        I: IntoIterator<Item = String>,
        F: Fn(&InfoCte) -> I + Sync,
    {
        self.ctes
            .par_iter()
            .flat_map(|info| {
                extractor(info)
                    .into_iter()
                    .filter(|chave| {
                        !docs_info.contains_key(&KeyDoc::new(chave, true))
                            && !docs_info.contains_key(&KeyDoc::new(chave, false))
                    })
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    /// Mapeia cada CT-e para o conjunto de chaves correlacionadas válidas presentes em `docs_info`.
    fn groupby_cte_correlated_valid<V: Sync, I, F>(
        &self,
        docs_info: &BTreeMap<KeyDoc, V>,
        extractor: F,
    ) -> HashMap<String, HashSet<String>>
    where
        I: IntoIterator<Item = String>,
        F: Fn(&InfoCte) -> I + Sync,
    {
        self.ctes
            .par_iter()
            .filter_map(|info| {
                let cte_key = info.cte.as_ref()?;
                let validos: HashSet<String> = extractor(info)
                    .into_iter()
                    .filter(|doc| doc != cte_key && docs_info.contains_key(&KeyDoc::new(doc, true)))
                    .collect();
                Some((cte_key.clone(), validos))
            })
            .collect()
    }

    /// Obtain correlations between CTe and NFe (zero-clone reference indexing)
    ///
    /// CTe contém 1 item
    ///
    /// NFe contém vários itens
    pub fn get_correlations(&mut self, arguments: &Arguments) {
        let (comps, comp_nfes, solo_nfes, nfe_ctes, nfe_nfes) = {
            // 1. Primeira etapa: Agrupamento inicial por referências (sem clonar structs)
            let (cte_info, nfe_info) = rayon::join(
                || {
                    Self::groupby_info_ref(&self.ctes, |info| {
                        info.cte
                            .as_ref()
                            .map(|cte| KeyDoc::new(cte, info.is_valid()))
                    })
                },
                || {
                    Self::groupby_info_ref(&self.nfes, |info| {
                        info.nfe
                            .as_ref()
                            .map(|nfe| KeyDoc::new(nfe, info.is_valid()))
                    })
                },
            );

            // 2. Segunda etapa: Cruzamento de dados em paralelo
            let mut ctes_nao_encontrados: HashSet<String> = HashSet::new();
            let mut nfes_nao_encontrados: HashSet<String> = HashSet::new();

            let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::new();
            let mut cte_nfes: HashMap<String, HashSet<String>> = HashMap::new();

            rayon::scope(|s| {
                s.spawn(|_| {
                    ctes_nao_encontrados =
                        self.get_chaves_nao_encontradas(&cte_info, InfoCte::get_correlated_ctes);
                });
                s.spawn(|_| {
                    nfes_nao_encontrados =
                        self.get_chaves_nao_encontradas(&nfe_info, InfoCte::get_correlated_nfes);
                });
                s.spawn(|_| {
                    cte_ctes =
                        self.groupby_cte_correlated_valid(&cte_info, InfoCte::get_correlated_ctes);
                });
                s.spawn(|_| {
                    cte_nfes =
                        self.groupby_cte_correlated_valid(&nfe_info, InfoCte::get_correlated_nfes);
                });
            });

            // 3. Processamento Sequencial de Lógica de Negócio
            if arguments.exibir_chaves_nao_encontradas && arguments.filtro.is_none() {
                show_docs("CTe", &ctes_nao_encontrados.to_vec_sorted());
                show_docs("NFe", &nfes_nao_encontrados.to_vec_sorted());
            }

            // DSU compacto O(V) de memória (evita cliques completos O(k²))
            let comps = CteComponents::from_adj(cte_ctes.iter().map(|(k, v)| (k, v.iter())));
            drop(cte_ctes);

            let mut comp_nfes: Vec<HashSet<String>> = vec![HashSet::new(); comps.components.len()];
            let mut solo_nfes: HashMap<String, HashSet<String>> = HashMap::new();
            for (cte, mut nfes) in cte_nfes {
                nfes.retain(|doc| nfe_info.contains_key(&KeyDoc::new(doc, true)));
                if nfes.is_empty() {
                    continue;
                }
                if let Some(&comp_id) = comps.cte_to_comp.get(&cte) {
                    comp_nfes[comp_id].extend(nfes);
                } else {
                    solo_nfes.insert(cte, nfes);
                }
            }

            let mut nfe_ctes: HashMap<String, HashSet<String>> = HashMap::new();
            for (comp_id, nfes) in comp_nfes.iter().enumerate() {
                if nfes.is_empty() {
                    continue;
                }
                let members = &comps.components[comp_id];
                for nfe in nfes {
                    nfe_ctes
                        .entry(nfe.clone())
                        .or_default()
                        .extend(members.iter().cloned());
                }
            }
            for (cte, nfes) in &solo_nfes {
                for nfe in nfes {
                    nfe_ctes.entry(nfe.clone()).or_default().insert(cte.clone());
                }
            }

            let mut nfe_nfes: HashMap<String, HashSet<String>> = HashMap::new();
            for info in self.nfes.iter().filter(|i| i.is_valid()) {
                let Some(nfe_key) = info.nfe.as_ref() else {
                    continue;
                };
                for cte_ref in &info.ctes {
                    if cte_ref != nfe_key && cte_info.contains_key(&KeyDoc::new(cte_ref, true)) {
                        nfe_ctes
                            .entry(nfe_key.clone())
                            .or_default()
                            .insert(cte_ref.clone());
                    }
                }
                for other_nfe in &info.nfes_relacionadas {
                    if other_nfe != nfe_key && nfe_info.contains_key(&KeyDoc::new(other_nfe, true))
                    {
                        nfe_nfes
                            .entry(nfe_key.clone())
                            .or_default()
                            .insert(other_nfe.clone());
                        nfe_nfes
                            .entry(other_nfe.clone())
                            .or_default()
                            .insert(nfe_key.clone());
                    }
                }
            }

            if arguments.exibir_correlacoes {
                let mut cte_nfes_view: HashMap<String, HashSet<String>> = solo_nfes.clone();
                for (comp_id, nfes) in comp_nfes.iter().enumerate() {
                    if !nfes.is_empty() {
                        for m in &comps.components[comp_id] {
                            cte_nfes_view.insert(m.clone(), nfes.clone());
                        }
                    }
                }
                print_cte_nfes_map(&cte_nfes_view, &nfe_info, arguments);
                print_nfe_ctes_map(&nfe_ctes, &cte_info, arguments);
            }

            (comps, comp_nfes, solo_nfes, nfe_ctes, nfe_nfes)
        };

        // 4. Atualização mútua em paralelo (calculando NCM/Total apenas 1x por componente DSU)
        {
            let nfe_info = Self::groupby_info_ref(&self.nfes, |info| {
                info.nfe
                    .as_ref()
                    .map(|nfe| KeyDoc::new(nfe, info.is_valid()))
            });
            Self::apply_nfes_to_ctes(
                &mut self.ctes,
                &comps,
                &comp_nfes,
                &solo_nfes,
                &nfe_info,
                arguments,
            );
        }
        {
            let cte_info = Self::groupby_info_ref(&self.ctes, |info| {
                info.cte
                    .as_ref()
                    .map(|cte| KeyDoc::new(cte, info.is_valid()))
            });
            Self::apply_ctes_to_nfes(&mut self.nfes, &nfe_ctes, &nfe_nfes, &cte_info, arguments);
        }
    }

    fn apply_nfes_to_ctes<T: Borrow<InfoNfe> + Sync>(
        ctes: &mut [InfoCte],
        comps: &CteComponents,
        comp_nfes: &[HashSet<String>],
        solo_nfes: &HashMap<String, HashSet<String>>,
        nfe_info: &BTreeMap<KeyDoc, Vec<T>>,
        arguments: &Arguments,
    ) {
        let comp_cached: Vec<CachedCompNfeSummary> = comp_nfes
            .par_iter()
            .map(|nfes| {
                if nfes.is_empty() {
                    None
                } else {
                    Some((
                        nfes.to_vec_sorted(),
                        get_nfes_grouped_by_ncm_description(nfes, nfe_info, arguments),
                        get_total_value_nfes(nfes, nfe_info),
                    ))
                }
            })
            .collect();

        ctes.par_iter_mut()
            .filter(|info| info.is_valid())
            .for_each(|info| {
                if let Some(cte) = &info.cte {
                    if let Some(&comp_id) = comps.cte_to_comp.get(cte) {
                        let anteriores: HashSet<&str> =
                            info.cte_anteriores.iter().map(String::as_str).collect();
                        let mut merged: BTreeSet<String> =
                            info.cte_complementar.iter().cloned().collect();
                        for c in &comps.components[comp_id] {
                            if c != cte && !anteriores.contains(c.as_str()) {
                                merged.insert(c.clone());
                            }
                        }
                        info.cte_complementar = merged.into_iter().collect();

                        if let Some((nfes_sorted, ncm_desc, total_val)) = &comp_cached[comp_id] {
                            info.nfes = nfes_sorted.clone();
                            info.ncm_descricao = ncm_desc.clone();
                            info.valor_total_nfes = *total_val;
                        }
                    } else if let Some(nfes) = solo_nfes.get(cte) {
                        info.nfes = nfes.to_vec_sorted();
                        info.ncm_descricao =
                            get_nfes_grouped_by_ncm_description(nfes, nfe_info, arguments);
                        info.valor_total_nfes = get_total_value_nfes(nfes, nfe_info);
                    }
                }
            });
    }
    fn apply_ctes_to_nfes<T: Borrow<InfoCte> + Sync>(
        nfes: &mut [InfoNfe],
        nfe_ctes: &HashMap<String, HashSet<String>>,
        nfe_nfes: &HashMap<String, HashSet<String>>,
        cte_info: &BTreeMap<KeyDoc, Vec<T>>,
        arguments: &Arguments,
    ) {
        nfes.par_iter_mut()
            .filter(|info| info.is_valid())
            .for_each(|info| {
                if let Some(nfe) = &info.nfe {
                    if let Some(ctes) = nfe_ctes.get(nfe) {
                        let mut merged: BTreeSet<String> = info.ctes.iter().cloned().collect();
                        merged.extend(ctes.iter().cloned());
                        info.ctes = merged.into_iter().collect();
                        info.tomadores = get_ctes_grouped_by_payer(ctes, cte_info, arguments);
                        info.valor_total_ctes = get_total_value_ctes(ctes, cte_info);
                    }
                    if let Some(rel_nfes) = nfe_nfes.get(nfe) {
                        let mut merged: BTreeSet<String> =
                            info.nfes_relacionadas.iter().cloned().collect();
                        merged.extend(rel_nfes.iter().cloned());
                        info.nfes_relacionadas = merged.into_iter().collect();
                    }
                }
            });
    }

    /// Salva as chaves de CT-e apuradas em arquivos particionados de texto.
    pub fn print_ctes(
        &self,
        filename: &str,
        size: usize,
    ) -> XmlParserResult<Vec<(PathBuf, usize)>> {
        let chaves = self.ctes.get_chaves();
        print_chaves(&chaves, filename, size)
    }

    /// Salva as chaves de NF-e apuradas em arquivos particionados de texto.
    pub fn print_nfes(
        &self,
        filename: &str,
        size: usize,
    ) -> XmlParserResult<Vec<(PathBuf, usize)>> {
        let chaves = self.nfes.get_chaves();
        print_chaves(&chaves, filename, size)
    }
}

/// Comparador contábil canônico para [`InfoNfe`] (zero-allocation).
pub fn cmp_info_nfe(a: &InfoNfe, b: &InfoNfe) -> Ordering {
    a.emitente_cnpj
        .cmp(&b.emitente_cnpj)
        .then_with(|| a.emitente_cpf.cmp(&b.emitente_cpf))
        .then_with(|| a.data_emissao.cmp(&b.data_emissao))
        .then_with(|| a.nfe.cmp(&b.nfe))
        .then_with(|| a.n_item.cmp(&b.n_item))
}

/// Comparador contábil canônico para [`InfoCte`] (zero-allocation).
pub fn cmp_info_cte(a: &InfoCte, b: &InfoCte) -> Ordering {
    a.emitente_cnpj
        .cmp(&b.emitente_cnpj)
        .then_with(|| a.emitente_cpf.cmp(&b.emitente_cpf))
        .then_with(|| a.remetente_cnpj.cmp(&b.remetente_cnpj))
        .then_with(|| a.remetente_cpf.cmp(&b.remetente_cpf))
        .then_with(|| a.destinatario_cnpj.cmp(&b.destinatario_cnpj))
        .then_with(|| a.destinatario_cpf.cmp(&b.destinatario_cpf))
        .then_with(|| a.expedidor_cnpj.cmp(&b.expedidor_cnpj))
        .then_with(|| a.expedidor_cpf.cmp(&b.expedidor_cpf))
        .then_with(|| a.recebedor_cnpj.cmp(&b.recebedor_cnpj))
        .then_with(|| a.recebedor_cpf.cmp(&b.recebedor_cpf))
        .then_with(|| a.data_emissao.cmp(&b.data_emissao))
        .then_with(|| a.cte.cmp(&b.cte))
}

/// Comparador contábil canônico para [`InfoEFinanceira`] (zero-allocation).
pub fn cmp_info_efin(a: &InfoEFinanceira, b: &InfoEFinanceira) -> Ordering {
    a.cnpj_do_declarante
        .cmp(&b.cnpj_do_declarante)
        .then_with(|| a.ni_do_declarado.cmp(&b.ni_do_declarado))
        .then_with(|| a.ano_mes_caixa.cmp(&b.ano_mes_caixa))
        .then_with(|| a.num_conta.cmp(&b.num_conta))
        .then_with(|| a.id.cmp(&b.id))
}

/// Salva um conjunto de chaves únicas ordenadas em blocos de até `size` linhas,
/// retornando os caminhos gerados e a quantidade de chaves gravadas em cada arquivo.
pub fn print_chaves(
    all_keys: &BTreeSet<String>,
    filename: &str,
    size: usize,
) -> XmlParserResult<Vec<(PathBuf, usize)>> {
    let mut generated = Vec::new();
    for (index, chaves) in all_keys.iter().chunks(size).into_iter().enumerate() {
        let file_txt = PathBuf::from(format!("{filename}-{:05}.txt", index + 1));
        let mut output = File::create(&file_txt)?;
        let mut count = 0usize;
        for chave in chaves {
            writeln!(output, "{chave}")?;
            count += 1;
        }
        generated.push((file_txt, count));
    }
    Ok(generated)
}

fn print_cte_nfes_map<T: Borrow<InfoNfe> + Sync>(
    cte_nfes: &HashMap<String, HashSet<String>>,
    nfe_info: &BTreeMap<KeyDoc, Vec<T>>,
    arguments: &Arguments,
) {
    println!("cte_nfes:");
    cte_nfes
        .iter()
        .sorted_by_key(|tuple| tuple.0)
        .for_each(|(cte, nfes)| {
            println!(
                "{cte}: {:>2} {:?} ; Valor Total = {} ; NCMs: {:?}",
                nfes.len(),
                nfes.to_vec_sorted(),
                get_total_value_nfes(nfes, nfe_info).unwrap_or_default(),
                get_nfes_grouped_by_ncm_description(nfes, nfe_info, arguments),
            )
        });
    println!("cte_nfes.len(): {}\n", cte_nfes.len());
}

fn print_nfe_ctes_map<T: Borrow<InfoCte> + Sync>(
    nfe_ctes: &HashMap<String, HashSet<String>>,
    cte_info: &BTreeMap<KeyDoc, Vec<T>>,
    arguments: &Arguments,
) {
    println!("nfe_ctes:");
    nfe_ctes
        .iter()
        .sorted_by_key(|tuple| tuple.0)
        .for_each(|(nfe, ctes)| {
            println!(
                "{nfe}: {:>2} {:?} ; Valor Total = {} ; Tomadores: {:?}",
                ctes.len(),
                ctes.to_vec_sorted(),
                get_total_value_ctes(ctes, cte_info).unwrap_or_default(),
                get_ctes_grouped_by_payer(ctes, cte_info, arguments),
            )
        });
    println!("nfe_ctes.len(): {}\n", nfe_ctes.len());
}

pub fn show_docs(doc_tipo: &str, docs: &[String]) {
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
