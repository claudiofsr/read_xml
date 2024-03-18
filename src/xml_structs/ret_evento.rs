use serde::{Deserialize, Serialize};
use super::assinaturas::Signature;

#[derive(Debug, Serialize, Deserialize)]
pub struct RetEvento {
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "infEvento")]
    pub inf_evento: RetInfEvento,
    #[serde(rename = "Signature")]
    pub signature: Option<Signature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetInfEvento {
    #[serde(rename = "cOrgao")]
    pub c_orgao: String,
    #[serde(rename = "cOrgaoAutor")]
    pub c_orgao_autor: Option<String>,
    #[serde(rename = "cStat")]
    pub c_stat: String,
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
    pub n_prot: String,
    #[serde(rename = "nSeqEvento")]
    pub n_seq_evento: String,
    #[serde(rename = "tpAmb")]
    pub tp_amb: String, // Identificação do Ambiente: 1 - Produção; 2 - Homologação
    #[serde(rename = "tpEvento")]
    pub tp_evento: String,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    #[serde(rename = "xEvento")]
    pub x_evento: Option<String>,
}