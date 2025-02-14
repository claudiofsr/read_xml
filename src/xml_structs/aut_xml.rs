use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AutXML {
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

/// Dados do protocolo de status
///
/// Chaves de acesso da NF-e/CT-e, compostas por:
///
/// UF do emitente, AAMM da emissão da NFe, CNPJ do emitente,
/// modelo, série e número da NF-e e código numérico+DV.
///
/// <https://dfe-portal.svrs.rs.gov.br/NFE/ConsultaSchema>
#[derive(Debug, Serialize, Deserialize)]
pub struct InfProtocolo {
    /// Chaves de acesso da CT-e
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    /// Chaves de acesso da NF-e
    #[serde(rename = "chNFe")]
    pub ch_nfe: Option<String>,
    /// Código do status da mensagem enviada.
    #[serde(rename = "cStat")]
    pub c_stat: Option<String>,
    /// Data e hora de processamento, no formato AAAA-MM-DDTHH:MM:SSTZD.
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    /// Digest Value da NF-e processada.
    #[serde(rename = "digVal")]
    pub dig_val: Option<String>,
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    /// Número do Protocolo de Status da NF-e.
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    /// Identificação do Ambiente: 1 - Produção ; 2 - Homologação
    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>,
    /// Versão do Aplicativo que processou a NF-e
    #[serde(rename = "verAplic")]
    pub ver_aplic: Option<String>,
    /// Descrição literal do status do serviço solicitado.
    #[serde(rename = "xMotivo")]
    pub x_motivo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfRespTec {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "idCSRT")]
    pub id_csrt: Option<String>,
    #[serde(rename = "hashCSRT")]
    pub hash_csrt: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xContato")]
    pub x_contato: Option<String>,
}
