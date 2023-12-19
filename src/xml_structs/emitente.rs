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
pub struct Emitente {
    /// Número do CNPJ do emitente
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    /// Número do CPF do emitente
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    /// CNAE Fiscal
    #[serde(rename = "CNAE")]
    pub cnae: Option<String>,
    /// Código de Regime Tributário.
    ///
    /// 1 – Simples Nacional;
    ///
    /// 2 – Simples Nacional – excesso de sublimite de receita bruta;
    ///
    /// 3 – Regime Normal.
    #[serde(rename = "CRT")]
    pub crt: Option<String>,
    /// Endereço do emitente
    #[serde(rename = "enderEmit")]
    pub ender_emit: Option<Endereco>,
    /// Inscrição Estadual
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    /// Inscrição Estadual do Substituto Tributário
    #[serde(rename = "IEST")]
    pub iest: Option<String>,
    /// Inscrição Municipal do tomador do serviço
    #[serde(rename = "IM")]
    pub im: Option<String>,
    /// Razão Social ou Nome do destinatário
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    /// Nome fantasia
    #[serde(rename = "xFant")]
    pub x_fant: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

impl Emitente {
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

    pub fn get_fantasia(&self) -> Option<String> {
        self
            .x_fant
            .as_ref()
            .map(|c| c.trim().to_string())
    }

    pub fn get_endereco_municipio(&self) -> Option<String> {
        self
            .ender_emit
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
            .ender_emit
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
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "xCpl")]
    pub x_cpl: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}