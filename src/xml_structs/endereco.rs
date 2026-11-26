//! # Mapeamento de Endereços Fiscais
//!
//! Este módulo gerencia os dados estruturados de localização física (`Endereco`)
//! que são comuns a emitentes, destinatários, tomadores e outros agentes
//! nas Notas Fiscais Eletrônicas (NF-e) e Conhecimentos de Transporte (CT-e).

use serde::{Deserialize, Serialize};

/// Dados estruturados correspondentes ao endereço físico de um Agente fiscal (`<enderDest>`, `<enderEmit>`, etc.).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endereco {
    /// Código correspondente ao País de acordo com a tabela do BACEN.
    #[serde(rename = "cPais", default)]
    pub c_pais: Option<String>,

    /// Código correspondente ao município de acordo com a tabela do IBGE.
    #[serde(rename = "cMun", default)]
    pub c_mun: Option<String>,

    /// Código de Endereçamento Postal (CEP).
    #[serde(rename = "CEP", default)]
    pub cep: Option<String>,

    /// Telefone de contato cadastrado.
    #[serde(rename = "fone", default)]
    pub fone: Option<String>,

    /// Número identificador correspondente ao endereço.
    #[serde(rename = "nro", default)]
    pub nro: Option<String>,

    /// Sigla correspondente à Unidade Federativa (UF).
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Complemento do endereço (p. ex. apartamento, sala, bloco).
    #[serde(rename = "xCpl", default)]
    pub x_cpl: Option<String>,

    /// Nome descritivo do Bairro correspondente.
    #[serde(rename = "xBairro", default)]
    pub x_bairro: Option<String>,

    /// Nome descritivo do Logradouro (p. ex. rua, avenida).
    #[serde(rename = "xLgr", default)]
    pub x_lgr: Option<String>,

    /// Nome descritivo do Município.
    #[serde(rename = "xMun", default)]
    pub x_mun: Option<String>,

    /// Nome descritivo do País correspondente.
    #[serde(rename = "xPais", default)]
    pub x_pais: Option<String>,

    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl Endereco {
    /// Retorna o nome do Município de forma higienizada.
    ///
    /// Remove espaços nas bordas, converte todos os caracteres para caixa alta
    /// e descarta strings vazias, retornando `None` nesses casos.
    pub fn get_municipio(&self) -> Option<String> {
        self.x_mun
            .as_ref()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .map(|c| c.to_uppercase())
    }

    /// Retorna a sigla da Unidade Federativa (UF) de forma higienizada.
    ///
    /// Remove espaços nas bordas, converte todos os caracteres para caixa alta
    /// e descarta strings vazias, retornando `None` nesses casos.
    pub fn get_unidade_federal(&self) -> Option<String> {
        self.uf
            .as_ref()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .map(|c| c.to_uppercase())
    }
}

/// Extensão de conveniência para estruturas opcionais que encapsulam dados de endereço.
pub trait EnderecoExtension {
    /// Tenta recuperar o nome do município higienizado a partir de uma referência opcional de `Endereco`.
    fn get_endereco_ext_municipio(&self) -> Option<String>;
    /// Tenta recuperar a sigla da UF higienizada a partir de uma referência opcional de `Endereco`.
    fn get_endereco_ext_estado(&self) -> Option<String>;
}

impl EnderecoExtension for Option<Endereco> {
    /// Recupera de forma segura o município higienizado diretamente de uma referência opcional ao endereço.
    fn get_endereco_ext_municipio(&self) -> Option<String> {
        self.as_ref().and_then(|endereco| endereco.get_municipio())
    }

    /// Recupera de forma segura a UF higienizada diretamente de uma referência opcional ao endereço.
    fn get_endereco_ext_estado(&self) -> Option<String> {
        self.as_ref()
            .and_then(|endereco| endereco.get_unidade_federal())
    }
}
