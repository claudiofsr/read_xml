
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
pub struct Expedidor {
    /// CNPJ do expedidor
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    /// CPF do expedidor
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
       /// Código do Regime Tributário.
    ///
    /// 1 – Simples Nacional;
    ///
    /// 2 – Simples Nacional – excesso de sublimite de receita bruta;
    ///
    /// 3 – Regime Normal.
    #[serde(rename = "CRT")]
    pub crt: Option<String>,
    #[serde(rename = "enderExped")]
    pub ender_exped: Option<Endereco>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    /// Razão Social ou Nome do expedidor
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    /// Nome Fantasia
    #[serde(rename = "xFant")]
    pub x_fant: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

impl Expedidor {
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

    /// Código do Regime Tributário.
    pub fn get_crt(&self) -> Option<u8> {
        self
            .crt
            .as_ref()
            .and_then(|codigo| {
                codigo
                    .remove_non_digits()
                    .parse()
                    .ok()
            })
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
            .ender_exped
            .as_ref()
            .and_then(|endereco| {
                endereco.get_municipio()
            })
    }

    pub fn get_endereco_uf(&self) -> Option<String> {
        self
            .ender_exped
            .as_ref()
            .and_then(|endereco| {
                endereco.get_unidade_federal()
            })
    }
}
