
use claudiofsr_lib::StrExtension;
use serde::{Deserialize, Serialize};

/**
Tomador do Serviço:
    0-Remetente;
    1-Expedidor;
    2-Recebedor;
    3-Destinatário
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct Toma4 {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(rename = "enderToma")]
    pub ender_toma: Option<Endereco>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "toma")]
    pub toma: Option<String>,
    #[serde(rename = "xFant")]
    pub x_fant: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
}

impl Toma4 {
    pub fn get_cnpj(&self) -> Option<String> {
        self
            .cnpj
            .as_ref()
            .map(|c| c.trim().format_cnpj())
    }

    pub fn get_nome(&self) -> Option<String> {
        self
            .x_nome
            .as_ref()
            .map(|c| c.trim().to_string())
    }

    pub fn get_endereco_municipio(&self) -> Option<String> {
        self
            .ender_toma
            .as_ref()
            .and_then(|endereco| {
                endereco
                    .x_mun
                    .as_ref()
                    .map(|c| c.trim().to_string())
            })
    }

    pub fn get_endereco_uf(&self) -> Option<String> {
        self
            .ender_toma
            .as_ref()
            .and_then(|endereco| {
                endereco
                    .uf
                    .as_ref()
                    .map(|c| c.trim().to_string())
            })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endereco {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "xCpl")]
    pub x_cpl: Option<String>,
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
}