//! # Acoplamento de Eventos e Cancelamentos
//!
//! Aplica cancelamentos e complementos aos documentos fiscais correspondentes.

use rayon::prelude::*;
use std::collections::HashMap;

use crate::{
    GroupByHashMapExt,
    xml_structs::{
        cancelamento_cte::InfoCteCancel, cancelamento_nfe::InfoNfeCancel, cte::InfoCte,
        cte_evento::InfoCteEvento, nfe::InfoNfe, nfe_evento::InfoNfeEvento,
    },
};

/// Atualiza as NF-es marcando-as como canceladas com base em eventos e retornos homologados.
pub fn adicionar_eventos_nfe(
    nfes: &mut [InfoNfe],
    eventos_nfe: &[InfoNfeEvento],
    cancelamentos_nfe: &[InfoNfeCancel],
) {
    // Em NFe pode conter vários eventos independentes.
    // HashMap<chave_nfe, Vec<&InfoNfeEvento>>

    let eventos_map: HashMap<String, Vec<&InfoNfeEvento>> =
        eventos_nfe.group_by_hashmap_key_vector_v2();

    let cancel_map: HashMap<String, Vec<&InfoNfeCancel>> =
        cancelamentos_nfe.group_by_hashmap_key_vector_v2();

    nfes.par_iter_mut().for_each(|info_nfe| {
        if let Some(nfe) = &info_nfe.nfe {
            if let Some(eventos) = eventos_map.get(nfe)
                && eventos.iter().any(|e| e.cancelado)
            {
                info_nfe.cancelado = Some("Sim".to_string());
                return;
            }

            if let Some(cancelamentos) = cancel_map.get(nfe)
                && cancelamentos.iter().any(|c| c.cancelado)
            {
                info_nfe.cancelado = Some("Sim".to_string());
            }
        }
    });
}

/// Atualiza os CT-es com marcação de cancelamento e incorpora documentos complementares.
pub fn adicionar_eventos_cte(
    ctes: &mut [InfoCte],
    eventos_cte: &[InfoCteEvento],
    cancelamentos_cte: &[InfoCteCancel],
) {
    // Em CTe pode conter vários eventos independentes.
    // HashMap<chave_cte, Vec<&InfoCteEvento>>

    let eventos_map: HashMap<String, Vec<&InfoCteEvento>> =
        eventos_cte.group_by_hashmap_key_vector_v2();

    let cancel_map: HashMap<String, Vec<&InfoCteCancel>> =
        cancelamentos_cte.group_by_hashmap_key_vector_v2();

    ctes.par_iter_mut().for_each(|info_cte| {
        if let Some(cte) = &info_cte.cte {
            if let Some(eventos) = eventos_map.get(cte) {
                for evento in eventos {
                    if evento.cancelado {
                        info_cte.cancelado = Some("Sim".to_string());
                    }
                    // append() or extend() or concat()
                    // https://rustjobs.dev/blog/vector-concatenation-in-rust/
                    info_cte.cte_complementar.extend(evento.get_ctes());
                }
            }

            if let Some(cancelamentos) = cancel_map.get(cte)
                && cancelamentos.iter().any(|c| c.cancelado)
            {
                info_cte.cancelado = Some("Sim".to_string());
            }
        }
    });
}
