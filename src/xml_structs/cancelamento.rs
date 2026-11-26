//! # Tratamento de Cancelamento de Documentos Fiscais (NF-e e CT-e)
//!
//! Este módulo gerencia a desserialização e o tratamento de dados relativos aos processos
//! de cancelamento homologados pela SEFAZ, mapeando as estruturas de pedido de cancelamento
//! (`Cancelamento`), retorno homologado (`Retencao`) e dados da transação (`InfoCancelamento`).

use crate::{OptExt, get_naive_date_from_yyyy_mm_dd, xml_structs::assinaturas::Signature};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Extensão de utilidades para consulta rápida a dados de documentos cancelados.
///
/// Este trait permite extrair diretamente dados de chaves de acesso e datas de recebimento
/// a partir de encapsuladores, simplificando fluxos condicionais de tratamento de erro.
pub trait CancelExt {
    /// Obtém de forma sanitizada a chave de acesso do CT-e cancelado, se presente.
    fn get_chave_cancelada_cte(&self) -> Option<String>;

    /// Obtém de forma sanitizada a chave de acesso da NF-e cancelada, se presente.
    fn get_chave_cancelada_nfe(&self) -> Option<String>;

    /// Converte e obtém a data em que o protocolo de cancelamento foi homologado pela SEFAZ.
    fn get_dh_recebimento(&self) -> Option<NaiveDate>;
}

impl CancelExt for Option<Cancelamento> {
    #[inline]
    fn get_chave_cancelada_cte(&self) -> Option<String> {
        self.as_ref()
            .and_then(|cancelamento| cancelamento.inf_canc.ch_cte.get_key())
    }

    #[inline]
    fn get_chave_cancelada_nfe(&self) -> Option<String> {
        self.as_ref()
            .and_then(|cancelamento| cancelamento.inf_canc.ch_nfe.get_key())
    }

    #[inline]
    fn get_dh_recebimento(&self) -> Option<NaiveDate> {
        self.as_ref()
            .and_then(|cancelamento| cancelamento.inf_canc.get_dh_recbto())
    }
}

/// Estrutura correspondente ao evento ou pedido de Cancelamento de documento fiscal (`<cancNFe>` / `<cancCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Cancelamento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Namespace XML correspondente ao documento de cancelamento.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    /// Versão do layout correspondente ao documento de cancelamento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Dados consolidados do pedido de cancelamento homologado.
    #[serde(rename = "infCanc")]
    pub inf_canc: InfoCancelamento,

    /// Assinatura digital do emitente vinculada à transação de cancelamento.
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

/// Estrutura de Retorno homologado de cancelamento enviado pela SEFAZ (`<retCancNFe>` / `<retCancCTe>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Retencao {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Namespace XML do retorno de cancelamento.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    /// Versão do leiaute do retorno de cancelamento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Bloco com as informações consolidadas da homologação de cancelamento.
    #[serde(rename = "infCanc")]
    pub inf_canc: InfoCancelamento,
}

/// Bloco de Informações do Cancelamento Homologado (`<infCanc>`).
///
/// Contém o detalhamento dos dados de autoria, justificativa, chaves de acesso envolvidas
/// e o respectivo status retornado pela SEFAZ.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfoCancelamento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Identificador único gerado para o lote de cancelamento.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Código do status retornado do processamento (p. ex., "101" para Cancelamento Homologado).
    #[serde(rename = "cStat", default)]
    pub c_stat: Option<String>,

    /// Código correspondente à Unidade Federativa (UF) autorizadora.
    #[serde(rename = "cUF", default)]
    pub c_uf: Option<String>,

    /// Chave de acesso correspondente ao CT-e cancelado, se aplicável.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Chave de acesso correspondente à NF-e cancelada, se aplicável.
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,

    /// Data e hora de processamento e recebimento do cancelamento.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número do protocolo gerado pela SEFAZ que atesta a homologação.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Identificação do Ambiente: 1 - Produção; 2 - Homologação.
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// Versão do aplicativo governamental que processou a transação.
    #[serde(rename = "verAplic", default)]
    pub ver_aplic: Option<String>,

    /// Justificativa descritiva informada para o cancelamento do documento.
    #[serde(rename = "xJust", default)]
    pub x_just: Option<String>,

    /// Motivo detalhado correspondente ao status de processamento retornado.
    #[serde(rename = "xMotivo", default)]
    pub x_motivo: Option<String>,

    /// Descrição do serviço solicitado (p. ex., "CANCELAR").
    #[serde(rename = "xServ", default)]
    pub x_serv: Option<String>,
}

impl InfoCancelamento {
    /// Converte de forma segura a string de recebimento (`dhRecbto`) para `NaiveDate`.
    #[inline]
    pub fn get_dh_recbto(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_recbto)
    }
}
