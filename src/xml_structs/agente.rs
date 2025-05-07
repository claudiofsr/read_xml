use claudiofsr_lib::{OptionExtension, StrExtension};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock as Lazy};

use crate::xml_structs::endereco::{Endereco, EnderecoExtension};

/**
    Obter o Tomador do Serviço tal que o

    Código do Tomador do Serviço:

    0. Remetente;
    1. Expedidor;
    2. Recebedor;
    3. Destinatário;
    4. Terceiro `[adicionado em CTe versão 4.00]`.
*/
pub static TOMADOR_DO_SERVICO: Lazy<HashMap<Option<u8>, &'static str>> = Lazy::new(|| {
    // Código do Tomador do Serviço:
    let tuples: [(u8, &str); 5] = [
        (0, "Remetente"),
        (1, "Expedidor"),
        (2, "Recebedor"),
        (3, "Destinatário"),
        (4, "Terceiro"), // [adicionado em CTe versão 4.00]
    ];

    let tomadores = tuples.map(|(n, tomador)| (Some(n), tomador));

    HashMap::from(tomadores)
});

/**
    6 tipos de Agentes:

    1. Destinatário
    2. Emitente
    3. Expedidor
    4. Recebedor
    5. Remetente
    6. Tomador
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct Agente {
    /// CNAE Fiscal
    #[serde(rename = "CNAE")]
    pub cnae: Option<String>,

    /// CNPJ
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,

    /// CPF
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,

    /// Código do Regime Tributário.
    ///
    /// 1 – Simples Nacional;
    ///
    /// 2 – Simples Nacional – excesso de sublimite de receita bruta;
    ///
    /// 3 – Regime Normal.
    #[serde(rename = "CRT")]
    pub crt: Option<String>,

    /// Email
    #[serde(rename = "email")]
    pub email: Option<String>,

    // 6 tipos de Endereço:
    #[serde(
        alias = "enderDest",
        alias = "enderEmit",
        alias = "enderExped",
        alias = "enderReceb",
        alias = "enderReme",
        alias = "enderToma"
    )]
    pub endereco: Option<Endereco>,

    /// Fone
    #[serde(rename = "fone")]
    pub fone: Option<String>,

    /// Inscrição Estadual
    #[serde(rename = "IE")]
    pub ie: Option<String>,

    #[serde(rename = "infNFe")]
    pub inf_nfe: Option<Vec<InfNfe>>,

    /// Inscrição Estadual do Substituto Tributário
    #[serde(rename = "IEST")]
    pub iest: Option<String>,

    /// Inscrição Municipal do tomador do serviço
    #[serde(rename = "IM")]
    pub im: Option<String>,

    /// Código do Tomador do Serviço
    #[serde(rename = "toma")]
    pub toma: Option<String>,

    /// Bairro do endereço
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,

    /// Nome fantasia
    #[serde(rename = "xFant")]
    pub x_fant: Option<String>,

    /// Logradouro
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,

    /// Nome do município.
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,

    /// Razão Social ou Nome
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
}

impl Agente {
    pub fn get_cnpj(&self) -> Option<String> {
        self.cnpj.as_ref().map(|c| c.trim().format_cnpj())
    }

    pub fn get_cpf(&self) -> Option<String> {
        self.cpf.as_ref().map(|c| c.trim().format_cpf())
    }

    /// Código do Regime Tributário (CRT)
    pub fn get_crt(&self) -> Option<u8> {
        self.crt
            .as_ref()
            .and_then(|codigo| codigo.remove_non_digits().parse().ok())
    }

    pub fn get_nome(&self) -> Option<String> {
        self.x_nome.parse()
    }

    pub fn get_fantasia(&self) -> Option<String> {
        self.x_fant.parse()
    }

    pub fn get_endereco_municipio(&self) -> Option<String> {
        self.endereco.get_endereco_ext_municipio()
    }

    pub fn get_endereco_estado(&self) -> Option<String> {
        self.endereco.get_endereco_ext_estado()
    }

    pub fn get_nfes(&self) -> Vec<String> {
        self.inf_nfe
            .iter()
            .flat_map(|vec_info| vec_info.iter().flat_map(|info| info.get_chave()))
            .collect()
    }

    /**
    Código do Tomador do Serviço:

    0. Remetente;
    1. Expedidor;
    2. Recebedor;
    3. Destinatário;
    4. Terceiro `[adicionado em CTe versão 4.00]`.
    */
    pub fn parse_tomador(&self) -> Option<u8> {
        self.toma.parse()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfe {
    pub chave: Option<String>,
}

impl InfNfe {
    pub fn get_chave(&self) -> Option<String> {
        self.chave.parse()
    }
}

pub trait AgenteExtension {
    fn get_ext_cnpj(&self) -> Option<String>;
    fn get_ext_cpf(&self) -> Option<String>;
    fn get_ext_crt(&self) -> Option<u8>;
    fn get_ext_nome(&self) -> Option<String>;
    fn get_ext_fantasia(&self) -> Option<String>;
    fn get_ext_municipio(&self) -> Option<String>;
    fn get_ext_estado(&self) -> Option<String>;
    fn get_ext_chaves(&self) -> Vec<String>;
    fn get_ext_tomador(&self) -> Option<u8>;
}

impl AgenteExtension for Option<Agente> {
    /// Agentes: CNPJ
    fn get_ext_cnpj(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_cnpj())
    }

    /// Agentes: CPF
    fn get_ext_cpf(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_cpf())
    }

    /// Agentes: Código do Regime Tributário (CRT)
    fn get_ext_crt(&self) -> Option<u8> {
        self.as_ref().and_then(|agente| agente.get_crt())
    }

    /// Agentes: None ou Razão Social do Agentes
    fn get_ext_nome(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_nome())
    }

    /// Agentes: None Fantasia do Agentes
    fn get_ext_fantasia(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_fantasia())
    }

    /// Agentes: Endereço do Município
    fn get_ext_municipio(&self) -> Option<String> {
        self.as_ref()
            .and_then(|agente| agente.get_endereco_municipio())
    }

    /// Agentes: Endereço do Estado
    fn get_ext_estado(&self) -> Option<String> {
        self.as_ref()
            .and_then(|agente| agente.get_endereco_estado())
    }

    /// Agentes: Vec<NFe>
    fn get_ext_chaves(&self) -> Vec<String> {
        self.iter()
            .flat_map(|remetente| remetente.get_nfes())
            .collect()
    }

    /// Código do Tomador do Serviço
    fn get_ext_tomador(&self) -> Option<u8> {
        self.as_ref().and_then(|tomador| tomador.parse_tomador())
    }
}

/*
#[derive(Debug, Serialize, Deserialize)]
pub enum Agentes {
    #[serde(rename = "dest")]
    Destinatario(Agente),

    #[serde(rename = "emit")]
    Emitente(Agente),

    #[serde(rename = "exped")]
    Expedidor(Agente),

    #[serde(rename = "receb")]
    Recebedor(Agente),

    #[serde(rename = "rem")]
    Remetente(Agente),

    #[serde(
        alias = "toma3",
        alias = "toma4",
        alias = "toma03",
        alias = "toma04"
    )]
    Tomador(Agente),
}
*/

#[cfg(test)]
mod agentes {
    use super::*;
    use serde_json::Result;

    #[test]
    /// `cargo test -- --show-output flatten_agentes`
    ///
    /// <https://stackoverflow.com/questions/45059538/how-to-deserialize-into-a-enum-variant-based-on-a-key-name>
    fn flatten_agentes() -> Result<()> {
        #[derive(Debug, Serialize, Deserialize)]
        struct MyStruct {
            field1: i32,
            #[serde(flatten)]
            an_enum: Option<MyEnum>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        enum MyEnum {
            #[serde(rename = "name_a")]
            FieldA(i32),
            FieldB(i32),
        }

        let jsons = [
            r#"{ "field1": 12, "FieldA": 15 }"#,
            r#"{ "field1": 42, "name_a": 76 }"#,
            r#"{ "field1": 15, "FieldB": 10 }"#,
            r#"{ "field1": 61, "name_a": 38, "FieldB": 54 }"#,
            r#"{ "field1": 17}"#,
        ];

        for (index, json) in jsons.iter().enumerate() {
            let result = serde_json::from_str::<MyStruct>(json)?;
            println!("result {}: {:?}", index + 1, result);
        }

        Ok(())
    }
}
