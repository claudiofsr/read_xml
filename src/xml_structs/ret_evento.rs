//! # Retorno de Registro de Eventos (NF-e/CT-e)
//!
//! Este módulo gerencia os esquemas de desserialização correspondentes ao protocolo
//! de retorno de registro de eventos fiscais da SEFAZ (`<retEvento>` e `<retEventoCTe>`).
//!
//! Eventos representam alterações ou registros subsequentes de um documento fiscal eletrônico,
//! tais como Carta de Correção Eletrônica (CC-e), Cancelamento ou Manifestação do Destinatário.

use super::assinaturas::Signature;
use serde::{Deserialize, Serialize};

/// Estrutura correspondente ao protocolo de resposta do lote de eventos fiscais (`<retEvento>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetEvento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do layout correspondente ao lote de retorno.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Bloco de informações detalhadas do retorno de registro do evento.
    #[serde(rename = "infEvento", default)]
    pub inf_evento: Option<RetInfEvento>,

    /// Bloco contendo a assinatura digital da SEFAZ correspondente ao protocolo.
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

/// Bloco contendo os dados de processamento do evento pela SEFAZ (`<infEvento>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetInfEvento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Identificador do nó gerado para fins de assinatura e indexação.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Código do órgão de recepção do evento (p. ex., código do Estado/UF).
    #[serde(rename = "cOrgao", default)]
    pub c_orgao: Option<String>,

    /// Código do órgão autorizador do evento se aplicável.
    #[serde(rename = "cOrgaoAutor", default)]
    pub c_orgao_autor: Option<String>,

    /// Código correspondente ao status da mensagem de retorno enviada pela SEFAZ.
    #[serde(rename = "cStat", default)]
    pub c_stat: Option<String>,

    /// Chave de acesso correspondente ao CT-e afetado pelo evento.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Chave de acesso correspondente à NF-e afetada pelo evento.
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,

    /// CNPJ do Destinatário associado ao documento fiscal.
    #[serde(rename = "CNPJDest", default)]
    pub cnpjdest: Option<String>,

    /// Data e hora de registro oficial do evento no banco de dados governamental.
    #[serde(rename = "dhRegEvento", default)]
    pub dh_reg_evento: Option<String>,

    /// Número sequencial do protocolo gerado pela autoridade fiscalizadora.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Número sequencial do evento para a chave de acesso correspondente (p. ex., CC-e nº 1, 2, etc.).
    #[serde(rename = "nSeqEvento", default)]
    pub n_seq_evento: Option<String>,

    /// Identificação do Ambiente de destino: 1 - Produção; 2 - Homologação.
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// Tipo de evento registrado (p. ex., "110111" para cancelamento).
    #[serde(rename = "tpEvento", default)]
    pub tp_evento: Option<String>,

    /// Versão do aplicativo que processou e autorizou o evento.
    #[serde(rename = "verAplic", default)]
    pub ver_aplic: Option<String>,

    /// Descrição textual literal correspondente ao status do retorno fiscal.
    #[serde(rename = "xMotivo", default)]
    pub x_motivo: Option<String>,

    /// Descrição literal correspondente à ação de evento solicitada.
    #[serde(rename = "xEvento", default)]
    pub x_evento: Option<String>,
}

impl RetInfEvento {
    /// Determina se o evento fiscal foi registrado e autorizado com sucesso pela SEFAZ.
    ///
    /// No regulamento nacional de emissão de NF-e/CT-e, os códigos de sucesso de eventos são:
    /// * `"135"`: Evento registrado e vinculado ao documento correspondente.
    /// * `"136"`: Evento registrado, mas o vínculo não pôde ser completado imediatamente.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.c_stat
            .as_deref()
            .is_some_and(|stat| stat == "135" || stat == "136")
    }

    /// Retorna uma referência à chave de acesso do documento afetado (NF-e ou CT-e).
    ///
    /// Retorna `None` se ambas as chaves estiverem ausentes. Esta operação é executada
    /// em tempo constante de forma eficiente, sem causar alocações adicionais na Heap.
    #[inline]
    pub fn get_chave_documento(&self) -> Option<&str> {
        self.ch_nfe.as_deref().or(self.ch_cte.as_deref())
    }
}
