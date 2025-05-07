use chrono::NaiveDate;
use claudiofsr_lib::BTreeSetExtension;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    Arguments, Information, OptExt, StructExtension,
    excel::InfoExtension,
    get_naive_date_from_yyyy_mm_dd,
    group_by_hashmap::GetKey,
    xml_structs::{agente::Agente, assinaturas::Signature, ret_evento::RetEvento},
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoCteEvento {
    pub cte: Option<String>,
    pub dh_emi: Option<NaiveDate>,
    pub cancelado: bool,
    pub cte_complementar: Option<String>,
    pub cte_redespacho: Option<String>,
    pub cte_redespacho_interno: Option<String>,
    pub cte_subcontratacao: Option<String>,
    pub cte_substituido: Option<String>,
    pub cte_vinculado: Option<String>,
}

impl InfoCteEvento {
    /// CTes: `[
    /// cte_complementar, cte_redespacho,
    /// cte_redespacho_interno, cte_subcontratacao,
    /// cte_substituido, cte_vinculado
    /// ]`
    pub fn get_ctes(&self) -> Vec<String> {
        let ctes: BTreeSet<String> = [
            &self.cte_complementar,
            &self.cte_redespacho,
            &self.cte_redespacho_interno,
            &self.cte_subcontratacao,
            &self.cte_substituido,
            &self.cte_vinculado,
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect();

        ctes.to_vec()
    }
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCteEvento {}

/// Ver src/group_by_hashmap.rs
impl GetKey for InfoCteEvento {
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

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
    pub ret_evento_cte: RetEvento, // exemplo: Retorno de Lote de Envio
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcEventoCte {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("evento cte xml_path: {xml_path:?}");
            println!("proc_evento_cte: {self:#?}\n");
        }
        Information::EventoCte(Box::new(self.get_info()))
    }
}

impl ProcEventoCte {
    pub fn get_cte(&self) -> Option<String> {
        self.evento_cte.inf_evento.ch_cte.get_key()
    }

    pub fn informacao_de_cancelamento(&self) -> bool {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_canc_cte // ev_canc_cte: Option<EvCancCte>, // Evento de Cancelamento
            .is_some()
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.evento_cte.inf_evento.get_dh_evento()
    }

    pub fn get_cte_complementar(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_complementar
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_compl())
    }

    pub fn get_cte_redespacho(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_redespacho
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_redesp())
    }

    pub fn get_cte_redespacho_interno(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_redespacho_inter
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_redesp_inter())
    }

    pub fn get_cte_subcontratacao(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_subcontratacao
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_subcon())
    }

    pub fn get_cte_substituido(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_substituido
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_substituicao())
    }

    pub fn get_cte_vinculado(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_multimodal
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_vinculado())
    }

    pub fn get_info(&self) -> InfoCteEvento {
        InfoCteEvento {
            cte: self.get_cte(),
            dh_emi: self.get_data_emissao(),
            cancelado: self.informacao_de_cancelamento(),
            cte_complementar: self.get_cte_complementar(),
            cte_redespacho: self.get_cte_redespacho(),
            cte_redespacho_interno: self.get_cte_redespacho_interno(),
            cte_subcontratacao: self.get_cte_subcontratacao(),
            cte_substituido: self.get_cte_substituido(),
            cte_vinculado: self.get_cte_vinculado(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evento {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "infEvento")]
    pub inf_evento: InfEvento,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfEvento {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "cOrgao")]
    pub c_orgao: Option<String>,
    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    #[serde(rename = "dhEvento")]
    pub dh_evento: Option<String>,
    #[serde(rename = "tpEvento")]
    pub tp_evento: Option<String>,
    #[serde(rename = "nSeqEvento")]
    pub n_seq_evento: Option<String>,
    #[serde(rename = "detEvento")]
    pub det_evento: DetEvento,
    #[serde(rename = "verEvento")]
    pub ver_evento: Option<String>,
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
    #[serde(rename = "evCancCTe")]
    pub ev_canc_cte: Option<EvCancCte>, // Evento de
    #[serde(rename = "evCTeComplementar")]
    pub ev_cte_complementar: Option<EvCteComplementar>,
    #[serde(rename = "evCTeRedespacho")]
    pub ev_cte_redespacho: Option<EvCteRedespacho>,
    #[serde(rename = "evCTeRedespachoInter")]
    pub ev_cte_redespacho_inter: Option<EvCteRedespachoInter>,
    #[serde(rename = "evCTeRegPassagemAuto")]
    pub ev_cte_reg_passagem_auto: Option<EvCteRegPassagemAuto>,
    #[serde(rename = "evCTeSubcontratacao")]
    pub ev_cte_subcontratacao: Option<EvCteSubcontratacao>,
    #[serde(rename = "evCTeSubstituido")]
    pub ev_cte_substituido: Option<EvCteSubstituido>,
    #[serde(rename = "evCTeMultimodal")]
    pub ev_cte_multimodal: Option<EvCteMultimodal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCceCte {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "xCondUso")]
    pub x_cond_uso: Option<String>,
    #[serde(rename = "infCorrecao")]
    pub inf_correcao: Vec<InfCorrecao>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCorrecao {
    #[serde(rename = "grupoAlterado")]
    pub grupo_alterado: Option<String>,
    #[serde(rename = "campoAlterado")]
    pub campo_alterado: Option<String>,
    #[serde(rename = "valorAlterado")]
    pub valor_alterado: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteAutorizadoMdfe {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "MDFe")]
    pub mdfe: Mdfe,
    #[serde(rename = "emit")]
    pub emit: Option<Agente>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mdfe {
    #[serde(rename = "chMDFe")]
    pub ch_mdfe: Option<String>,
    pub modal: Option<String>,
    #[serde(rename = "dhEmi")]
    pub dh_emi: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRedespacho {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeRedesp")]
    pub ch_cte_redesp: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
}

impl EvCteRedespacho {
    pub fn get_ch_cte_redesp(&self) -> Option<String> {
        self.ch_cte_redesp.get_key()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRedespachoInter {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeRedespInter")]
    pub ch_cte_redesp_inter: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
}

impl EvCteRedespachoInter {
    pub fn get_ch_cte_redesp_inter(&self) -> Option<String> {
        self.ch_cte_redesp_inter.get_key()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRegPassagemAuto {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "tpTransm")]
    pub tp_transm: Option<String>,
    #[serde(rename = "infPass")]
    pub inf_pass: Option<InfPass>,
    #[serde(rename = "chMDFe")]
    pub ch_mdfe: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfPass {
    #[serde(rename = "cUFTransito")]
    pub c_uftransito: Option<String>,
    #[serde(rename = "cIdEquip")]
    pub c_id_equip: Option<String>,
    #[serde(rename = "xIdEquip")]
    pub x_id_equip: Option<String>,
    #[serde(rename = "tpEquip")]
    pub tp_equip: Option<String>,
    pub placa: Option<String>,
    #[serde(rename = "tpSentido")]
    pub tp_sentido: Option<String>,
    #[serde(rename = "dhPass")]
    pub dh_pass: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    #[serde(rename = "NSU")]
    pub nsu: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteComplementar {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeCompl")]
    pub ch_cte_compl: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
}

impl EvCteComplementar {
    pub fn get_ch_cte_compl(&self) -> Option<String> {
        self.ch_cte_compl.get_key()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteSubcontratacao {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeSubcon")]
    pub ch_cte_subcon: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
}

impl EvCteSubcontratacao {
    pub fn get_ch_cte_subcon(&self) -> Option<String> {
        self.ch_cte_subcon.get_key()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteSubstituido {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeSubstituicao")]
    pub ch_cte_substituicao: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
}

impl EvCteSubstituido {
    pub fn get_ch_cte_substituicao(&self) -> Option<String> {
        self.ch_cte_substituicao.get_key()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteMultimodal {
    #[serde(rename = "descEvento")]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeVinculado")]
    pub ch_cte_vinculado: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
}

impl EvCteMultimodal {
    pub fn get_ch_cte_vinculado(&self) -> Option<String> {
        self.ch_cte_vinculado.get_key()
    }
}
