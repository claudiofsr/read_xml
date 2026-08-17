//! # Agregações Fiscais e Agrupamentos Analíticos
//!
//! Fornece funções de soma e consolidação para categorizar itens de NF-e por
//! NCM/Descrição e agrupar tomadores de CT-e por relevância de valores.

use claudiofsr_lib::RoundFloat;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;
use std::{
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
pub fn get_nfes_grouped_by_ncm_description(
    nfes: &HashSet<String>,
    nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>,
    arguments: &Arguments,
) -> Vec<String> {
    let tuples: Vec<(NFeKeys, f64)> = nfes
        .par_iter() // rayon: parallel iterator
        .flat_map(|nfe| {
            let chave_valida = KeyDoc::new(nfe, true);
            nfe_info.get(&chave_valida).map(|infos| {
                infos
                    .iter()
                    .flat_map(|info| match (&info.ncm, &info.descricao, &info.v_prod) {
                        (Some(ncm), Some(descricao), Some(valor)) if *valor > 0.0 => {
                            Some((NFeKeys { ncm, descricao }, *valor))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect();

    let map_reduce: HashMap<NFeKeys, f64> = tuples.group_by_key();
    let opt_valor_total = get_total_value_nfes(nfes, nfe_info);

    match opt_valor_total {
        Some(valor_total) if valor_total > 0.0 => map_reduce
            .into_iter()
            .map(|(keys, valor)| {
                let porcentagem = (valor / valor_total) * 100.0;
                NFeItens {
                    keys,
                    valor: valor.round_float(2),
                    pct: porcentagem.round_float(2),
                }
            })
            // sort in descending order of values
            .sorted_by_key(|item| {
                (
                    Reverse((item.valor * 100.0) as i64),
                    item.keys.ncm,
                    item.keys.descricao,
                )
            })
            // Collect at most the N largest items
            .take(arguments.itens)
            .map(|item| item.to_string())
            .collect(),
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
pub fn get_ctes_grouped_by_payer(
    ctes: &HashSet<String>,
    cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>,
    arguments: &Arguments,
) -> Vec<String> {
    let tuples: Vec<(CTeKeys, f64)> = ctes
        .par_iter() // rayon: parallel iterator
        .flat_map(|cte| {
            let chave_valida = KeyDoc::new(cte, true);
            cte_info.get(&chave_valida).map(|infos| {
                let keys: Vec<(CTeKeys, f64)> = infos
                    .iter()
                    .flat_map(|info| {
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

    match opt_valor_total {
        Some(valor_total) if valor_total > 0.0 => map_reduce
            .into_iter()
            .map(|(keys, valor)| {
                let pct = (valor / valor_total) * 100.0;
                CTeItens {
                    keys,
                    valor: valor.round_float(2),
                    pct: pct.round_float(2),
                }
            })
            // sort in descending order of values
            .sorted_by_key(|item| {
                (
                    Reverse((item.valor * 100.0) as i64), // 10 ^ 2 = 100
                    item.keys.tomador_cnpj_cpf.clone(),
                    item.keys.tomador_atributo.clone(),
                )
            })
            .take(arguments.itens)
            .map(|item| item.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Calcula a soma do valor total de CT-es vinculados.
///
/// nfe: [cte1, cte2, cte3, ,,,, cteN]
pub fn get_total_value_ctes(
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

/// Calcula a soma do valor total dos produtos das NF-es vinculadas.
///
/// cte: [nfe1, nfe2, nfe3, ,,,, nfeN]
pub fn get_total_value_nfes(
    nfes: &HashSet<String>,
    nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>,
) -> Option<f64> {
    nfes.par_iter()
        .map(|nfe| {
            nfe_info // Soma dos Itens da NFe
                .get(&KeyDoc::new(nfe, true))
                .and_then(|infos| infos.iter().map(|info| info.v_prod).sum::<Option<f64>>())
        })
        .sum::<Option<f64>>()
        .map(|sum| sum.round_float(2))
}
