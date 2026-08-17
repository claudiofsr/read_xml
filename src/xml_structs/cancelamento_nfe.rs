//! # Processamento de Cancelamento de NF-e
//!
//! Este módulo gerencia os esquemas de validação e desserialização relativos aos eventos
//! de cancelamento de Notas Fiscais Eletrônicas (`ProcCancNfe`).
//!
//! Ele consolida as informações do arquivo físico do XML de cancelamento em estruturas
//! tabulares intermediárias (`InfoNfeCancel`) para posterior compilação de relatórios e planilhas.

use crate::{
    Arguments, GetKey, InfoExtension, Information, StructExtension,
    xml_structs::cancelamento::{CancelExt, Cancelamento, Retencao},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Estrutura consolidada para exportação tabular de dados de cancelamento de NF-e.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoNfeCancel {
    /// Chave de acesso correspondente à NF-e cancelada.
    #[serde(rename = "nfe", default)]
    pub nfe: Option<String>,

    /// Data em que o cancelamento foi efetivado e homologado pela SEFAZ.
    #[serde(rename = "dh_recebimento", default)]
    pub dh_recebimento: Option<NaiveDate>,

    /// Indicador se a NF-e correspondente está cancelada.
    pub cancelado: bool,
}

impl InfoExtension for InfoNfeCancel {}

impl GetKey for InfoNfeCancel {
    /// Implementação auxiliar de chave de acesso para agrupamentos sequenciais ou paralelos.
    fn get_chave(&self) -> Option<String> {
        self.nfe.clone()
    }
}

/// Representação estrutural direta do nó XML de processamento de cancelamento de NF-e (`<procCancNFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcCancNfe {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do layout do protocolo de cancelamento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente ao protocolo de cancelamento.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Bloco contendo os detalhes do pedido de cancelamento da NF-e.
    #[serde(rename = "cancNFe", default)]
    pub canc_nfe: Option<Cancelamento>,

    /// Bloco contendo os detalhes do retorno de homologação de cancelamento.
    #[serde(rename = "retCancNFe", default)]
    pub ret_canc_nfe: Option<Retencao>,
}

impl ProcCancNfe {
    /// Extrai a chave de acesso da NF-e cancelada associada a este protocolo.
    pub fn get_nfe(&self) -> Option<String> {
        self.canc_nfe.get_chave_cancelada_nfe()
    }

    /// Extrai a data de homologação do recebimento do cancelamento.
    pub fn get_dh_recebimento(&self) -> Option<NaiveDate> {
        self.canc_nfe.get_dh_recebimento()
    }

    /// Verifica se as informações de cancelamento estão fisicamente presentes no nó.
    pub fn informacao_de_cancelamento(&self) -> bool {
        self.get_nfe().is_some()
    }

    /// Compila os metadados do XML na estrutura consolidada `InfoNfeCancel`.
    pub fn get_info(&self) -> InfoNfeCancel {
        InfoNfeCancel {
            nfe: self.get_nfe(),
            dh_recebimento: self.get_dh_recebimento(),
            cancelado: self.informacao_de_cancelamento(),
        }
    }
}

impl StructExtension for ProcCancNfe {
    /// Converte os nós desserializados do XML de cancelamento de NF-e para informações consolidadas.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("proc canc nfe xml_path: {xml_path:?}");
            println!("proc_canc_nfe: {self:#?}\n");
        }
        Information::CancelamentoNfe(Box::new(self.get_info()))
    }
}
