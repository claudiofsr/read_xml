use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use serde::{Deserialize, Serialize};

use crate::{
    StructExtension,
    excel::InfoExtension,
    xml_structs::assinaturas::Signature,
    xml_structs::emitente::Emitente,
    xml_structs::ret_evento::RetEvento,
    get_naive_date_from_yyyy_mm_dd,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoCteEvento {
    pub cte: Option<String>,
    pub dh_emi: Option<NaiveDate>,
    pub cancelado: bool,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCteEvento {}

/// Schema XML de validação do processo de Cancelamento
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcEventoCte {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "eventoCTe")]
    pub evento_cte: Evento, // Tipo Evento
    #[serde(rename = "retEventoCTe")]
    pub ret_evento_cte: RetEvento, // Tipo Retorno de Lote de Envio
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcEventoCte {}

impl ProcEventoCte {
    pub fn get_cte(&self) -> Option<String> {
        self
            .evento_cte
            .inf_evento
            .ch_cte
            .as_ref()
            .map(|s| s.remove_non_digits())
    }

    pub fn informacao_de_cancelamento(&self) -> bool {
        self
            .evento_cte
            .inf_evento
            .det_evento
            .ev_canc_cte // ev_canc_cte: Option<EvCancCte>, // Evento de Cancelamento
            .is_some()
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self
            .evento_cte
            .inf_evento
            .get_dh_evento()
    }

    pub fn get_info(&self) -> InfoCteEvento {
        InfoCteEvento {
            cte: self.get_cte(),
            dh_emi: self.get_data_emissao(),
            cancelado: self.informacao_de_cancelamento(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evento {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "infEvento")]
    pub inf_evento: InfEvento,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfEvento {
    #[serde(rename = "cOrgao")]
    pub c_orgao: String,
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    #[serde(rename = "dhEvento")]
    pub dh_evento: Option<String>,
    #[serde(rename = "tpEvento")]
    pub tp_evento: String,
    #[serde(rename = "nSeqEvento")]
    pub n_seq_evento: String,
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "detEvento")]
    pub det_evento: DetEvento,
    #[serde(rename = "verEvento")]
    pub ver_evento: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
}

impl InfEvento {
    pub fn get_dh_evento(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_evento)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetEvento {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@versaoEvento")]
    pub versao_evento: Option<String>,
    #[serde(rename = "evCCeCTe")]
    pub ev_cce_cte: Option<EvCceCte>,
    #[serde(rename = "evCTeAutorizadoMDFe")]
    pub ev_cte_autorizado_mdfe: Option<EvCteAutorizadoMdfe>,
    #[serde(rename = "evCTeRedespachoInter")]
    pub ev_cte_redespacho_inter: Option<EvCteRedespachoInter>,
    #[serde(rename = "evCancCTe")]
    pub ev_canc_cte: Option<EvCancCte>, // Evento de Cancelamento
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCceCte {
    #[serde(rename = "descEvento")]
    pub desc_evento: String,
    #[serde(rename = "xCondUso")]
    pub x_cond_uso: String,
    #[serde(rename = "infCorrecao")]
    pub inf_correcao: Vec<InfCorrecao>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCorrecao {
    #[serde(rename = "grupoAlterado")]
    pub grupo_alterado: String,
    #[serde(rename = "campoAlterado")]
    pub campo_alterado: String,
    #[serde(rename = "valorAlterado")]
    pub valor_alterado: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteAutorizadoMdfe {
    #[serde(rename = "descEvento")]
    pub desc_evento: String,
    #[serde(rename = "MDFe")]
    pub mdfe: Mdfe,
    #[serde(rename = "emit")]
    pub emit: Emitente,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mdfe {
    #[serde(rename = "chMDFe")]
    pub ch_mdfe: String,
    pub modal: String,
    #[serde(rename = "dhEmi")]
    pub dh_emi: String,
    #[serde(rename = "nProt")]
    pub n_prot: String,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRedespachoInter {
    #[serde(rename = "descEvento")]
    pub desc_evento: String,
    #[serde(rename = "chCTeRedespInter")]
    pub ch_cte_redesp_inter: String,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
    #[serde(rename = "nProt")]
    pub n_prot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCancCte {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "xJust")]
    pub x_just: Option<String>,
}

