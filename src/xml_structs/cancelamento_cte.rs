use crate::{
    Arguments, Information, StructExtension,
    excel::InfoExtension,
    group_by_hashmap::GetKey,
    xml_structs::cancelamento::{CancelExt, Cancelamento, Retencao},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoCteCancel {
    pub cte: Option<String>,
    pub dh_recebimento: Option<NaiveDate>,
    pub cancelado: bool,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCteCancel {}

/// Ver src/group_by_hashmap.rs
impl GetKey for InfoCteCancel {
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcCancCte {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "cancCTe")]
    pub canc_cte: Option<Cancelamento>,
    #[serde(rename = "retCancCTe")]
    pub ret_canc_cte: Option<Retencao>,
}

impl ProcCancCte {
    pub fn get_cte(&self) -> Option<String> {
        self.canc_cte.get_chave_cancelada_cte()
    }

    pub fn get_dh_recebimento(&self) -> Option<NaiveDate> {
        self.canc_cte.get_dh_recebimento()
    }

    pub fn informacao_de_cancelamento(&self) -> bool {
        self.get_cte().is_some()
    }

    pub fn get_info(&self) -> InfoCteCancel {
        InfoCteCancel {
            cte: self.get_cte(),
            dh_recebimento: self.get_dh_recebimento(),
            cancelado: self.informacao_de_cancelamento(),
        }
    }
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcCancCte {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("proc canc cte xml_path: {xml_path:?}");
            println!("proc_canc_cte: {self:#?}\n");
        }
        Information::CancelamentoCte(Box::new(self.get_info()))
    }
}
