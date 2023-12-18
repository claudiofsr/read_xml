/**
Generates XML file from Multiple XML schemas (xsd files).

This feature requires Apache XMLBeans.
https://xmlbeans.apache.org/docs/2.0.0/guide/tools.html#xsd2inst
xsd2inst (Schema to Instance Tool)
Prints an XML instance from the specified global element using the specified schema.

XSD to XML:
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst procEventoCancNFe_v1.00.xsd -name procEventoNFe -dl > procEventoCancNFe_v1.00.xml

Converter XML file to Rust struct:
read_xml -s procEventoCancNFe_v1.00.xml > procEventoCancNFe_v1.00.rs

https://dfe-portal.svrs.rs.gov.br/MDFE/ConsultaSchema
https://dfe-portal.svrs.rs.gov.br/Nfe

No Manjaro Linux, install qxmledit:
yay -S qxmledit
Make PDF files from XSD.
*/

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
    REGEX_CANCELAMENTO,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoNfeEvento {
    pub nfe: Option<String>,
    pub dh_emi: Option<NaiveDate>,
    pub cancelado: bool,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoNfeEvento {}

/// Schema XML de validação do processo de Cancelamento
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcEventoNfe {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "evento")]
    pub evento: Evento, // Tipo Evento
    #[serde(rename = "retEvento")]
    pub ret_evento: RetEvento, // Tipo Retorno de Lote de Envio
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcEventoNfe {}

impl ProcEventoNfe {
    pub fn get_nfe(&self) -> Option<String> {
        self
            .evento
            .inf_evento
            .ch_nfe
            .as_ref()
            .map(|s| s.remove_non_digits())
    }

    pub fn informacao_de_cancelamento(&self) -> bool {
        REGEX_CANCELAMENTO.is_match(
            &self
                .evento
                .inf_evento
                .det_evento
                .desc_evento
        )
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self
            .evento.inf_evento
            .get_dh_evento()
    }

    pub fn get_info(&self) -> InfoNfeEvento {
        InfoNfeEvento {
            nfe: self.get_nfe(),
            dh_emi: self.get_data_emissao(),
            cancelado: self.informacao_de_cancelamento(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evento {
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
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
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "chNFe")]
    pub ch_nfe: Option<String>,
    #[serde(rename = "dhEvento")]
    pub dh_evento: Option<String>,
    #[serde(rename = "tpEvento")]
    pub tp_evento: String,
    #[serde(rename = "nSeqEvento")]
    pub n_seq_evento: String,
    #[serde(rename = "verEvento")]
    pub ver_evento: String,
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "detEvento")]
    pub det_evento: DetEvento,
}

impl InfEvento {
    pub fn get_dh_evento(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_evento)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetEvento {
     #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "descEvento")]
    pub desc_evento: String, // Descrição do Evento - “Cancelamento”
    #[serde(rename = "cOrgaoAutor")]
    pub c_orgao_autor: Option<String>,
    #[serde(rename = "CTe")]
    pub cte: Option<Cte>,
    #[serde(rename = "emit")]
    pub emit: Option<Emitente>,
    #[serde(rename = "tpAutor")]
    pub tp_autor: Option<String>,
    #[serde(rename = "MDFe")]
    pub mdfe: Option<Mdfe>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "xCorrecao")]
    pub x_correcao: Option<String>,
    #[serde(rename = "xCondUso")]
    pub x_cond_uso: Option<String>,
    #[serde(rename = "xJust")]
    pub x_just: Option<String>,
    #[serde(rename = "verAplic")]
    pub ver_aplic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cte {
    #[serde(rename = "chCTe")]
    pub ch_cte: String,
    pub modal: Option<String>,
    #[serde(rename = "dhEmi")]
    pub dh_emi: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "nProtCanc")]
    pub n_prot_canc: Option<String>,
    #[serde(rename = "nProtCTe")]
    pub n_prot_cte: Option<String>,
    #[serde(rename = "dhEntrega")]
    pub dh_entrega: Option<String>,
    #[serde(rename = "nDoc")]
    pub n_doc:Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "hashEntregaCTe")]
    pub hash_entrega_cte: Option<String>,
    #[serde(rename = "dhHashEntregaCTe")]
    pub dh_hash_entrega_cte: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mdfe {
    #[serde(rename = "chMDFe")]
    pub ch_mdfe: String,
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    pub modal: Option<String>,
    #[serde(rename = "dhEmi")]
    pub dh_emi: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "nProtCanc")]
    pub n_prot_canc: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
}

