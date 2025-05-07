use crate::{
    Arguments, Information, StructExtension,
    excel::InfoExtension,
    group_by_hashmap::GetKey,
    xml_structs::cancelamento::{CancelExt, Cancelamento, Retencao},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoNfeCancel {
    pub nfe: Option<String>,
    pub dh_recebimento: Option<NaiveDate>,
    pub cancelado: bool,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoNfeCancel {}

/// Ver src/group_by_hashmap.rs
impl GetKey for InfoNfeCancel {
    fn get_chave(&self) -> Option<String> {
        self.nfe.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcCancNfe {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "cancNFe")]
    pub canc_nfe: Option<Cancelamento>,
    #[serde(rename = "retCancNFe")]
    pub ret_canc_nfe: Option<Retencao>,
}

impl ProcCancNfe {
    pub fn get_nfe(&self) -> Option<String> {
        self.canc_nfe.get_chave_cancelada_nfe()
    }

    pub fn get_dh_recebimento(&self) -> Option<NaiveDate> {
        self.canc_nfe.get_dh_recebimento()
    }

    pub fn informacao_de_cancelamento(&self) -> bool {
        self.get_nfe().is_some()
    }

    pub fn get_info(&self) -> InfoNfeCancel {
        InfoNfeCancel {
            nfe: self.get_nfe(),
            dh_recebimento: self.get_dh_recebimento(),
            cancelado: self.informacao_de_cancelamento(),
        }
    }
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcCancNfe {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("proc canc nfe xml_path: {xml_path:?}");
            println!("proc_canc_nfe: {self:#?}\n");
        }
        Information::CancelamentoNfe(Box::new(self.get_info()))
    }
}
