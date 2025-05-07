use crate::{OptExt, get_naive_date_from_yyyy_mm_dd, xml_structs::assinaturas::Signature};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub trait CancelExt {
    fn get_chave_cancelada_cte(&self) -> Option<String>;
    fn get_chave_cancelada_nfe(&self) -> Option<String>;
    fn get_dh_recebimento(&self) -> Option<NaiveDate>;
}

impl CancelExt for Option<Cancelamento> {
    fn get_chave_cancelada_cte(&self) -> Option<String> {
        self.as_ref()
            .and_then(|cancelamento| cancelamento.inf_canc.ch_cte.get_key())
    }

    fn get_chave_cancelada_nfe(&self) -> Option<String> {
        self.as_ref()
            .and_then(|cancelamento| cancelamento.inf_canc.ch_nfe.get_key())
    }

    fn get_dh_recebimento(&self) -> Option<NaiveDate> {
        self.as_ref()
            .and_then(|cancelamento| cancelamento.inf_canc.get_dh_recbto())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cancelamento {
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "infCanc")]
    pub inf_canc: InfoCancelamento,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Retencao {
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "infCanc")]
    pub inf_canc: InfoCancelamento,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoCancelamento {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "cStat")]
    pub c_stat: Option<String>,
    #[serde(rename = "cUF")]
    pub c_uf: Option<String>,

    // 2 Chaves:
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    #[serde(rename = "chNFe")]
    pub ch_nfe: Option<String>,

    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>,
    #[serde(rename = "verAplic")]
    pub ver_aplic: Option<String>,
    #[serde(rename = "xJust")]
    pub x_just: Option<String>,
    #[serde(rename = "xMotivo")]
    pub x_motivo: Option<String>,
    #[serde(rename = "xServ")]
    pub x_serv: Option<String>,
}

impl InfoCancelamento {
    pub fn get_dh_recbto(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_recbto)
    }
}
