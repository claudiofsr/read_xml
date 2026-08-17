//! # Mapeamento de Autorizações, Protocolos e Responsabilidade Técnica
//!
//! Este módulo contém estruturas auxiliares comuns compartilhadas entre os leiautes
//! de Nota Fiscal Eletrônica (NF-e) e Conhecimento de Transporte Eletrônico (CT-e).
//!
//! As estruturas mapeadas aqui gerenciam permissões de download de XML (`AutXML`),
//! metadados de protocolo de status e recepção (`InfProtocolo`) e informações
//! cadastrais de responsabilidade técnica (`InfRespTec`).

use serde::{Deserialize, Serialize};

/// Pessoas autorizadas a acessar o arquivo XML da NF-e ou CT-e (`<autXML>`).
///
/// Permite que o emitente autorize CNPJs ou CPFs adicionais a realizarem o download
/// do arquivo digital do documento fiscal nos portais nacionais da SEFAZ.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutXML {
    /// CPF da pessoa autorizada.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// CNPJ da pessoa autorizada.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados do protocolo de status e autorização de uso (`<infProt>`).
///
/// Contém as informações de processamento da SEFAZ relativas à validação,
/// assinatura e recepção final da NF-e ou CT-e.
///
/// Para maiores detalhes, consulte:
/// <https://dfe-portal.svrs.rs.gov.br/NFE/ConsultaSchema>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfProtocolo {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Identificador único do protocolo gerado pela SEFAZ.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Chave de acesso correspondente ao Conhecimento de Transporte Eletrônico (CT-e).
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Chave de acesso correspondente à Nota Fiscal Eletrônica (NF-e).
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,

    /// Código correspondente ao status da mensagem enviada (p. ex., "100" para Autorizado).
    #[serde(rename = "cStat", default)]
    pub c_stat: Option<String>,

    /// Data e hora de processamento do documento, no formato AAAA-MM-DDTHH:MM:SSTZD.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Assinatura digital gerada a partir dos dados da NF-e/CT-e processada.
    #[serde(rename = "digVal", default)]
    pub dig_val: Option<String>,

    /// Número sequencial atribuído ao protocolo de status da NF-e/CT-e.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Identificação do Ambiente de destino: 1 - Produção; 2 - Homologação.
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// Versão do aplicativo governamental que realizou o processamento do documento.
    #[serde(rename = "verAplic", default)]
    pub ver_aplic: Option<String>,

    /// Descrição literal detalhada do status retornado pelo serviço.
    #[serde(rename = "xMotivo", default)]
    pub x_motivo: Option<String>,
}

/// Dados do Responsável Técnico pelo software emissor (`<infRespTec>`).
///
/// Grupo de informações exigido pelas SEFAZs estaduais para identificar a empresa
/// desenvolvedora do sistema que gerou o arquivo XML da NF-e ou CT-e.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfRespTec {
    /// CNPJ da empresa desenvolvedora do sistema emissor de documentos fiscais.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Endereço de correio eletrônico do departamento de suporte ou contato técnico.
    #[serde(rename = "email", default)]
    pub email: Option<String>,

    /// Telefone cadastrado para contato técnico direto.
    #[serde(rename = "fone", default)]
    pub fone: Option<String>,

    /// Identificador do Código de Segurança do Responsável Técnico (CSRT) se cadastrado.
    #[serde(rename = "idCSRT", default)]
    pub id_csrt: Option<String>,

    /// Hash gerado a partir do CSRT associado a esta empresa desenvolvedora.
    #[serde(rename = "hashCSRT", default)]
    pub hash_csrt: Option<String>,

    /// Conteúdo descritivo em texto livre contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Nome descritivo da pessoa física para contato técnico de suporte.
    #[serde(rename = "xContato", default)]
    pub x_contato: Option<String>,
}
