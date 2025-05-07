use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entrega {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "xCpl")]
    pub x_cpl: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "comData")]
    pub com_data: Option<ComData>,
    #[serde(rename = "comHora")]
    pub com_hora: Option<ComHora>,
    #[serde(rename = "noInter")]
    pub no_inter: Option<NoInter>,
    #[serde(rename = "noPeriodo")]
    pub no_periodo: Option<NoPeriodo>,
    #[serde(rename = "semData")]
    pub sem_data: Option<SemData>,
    #[serde(rename = "semHora")]
    pub sem_hora: Option<SemHora>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComData {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dProg")]
    pub d_prog: Option<String>,
    #[serde(rename = "tpPer")]
    pub tp_per: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComHora {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "hProg")]
    pub h_prog: Option<String>,
    #[serde(rename = "tpHor")]
    pub tp_hor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoInter {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "hFim")]
    pub h_fim: Option<String>,
    #[serde(rename = "hIni")]
    pub h_ini: Option<String>,
    #[serde(rename = "tpHor")]
    pub tp_hor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoPeriodo {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dFim")]
    pub d_fim: Option<String>,
    #[serde(rename = "dIni")]
    pub d_ini: Option<String>,
    #[serde(rename = "tpPer")]
    pub tp_per: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemData {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "tpPer")]
    pub tp_per: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemHora {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "tpHor")]
    pub tp_hor: Option<String>,
}
