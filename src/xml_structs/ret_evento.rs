use super::assinaturas::Signature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RetEvento {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "infEvento")]
    pub inf_evento: Option<RetInfEvento>,
    #[serde(rename = "Signature")]
    pub signature: Option<Signature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetInfEvento {
    #[serde(rename = "cOrgao")]
    pub c_orgao: Option<String>,
    #[serde(rename = "cOrgaoAutor")]
    pub c_orgao_autor: Option<String>,
    #[serde(rename = "cStat")]
    pub c_stat: Option<String>,
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    #[serde(rename = "chNFe")]
    pub ch_nfe: Option<String>,
    #[serde(rename = "CNPJDest")]
    pub cnpjdest: Option<String>,
    #[serde(rename = "dhRegEvento")]
    pub dh_reg_evento: Option<String>,
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "nSeqEvento")]
    pub n_seq_evento: Option<String>,
    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>, // Identificação do Ambiente: 1 - Produção; 2 - Homologação
    #[serde(rename = "tpEvento")]
    pub tp_evento: Option<String>,
    #[serde(rename = "verAplic")]
    pub ver_aplic: Option<String>,
    #[serde(rename = "xMotivo")]
    pub x_motivo: Option<String>,
    #[serde(rename = "xEvento")]
    pub x_evento: Option<String>,
}
