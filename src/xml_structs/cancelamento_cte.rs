//! # Processamento de Cancelamento de CT-e
//!
//! Este módulo gerencia o mapeamento, a extração de dados e a consolidação de eventos
//! de cancelamento homologados para Conhecimentos de Transporte Eletrônicos (CT-e).
//!
//! As estruturas aqui mapeadas lidam com o parsing do protocolo de cancelamento
//! (`ProcCancCte`) enviado pela SEFAZ e a geração de uma representação simplificada
//! e limpa (`InfoCteCancel`).

use crate::{
    Arguments, GetKey, InfoExtension, Information, StructExtension,
    xml_structs::cancelamento::{CancelExt, Cancelamento, Retencao},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Representação intermediária e consolidada de um evento de cancelamento de CT-e.
///
/// Esta estrutura unifica a identificação do documento fiscal afetado e a respectiva
/// data de homologação, utilizada para posterior exportação e cruzamento de dados.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoCteCancel {
    /// Chave de acesso de 44 dígitos do CT-e cancelado.
    #[serde(default)]
    pub cte: Option<String>,

    /// Data em que o cancelamento foi homologado pela SEFAZ.
    #[serde(default)]
    pub dh_recebimento: Option<NaiveDate>,

    /// Indicador lógico se o documento fiscal correspondente está cancelado.
    #[serde(default)]
    pub cancelado: bool,
}

/// Implementação do traço de formatação de metadados para geração do Excel.
///
/// Ref: <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCteCancel {}

impl GetKey for InfoCteCancel {
    /// Retorna a chave de acesso do CT-e associada a este cancelamento para indexação.
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

/// Estrutura correspondente ao XML do processo de cancelamento de CT-e (`<procCancCTe>`).
///
/// ### Zoneamento Estrutural do Serde
///
/// Para suportar variações na ordenação das tags em fluxos de integração, a struct adota:
/// 1. **Atributos XML no Topo (`@`)**: Capturados prioritariamente e decorados com `default`.
/// 2. **Campos Opcionais no Centro (`default`)**: Mapeia os dados do cancelamento (`cancCTe`)
///    e o respectivo retorno homologado (`retCancCTe`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcCancCte {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do leiaute XML do processo de cancelamento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente ao processo.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou inversão física de tags)
    // =========================================================================
    /// Bloco de informações detalhadas do pedido de cancelamento do CT-e.
    #[serde(rename = "cancCTe", default)]
    pub canc_cte: Option<Cancelamento>,

    /// Bloco correspondente ao retorno e homologação emitida pela SEFAZ.
    #[serde(rename = "retCancCTe", default)]
    pub ret_canc_cte: Option<Retencao>,
}

impl ProcCancCte {
    /// Extrai a chave de acesso do CT-e que foi alvo deste cancelamento.
    pub fn get_cte(&self) -> Option<String> {
        self.canc_cte.get_chave_cancelada_cte()
    }

    /// Obtém a data de homologação do recebimento do cancelamento.
    pub fn get_dh_recebimento(&self) -> Option<NaiveDate> {
        self.canc_cte.get_dh_recebimento()
    }

    /// Determina se a estrutura contém informações válidas e consistentes de cancelamento.
    pub fn informacao_de_cancelamento(&self) -> bool {
        self.get_cte().is_some()
    }

    /// Consolida os dados desserializados em uma estrutura simplificada `InfoCteCancel`.
    pub fn get_info(&self) -> InfoCteCancel {
        InfoCteCancel {
            cte: self.get_cte(),
            dh_recebimento: self.get_dh_recebimento(),
            cancelado: self.informacao_de_cancelamento(),
        }
    }
}

/// Implementação do traço do sistema para conversão do processamento XML em Enums de Informação.
///
/// Ref: <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcCancCte {
    /// Converte os nós do processo de cancelamento do XML para uma representação Enum `Information`.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("proc canc cte xml_path: {xml_path:?}");
            println!("proc_canc_cte: {self:#?}\n");
        }
        Information::CancelamentoCte(Box::new(self.get_info()))
    }
}
