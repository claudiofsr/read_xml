//! # Cadastro de Agentes (Participantes)
//!
//! Este módulo gerencia os participantes (Agentes) que figuram nos documentos fiscais eletrônicos
//! (NF-e e CT-e), como o Emitente, Destinatário, Expedidor, Recebedor, Remetente e Tomador.

use claudiofsr_lib::{OptionExtension, StrExtension};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock as Lazy};

use crate::xml_structs::endereco::{Endereco, EnderecoExtension};

/// Mapeamento descritivo dos papéis atribuídos ao Tomador do Serviço de Transporte.
///
/// Código do Tomador do Serviço:
///
/// * `0` - Remetente;
/// * `1` - Expedidor;
/// * `2` - Recebedor;
/// * `3` - Destinatário;
/// * `4` - Terceiro `[adicionado em CT-e versão 4.00]`.
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

/// Representação unificada dos participantes (Agentes) em documentos fiscais eletrônicos.
///
/// Existem 6 papéis possíveis mapeados sob esta estrutura genérica:
///
/// 1. Destinatário (`dest`)
/// 2. Emitente (`emit`)
/// 3. Expedidor (`exped`)
/// 4. Recebedor (`receb`)
/// 5. Remetente (`rem`)
/// 6. Tomador (`toma`)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agente {
    /// Classificação Nacional de Atividades Econômicas (CNAE Fiscal).
    #[serde(rename = "CNAE", default)]
    pub cnae: Option<String>,

    /// Cadastro Nacional da Pessoa Jurídica (CNPJ).
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Cadastro de Pessoas Físicas (CPF).
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Código do Regime Tributário (CRT) do participante:
    ///
    /// * `1` - Simples Nacional;
    /// * `2` - Simples Nacional - excesso de sublimite de receita bruta;
    /// * `3` - Regime Normal.
    #[serde(rename = "CRT", default)]
    pub crt: Option<String>,

    /// Correio eletrônico para contato.
    #[serde(rename = "email", default)]
    pub email: Option<String>,

    /// Endereço físico estruturado associado ao participante.
    ///
    /// Mapeia dinamicamente os aliases de endereço utilizados em diferentes seções do XML.
    #[serde(
        alias = "enderDest",
        alias = "enderEmit",
        alias = "enderExped",
        alias = "enderReceb",
        alias = "enderReme",
        alias = "enderToma",
        default
    )]
    pub endereco: Option<Endereco>,

    /// Telefone para contato.
    #[serde(rename = "fone", default)]
    pub fone: Option<String>,

    /// Inscrição Estadual (IE) do participante.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,

    /// Indicador da IE do Destinatário:
    /// * `1` - Contribuinte ICMS (pagamento de imposto);
    /// * `2` - Contribuinte isento de Inscrição;
    /// * `9` - Não Contribuinte, que pode ou não ter Inscrição Estadual.
    #[serde(rename = "indIEDest", default)]
    pub ind_ie_dest: Option<String>,

    /// Lista de referências a chaves de NF-e associadas ao participante.
    #[serde(rename = "infNFe", default)]
    inf_nfe: Option<Vec<AgenteInfNfe>>, // Nome alterado (de InfNfe) para clareza.

    /// Inscrição Estadual do Substituto Tributário (IEST).
    #[serde(rename = "IEST", default)]
    pub iest: Option<String>,

    /// Inscrição Municipal (IM) do prestador ou tomador.
    #[serde(rename = "IM", default)]
    pub im: Option<String>,

    /// Indicador numérico do papel do Tomador do Serviço.
    #[serde(rename = "toma", default)]
    pub toma: Option<String>,

    /// Nome do bairro do participante.
    #[serde(rename = "xBairro", default)]
    pub x_bairro: Option<String>,

    /// Nome Fantasia comercial do participante.
    #[serde(rename = "xFant", default)]
    pub x_fant: Option<String>,

    /// Logradouro do endereço do participante.
    #[serde(rename = "xLgr", default)]
    pub x_lgr: Option<String>,

    /// Nome do município do participante.
    #[serde(rename = "xMun", default)]
    pub x_mun: Option<String>,

    /// Razão Social ou Nome completo do participante.
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
}

impl Agente {
    /// Obtém o CNPJ do participante formatado com a máscara padrão.
    pub fn get_cnpj(&self) -> Option<String> {
        self.cnpj.as_ref().map(|c| c.trim().format_cnpj())
    }

    /// Obtém o CPF do participante formatado com a máscara padrão.
    pub fn get_cpf(&self) -> Option<String> {
        self.cpf.as_ref().map(|c| c.trim().format_cpf())
    }

    /// Retorna o Código do Regime Tributário (CRT) de forma numérica.
    pub fn get_crt(&self) -> Option<u8> {
        self.crt
            .as_ref()
            .and_then(|codigo| codigo.remove_non_digits().parse().ok())
    }

    /// Obtém a Razão Social ou Nome completo sanitizado.
    pub fn get_nome(&self) -> Option<String> {
        self.x_nome.parse_opt()
    }

    /// Obtém o Nome Fantasia sanitizado.
    pub fn get_fantasia(&self) -> Option<String> {
        self.x_fant.parse_opt()
    }

    /// Obtém o município correspondente ao endereço do participante.
    pub fn get_endereco_municipio(&self) -> Option<String> {
        self.endereco.get_endereco_ext_municipio()
    }

    /// Obtém a sigla da UF correspondente ao endereço do participante.
    pub fn get_endereco_estado(&self) -> Option<String> {
        self.endereco.get_endereco_ext_estado()
    }

    /// Varre as referências de NF-e associadas ao participante e consolida suas chaves de acesso.
    pub fn get_nfes(&self) -> Vec<String> {
        self.inf_nfe
            .iter()
            .flat_map(|vec_info| vec_info.iter().flat_map(|info| info.get_chave()))
            .collect()
    }

    /// Código de identificação do Tomador do Serviço de Transporte:
    ///
    /// * `0` - Remetente;
    /// * `1` - Expedidor;
    /// * `2` - Recebedor;
    /// * `3` - Destinatário;
    /// * `4` - Terceiro `[adicionado em CT-e versão 4.00]`.
    pub fn parse_tomador(&self) -> Option<u8> {
        self.toma.parse_opt()
    }
}

/// Representação minimalista utilizada para referenciar chaves de acesso vinculadas a agentes (`<infNFe>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AgenteInfNfe {
    /// Chave de acesso de 44 dígitos correspondente à NF-e vinculada.
    #[serde(rename = "chave", default)]
    pub chave: Option<String>,
}

impl AgenteInfNfe {
    /// Obtém a chave de acesso mapeada de forma limpa.
    pub fn get_chave(&self) -> Option<String> {
        self.chave.parse_opt()
    }
}

/// Extensão auxiliar projetada para facilitar o acesso indireto a campos de agentes encapsulados em `Option<Agente>`.
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
    /// Obtém o CNPJ do participante se este estiver presente.
    fn get_ext_cnpj(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_cnpj())
    }

    /// Obtém o CPF do participante se este estiver presente.
    fn get_ext_cpf(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_cpf())
    }

    /// Obtém o CRT (Código do Regime Tributário) do participante se este estiver presente.
    fn get_ext_crt(&self) -> Option<u8> {
        self.as_ref().and_then(|agente| agente.get_crt())
    }

    /// Obtém a Razão Social ou Nome do participante se este estiver presente.
    fn get_ext_nome(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_nome())
    }

    /// Obtém o Nome Fantasia do participante se este estiver presente.
    fn get_ext_fantasia(&self) -> Option<String> {
        self.as_ref().and_then(|agente| agente.get_fantasia())
    }

    /// Obtém o município associado ao endereço do participante se este estiver presente.
    fn get_ext_municipio(&self) -> Option<String> {
        self.as_ref()
            .and_then(|agente| agente.get_endereco_municipio())
    }

    /// Obtém o Estado (UF) associado ao endereço do participante se este estiver presente.
    fn get_ext_estado(&self) -> Option<String> {
        self.as_ref()
            .and_then(|agente| agente.get_endereco_estado())
    }

    /// Retorna uma lista contendo as chaves de acesso de NF-e vinculadas ao participante se este estiver presente.
    fn get_ext_chaves(&self) -> Vec<String> {
        self.iter()
            .flat_map(|remetente| remetente.get_nfes())
            .collect()
    }

    /// Obtém o código numérico do tomador se este estiver presente.
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
