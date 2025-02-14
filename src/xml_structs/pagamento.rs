use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagamento {
    #[serde(rename = "vTroco")]
    pub v_troco: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "detPag")]
    pub det_pag: Vec<DetalhesPagamento>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetalhesPagamento {
    #[serde(rename = "indPag")]
    pub ind_pag: Option<String>,
    #[serde(rename = "tPag")]
    pub t_pag: Option<String>,
    #[serde(rename = "xPag")]
    pub x_pag: Option<String>,
    #[serde(rename = "vPag")]
    pub v_pag: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "card")]
    pub card: Option<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    #[serde(rename = "tpIntegra")]
    pub tp_integra: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "tBand")]
    pub t_band: Option<String>,
    #[serde(rename = "cAut")]
    pub c_aut: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}
