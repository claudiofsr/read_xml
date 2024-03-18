
use claudiofsr_lib::StrExtension;
use serde::{Deserialize, Serialize};
use crate::xml_structs::endereco::Endereco;

/**
Tomador do Serviço:
    0-Remetente;
    1-Expedidor;
    2-Recebedor;
    3-Destinatário
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct Tomador {
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
    #[serde(rename = "enderToma3")]
    pub ender_toma3: Option<Endereco>,
    #[serde(rename = "enderToma4")]
    pub ender_toma4: Option<Endereco>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "toma")]
    pub toma: Option<String>,
    /// Nome Fantasia
    #[serde(rename = "xFant")]
    pub x_fant: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

impl Tomador {
    pub fn get_cnpj(&self) -> Option<String> {
        self
            .cnpj
            .as_ref()
            .map(|c| c.trim().format_cnpj())
    }

    pub fn get_cpf(&self) -> Option<String> {
        self
            .cpf
            .as_ref()
            .map(|c| c.trim().format_cpf())
    }

    pub fn get_nome(&self) -> Option<String> {
        self
            .x_nome
            .as_ref()
            .map(|c| c.trim().to_string())
    }

    pub fn get_fantasia(&self) -> Option<String> {
        self
            .x_fant
            .as_ref()
            .map(|c| c.trim().to_string())
    }

    pub fn get_endereco_municipio(&self) -> Option<String> {
        self
            .ender_toma
            .as_ref()
            .and_then(|endereco| {
                endereco.get_municipio()
            })
    }

    pub fn get_endereco_uf(&self) -> Option<String> {
        self
            .ender_toma
            .as_ref()
            .and_then(|endereco| {
                endereco.get_unidade_federal()
            })
    }

    /**
    Tomador do Serviço:
        0-Remetente;
        1-Expedidor;
        2-Recebedor;
        3-Destinatário
    */
    pub fn get_codigo_do_tomador(&self) -> Option<u8> {
        self
            .toma
            .as_ref()
            .and_then(|codigo| {
                codigo.trim().parse::<u8>().ok()
            })
    }
}
