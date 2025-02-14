use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endereco {
    // Código do País
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    // Código do município (utilizar a tabela do IBGE)
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    // Número
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    // Sigla da UF
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    // Complemento
    #[serde(rename = "xCpl")]
    pub x_cpl: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    // Logradouro
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    // Nome do Município
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    // Nome do País
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

impl Endereco {
    /// Nome do Município
    pub fn get_municipio(&self) -> Option<String> {
        self.x_mun.as_ref().map(|c| c.trim().to_uppercase())
    }

    /// Sigla da UF
    pub fn get_unidade_federal(&self) -> Option<String> {
        self.uf.as_ref().map(|c| c.trim().to_uppercase())
    }
}

pub trait EnderecoExtension {
    fn get_endereco_ext_municipio(&self) -> Option<String>;
    fn get_endereco_ext_estado(&self) -> Option<String>;
}

impl EnderecoExtension for Option<Endereco> {
    fn get_endereco_ext_municipio(&self) -> Option<String> {
        self.as_ref().and_then(|endereco| endereco.get_municipio())
    }

    fn get_endereco_ext_estado(&self) -> Option<String> {
        self.as_ref()
            .and_then(|endereco| endereco.get_unidade_federal())
    }
}
