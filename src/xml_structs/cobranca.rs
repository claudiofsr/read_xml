use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cobranca {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dup")]
    pub dup: Option<Vec<Dup>>,
    #[serde(rename = "fat")]
    pub fat: Option<Fat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dup {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dVenc")]
    pub d_venc: Option<String>,
    #[serde(rename = "nDup")]
    pub n_dup: Option<String>,
    #[serde(rename = "vDup")]
    pub v_dup: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fat {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nFat")]
    pub n_fat: Option<String>,
    #[serde(rename = "vDesc")]
    pub v_desc: Option<String>,
    #[serde(rename = "vLiq")]
    pub v_liq: Option<String>,
    #[serde(rename = "vOrig")]
    pub v_orig: Option<String>,
}
