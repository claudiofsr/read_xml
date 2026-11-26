//! # Agregações Fiscais e Agrupamentos Analíticos
//!
//! Fornece funções de soma e consolidação para categorizar itens de NF-e por
//! NCM/Descrição e agrupar tomadores de CT-e por relevância de valores.

use claudiofsr_lib::RoundFloat;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;
use std::{
    borrow::Borrow,
    cmp::Reverse,
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
};

use crate::{
    Arguments, GroupBy, KeyDoc,
    xml_structs::{agente::TOMADOR_DO_SERVICO, cte::InfoCte, nfe::InfoNfe},
};

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
            "(ncm: {}, descricao: {}, valor: {:.2}, pct: {:.2}%)",
            self.keys.ncm, self.keys.descricao, self.valor, self.pct
        )
    }
}

/// Agrupa e ordena os itens de NF-e correlacionados a um CT-e por NCM
/// e descrição em ordem decrescente de valor.
pub fn get_nfes_grouped_by_ncm_description<T: Borrow<InfoNfe> + Sync>(
    nfes: &HashSet<String>,
    nfe_info: &BTreeMap<KeyDoc, Vec<T>>,
    arguments: &Arguments,
) -> Vec<String> {
    let tuples: Vec<(NFeKeys, f64)> = nfes
        .par_iter() // rayon: parallel iterator
        .flat_map(|nfe| {
            let chave_valida = KeyDoc::new(nfe, true);
            nfe_info.get(&chave_valida).map(|infos| {
                infos
                    .iter()
                    .flat_map(|item| {
                        let info = item.borrow();
                        match (&info.ncm, &info.descricao, &info.v_prod) {
                            (Some(ncm), Some(descricao), Some(valor)) if *valor > 0.0 => {
                                Some((NFeKeys { ncm, descricao }, *valor))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect();

    let map_reduce: HashMap<NFeKeys, f64> = tuples.group_by_key();
    let opt_valor_total = get_total_value_nfes(nfes, nfe_info);
    format_ncm_ranking(map_reduce, opt_valor_total, arguments.itens)
}

fn format_ncm_ranking<'a>(
    map_reduce: HashMap<NFeKeys<'a>, f64>,
    opt_valor_total: Option<f64>,
    max_items: usize,
) -> Vec<String> {
    match opt_valor_total {
        Some(valor_total) if valor_total > 0.0 => map_reduce
            .into_iter()
            .map(|(keys, valor)| NFeItens {
                keys,
                valor: valor.round_float(2),
                pct: ((valor / valor_total) * 100.0).round_float(2),
            })
            .sorted_by_key(|item| {
                (
                    Reverse((item.valor * 100.0).round() as i64),
                    item.keys.ncm,
                    item.keys.descricao,
                )
            })
            .take(max_items)
            .map(|item| item.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Consolida itens de NF-e por `(ncm, descricao)` em ordem decrescente de valor.
pub fn consolidate_ncm_items<'a, I>(items: I, max_items: usize) -> (Vec<String>, Option<f64>)
where
    I: IntoIterator<Item = (Option<&'a str>, Option<&'a str>, Option<f64>)>,
{
    let mut sum_total = 0.0f64;
    let mut has_total = false;
    let mut grouped: HashMap<NFeKeys<'a>, f64> = HashMap::new();

    for (opt_ncm, opt_desc, opt_val) in items {
        if let Some(valor) = opt_val {
            sum_total += valor;
            has_total = true;
            if valor > 0.0
                && let (Some(ncm), Some(descricao)) = (opt_ncm, opt_desc)
            {
                *grouped.entry(NFeKeys { ncm, descricao }).or_insert(0.0) += valor;
            }
        }
    }

    let opt_total = has_total.then(|| sum_total.round_float(2));
    let list = format_ncm_ranking(grouped, opt_total, max_items);
    (list, opt_total)
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
            "(cnpj_cpf: {}, atributo: {}, valor: {:.2}, pct: {:.2}%)",
            self.keys.tomador_cnpj_cpf, self.keys.tomador_atributo, self.valor, self.pct
        )
    }
}

/// Get ctes grouped by payer
///
/// Informaçẽs de CTes agrupadas por Tomador/Pagador (CNPJ_CPF e Atributo)
///
/// Agrupa e calcula a distribuição de tomadores/pagadores dos fretes (CT-es)
/// vinculados a uma NF-e.
pub fn get_ctes_grouped_by_payer<T: Borrow<InfoCte> + Sync>(
    ctes: &HashSet<String>,
    cte_info: &BTreeMap<KeyDoc, Vec<T>>,
    arguments: &Arguments,
) -> Vec<String> {
    let tuples: Vec<(CTeKeys, f64)> = ctes
        .par_iter() // rayon: parallel iterator
        .flat_map(|cte| {
            let chave_valida = KeyDoc::new(cte, true);
            cte_info.get(&chave_valida).map(|infos| {
                let keys: Vec<(CTeKeys, f64)> = infos
                    .iter()
                    .flat_map(|item| {
                        let info = item.borrow();
                        let opt_cnpj_cpf = info.get_cnpj_cpf_base_do_tomador();
                        let opt_atributo = TOMADOR_DO_SERVICO
                            .get(&info.tomador_codigo)
                            .map(|&t| t.to_string());

                        match (opt_cnpj_cpf, opt_atributo, &info.valor_total) {
                            (Some(tomador_cnpj_cpf), Some(tomador_atributo), Some(valor))
                                if *valor > 0.0 =>
                            {
                                Some((
                                    CTeKeys {
                                        tomador_cnpj_cpf,
                                        tomador_atributo,
                                    },
                                    *valor,
                                ))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();
                keys
            })
        })
        .flatten()
        .collect();

    let map_reduce: HashMap<CTeKeys, f64> = tuples.group_by_key();
    let opt_valor_total = get_total_value_ctes(ctes, cte_info);
    format_payer_ranking(map_reduce, opt_valor_total, arguments.itens)
}

fn format_payer_ranking(
    map_reduce: HashMap<CTeKeys, f64>,
    opt_valor_total: Option<f64>,
    max_items: usize,
) -> Vec<String> {
    match opt_valor_total {
        Some(valor_total) if valor_total > 0.0 => map_reduce
            .into_iter()
            .map(|(keys, valor)| CTeItens {
                keys,
                valor: valor.round_float(2),
                pct: ((valor / valor_total) * 100.0).round_float(2),
            })
            .sorted_by_key(|item| {
                (
                    Reverse((item.valor * 100.0).round() as i64),
                    item.keys.tomador_cnpj_cpf.clone(),
                    item.keys.tomador_atributo.clone(),
                )
            })
            .take(max_items)
            .map(|item| item.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Consolida tomadores de CT-e por `(cnpj_cpf_base, tomador_codigo)` em ordem decrescente de valor.
pub fn consolidate_payer_items<I>(items: I, max_items: usize) -> (Vec<String>, Option<f64>)
where
    I: IntoIterator<Item = (Option<String>, Option<u8>, Option<f64>)>,
{
    let mut sum_total = 0.0f64;
    let mut has_total = false;
    let mut grouped: HashMap<CTeKeys, f64> = HashMap::new();

    for (opt_doc, opt_cod, opt_val) in items {
        if let Some(valor) = opt_val {
            sum_total += valor;
            has_total = true;
            if valor > 0.0
                && let (Some(tomador_cnpj_cpf), Some(&atr)) =
                    (opt_doc, TOMADOR_DO_SERVICO.get(&opt_cod))
            {
                *grouped
                    .entry(CTeKeys {
                        tomador_cnpj_cpf,
                        tomador_atributo: atr.to_string(),
                    })
                    .or_insert(0.0) += valor;
            }
        }
    }

    let opt_total = has_total.then(|| sum_total.round_float(2));
    let list = format_payer_ranking(grouped, opt_total, max_items);
    (list, opt_total)
}

/// Calcula a soma do valor total de CT-es vinculados.
///
/// nfe: [cte1, cte2, cte3, ,,,, cteN]
pub fn get_total_value_ctes<T: Borrow<InfoCte> + Sync>(
    ctes: &HashSet<String>,
    cte_info: &BTreeMap<KeyDoc, Vec<T>>,
) -> Option<f64> {
    ctes.par_iter()
        .map(|cte| {
            let chave_valida = KeyDoc::new(cte, true);
            cte_info // Soma dos Itens do CTe
                .get(&chave_valida)
                .and_then(|infos| {
                    infos
                        .iter()
                        .map(|item| item.borrow().valor_total)
                        .sum::<Option<f64>>()
                })
        })
        .sum::<Option<f64>>()
        .map(|sum| sum.round_float(2))
}

/// Calcula a soma do valor total dos produtos das NF-es vinculadas.
///
/// cte: [nfe1, nfe2, nfe3, ,,,, nfeN]
pub fn get_total_value_nfes<T: Borrow<InfoNfe> + Sync>(
    nfes: &HashSet<String>,
    nfe_info: &BTreeMap<KeyDoc, Vec<T>>,
) -> Option<f64> {
    nfes.par_iter()
        .map(|nfe| {
            nfe_info // Soma dos Itens da NFe
                .get(&KeyDoc::new(nfe, true))
                .and_then(|infos| {
                    infos
                        .iter()
                        .map(|item| item.borrow().v_prod)
                        .sum::<Option<f64>>()
                })
        })
        .sum::<Option<f64>>()
        .map(|sum| sum.round_float(2))
}
