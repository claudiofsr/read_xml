/**
Generates XML file from Multiple XML schemas (xsd files).

This feature requires Apache XMLBeans.
https://xmlbeans.apache.org/docs/2.0.0/guide/tools.html#xsd2inst
xsd2inst (Schema to Instance Tool)
Prints an XML instance from the specified global element using the specified schema.

XSD to XML:
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst procCTe_v3.00.xsd -name cteProc -dl > procCTe_v3.00.xml

Converter XML file to Rust struct:
read_xml -s procCTe_v3.00.xml > procCTe_v3.00.rs

https://dfe-portal.svrs.rs.gov.br/Cte
https://dfe-portal.svrs.rs.gov.br/CTE/ConsultaSchema

No Manjaro Linux, install qxmledit:
yay -S qxmledit
Make PDF files from XSD.
*/

use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use rust_xlsxwriter::serialize_chrono_option_naive_to_excel;
use serde::{Serialize, Deserialize};
use struct_iterable::Iterable;

use crate::{
    StructExtension,
    excel::InfoExtension,
    xml_structs::assinaturas::{Signature, ProtSignature},
    xml_structs::integrated_dev_env::Ide,
    xml_structs::emitente::Emitente,
    xml_structs::expedidor::Exped,
    xml_structs::destinatario::Destinatario,
    xml_structs::recebedor::Receb,
    xml_structs::remetente::Remetente,
    xml_structs::impostos::Imposto,
    xml_structs::aut_xml::{AutXML, InfProtocolo, InfRespTec},
    serialize_vec_string,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoCte {
    #[serde(rename = "CNPJ do Remetente")]
    remetente_cnpj: Option<String>,
    #[serde(rename = "Nome do Remetente")]
    remetente_nome: Option<String>,
    #[serde(rename = "Municípo do Remetente")]
    remetente_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Remetente")]
    remetente_ender_estado: Option<String>,
    #[serde(rename = "CNPJ do Destinatário")]
    destinatario_cnpj: Option<String>,
    #[serde(rename = "Nome do Destinatário")]
    destinatario_nome: Option<String>,
    #[serde(rename = "Municípo do Destinatário")]
    destinatario_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Destinatário")]
    destinatario_ender_estado: Option<String>,
    #[serde(rename = "Chave do Documento Fiscal")]
    pub cte: Option<String>,
    #[serde(rename = "Registro de Origem")]
    doc_tipo: String,
    #[serde(rename = "Cancelado")]
    pub cancelado: Option<String>,
    #[serde(rename = "Nº do Documento Fiscal")]
    numero_da_nota: Option<u32>,
    #[serde(rename = "CFOP (Código Fiscal de Operações e Prestações)")]
    cfop: Option<u16>,
    #[serde(rename = "Data de Emissão", serialize_with = "serialize_chrono_option_naive_to_excel")]
    dh_emi: Option<NaiveDate>,
    #[serde(rename = "Data de Saída / Entrega", serialize_with = "serialize_chrono_option_naive_to_excel")]
    dh_sai_ent: Option<NaiveDate>,
    #[serde(rename = "CTes Anteriores", serialize_with = "serialize_vec_string")]
    cte_anteriores: Vec<String>,
    #[serde(rename = "NFes Vinculados", serialize_with = "serialize_vec_string")]
    nfes: Vec<String>,
    #[serde(rename = "Valor Total")]
    valor_total: Option<f64>,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCte {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CteProc {
    #[serde(rename = "@dhConexao")]
    pub dh_conexao: Option<String>,
    #[serde(rename = "@ipTransmissor")]
    pub ip_transmissor: Option<String>,
    #[serde(rename = "@nPortaCon")]
    pub n_porta_con: Option<String>,
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CTe")]
    pub cte: Cte,
    #[serde(rename = "protCTe")]
    pub prot_cte: ProtCte,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for CteProc {}

impl CteProc {
    pub fn get_cte(&self) -> Option<String> {
        self
            .prot_cte
            .inf_prot
            .ch_cte
            .as_ref()
            .map(|s| s.remove_non_digits())
    }

    pub fn get_remetente_cnpj(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_rem_cnpj()
            })
    }

    pub fn get_remetente_nome(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_rem_nome()
            })
    }

    pub fn get_remetente_ender_municipio(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_rem_ender_municipio()
            })
    }

    pub fn get_remetente_ender_estado(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_rem_ender_estado()
            })
    }

    pub fn get_destinatario_cnpj(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_cnpj()
            })
    }

    pub fn get_destinatario_nome(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_nome()
            })
    }

    pub fn get_numero_da_nota(&self) -> Option<u32> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|info| {
                info.ide.get_n_ct()
            })
    }

    pub fn get_destinatario_ender_municipio(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_ender_municipio()
            })
    }

    pub fn get_destinatario_ender_estado(&self) -> Option<String> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_ender_estado()
            })
    }

    pub fn get_cfop(&self) -> Option<u16> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|info| {
                info.ide.get_cfop()
            })
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|info| {
                info.ide.get_dh_emi()
            })
    }

    pub fn get_data_saida(&self) -> Option<NaiveDate> {
        self
            .cte
            .inf_cte
            .as_ref()
            .and_then(|info| {
                info.ide.get_dh_sai()
            })
    }

    // How do I avoid unwrap when converting a vector of Options or Results to only the successful values?
    // https://stackoverflow.com/questions/36020110/how-do-i-avoid-unwrap-when-converting-a-vector-of-options-or-results-to-only-the
    // vec.into_iter().flatten().collect()

    pub fn get_cte_anteriores(&self) -> Vec<String> {
        /*
            self
                .cte
                .inf_cte
                .iter()
                .flat_map(|informacao_de_cte| {
                    informacao_de_cte
                        .get_info_de_ctes()
                })
                .collect::<Vec<String>>()
        */

        match self.cte.inf_cte.as_ref() {
            Some(info) => info.get_info_de_ctes(),
            None => Vec::new(),
        }
    }

    pub fn get_nfes(&self) -> Vec<String> {
        match self.cte.inf_cte.as_ref() {
            Some(info) => info.get_info_de_nfes(),
            None => Vec::new(),
        }
    }

    pub fn get_value_total(&self) -> Option<f64> {
        self
            .cte
            .inf_cte
            .as_ref()
            .map(|info| info.v_prest.v_tprest)
    }

    pub fn get_info(&self) -> InfoCte {
        InfoCte {
            remetente_cnpj: self.get_remetente_cnpj(),
            remetente_nome: self.get_remetente_nome(),
            remetente_ender_municipio: self.get_remetente_ender_municipio(),
            remetente_ender_estado: self.get_remetente_ender_estado(),
            destinatario_cnpj: self.get_destinatario_cnpj(),
            destinatario_nome: self.get_destinatario_nome(),
            destinatario_ender_municipio: self.get_destinatario_ender_municipio(),
            destinatario_ender_estado: self.get_destinatario_ender_estado(),
            cte: self.get_cte(),
            doc_tipo: "CTe".to_string(),
            cancelado: None,
            numero_da_nota: self.get_numero_da_nota(),
            cfop: self.get_cfop(),
            dh_emi: self.get_data_emissao(),
            dh_sai_ent: self.get_data_saida(),
            cte_anteriores: self.get_cte_anteriores(),
            nfes: self.get_nfes(),
            valor_total: self.get_value_total(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cte {
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "infCte")]
    pub inf_cte: Option<InfCte>,
    #[serde(rename = "infCTeSupl")]
    pub inf_cte_supl: Option<InfCteSupl>,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteSupl {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "qrCodCTe")]
    pub qr_cod_cte: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCte {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "autXML")]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "compl")]
    pub compl: Option<Compl>,
    #[serde(rename = "dest")]
    pub dest: Option<Destinatario>,
    #[serde(rename = "emit")]
    pub emit: Option<Emitente>,
    #[serde(rename = "exped")]
    pub exped: Option<Exped>,
    #[serde(rename = "ide")]
    pub ide: Ide,
    #[serde(rename = "imp")]
    pub imp: Imposto,
    #[serde(rename = "infCTeNorm")]
    pub inf_cte_norm: Option<InfCteNorm>,
    #[serde(rename = "infCteAnu")]
    pub inf_cte_anu: Option<InfCteAnu>,
    #[serde(rename = "infCteComp")]
    pub inf_cte_comp: Option<InfCteComp>,
    #[serde(rename = "infPAA")]
    pub inf_paa: Option<InfPaa>,
    #[serde(rename = "infRespTec")]
    pub inf_resp_tec: Option<InfRespTec>,
    #[serde(rename = "infSolicNFF")]
    pub inf_solic_nff: Option<InfSolicNff>,
    #[serde(rename = "receb")]
    pub receb: Option<Receb>,
    #[serde(rename = "rem")]
    pub rem: Option<Remetente>,
    #[serde(rename = "vPrest")]
    pub v_prest: VPrest,
}

impl InfCte {

    /// Remetente: CNPJ
    fn get_rem_cnpj(&self) -> Option<String> {
        self
            .rem
            .as_ref()
            .and_then(|remetente| {
                remetente.get_cnpj()
            })
    }

    /// Remetente: None
    fn get_rem_nome(&self) -> Option<String> {
        self
            .rem
            .as_ref()
            .and_then(|remetente| {
                remetente.get_nome()
            })
    }

    /// Remetente: Endereço do Município
    fn get_rem_ender_municipio(&self) -> Option<String> {
        self
            .rem
            .as_ref()
            .and_then(|remetente| {
                remetente.get_endereco_municipio()
            })
    }

    /// Remetente: Endereço do Estado
    fn get_rem_ender_estado(&self) -> Option<String> {
        self
            .rem
            .as_ref()
            .and_then(|remetente| {
                remetente.get_endereco_uf()
            })
    }

    /// Destinatário: CNPJ
    fn get_dest_cnpj(&self) -> Option<String> {
        self
            .dest
            .as_ref()
            .and_then(|destinatario| {
                destinatario.get_cnpj()
            })
    }

    /// Destinatário: Nome
    fn get_dest_nome(&self) -> Option<String> {
        self
            .dest
            .as_ref()
            .and_then(|destinatario| {
                destinatario.get_nome()
            })
    }

    /// Destinatário: Endereço do Município
    fn get_dest_ender_municipio(&self) -> Option<String> {
        self
            .dest
            .as_ref()
            .and_then(|destinatario| {
                destinatario.get_endereco_municipio()
            })
    }

    /// Destinatário: Endereço do Estado
    fn get_dest_ender_estado(&self) -> Option<String> {
        self
            .dest
            .as_ref()
            .and_then(|destinatario| {
                destinatario.get_endereco_uf()
            })
    }

    fn get_info_de_ctes(&self) -> Vec<String> {
        match self.inf_cte_norm.as_ref() {
            Some(info) => info.get_docs_anteriores(),
            None => Vec::new(),
        }
    }

    fn get_info_de_nfes(&self) -> Vec<String> {
        match self.inf_cte_norm.as_ref() {
            Some(info) => info.get_info_de_documentos(),
            None => Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Compl {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Entrega")]
    pub entrega: Option<Entrega>,
    #[serde(rename = "ObsCont")]
    pub obs_cont: Option<Vec<ObsCont>>,
    #[serde(rename = "ObsFisco")]
    pub obs_fisco: Option<ObsFisco>,
    #[serde(rename = "destCalc")]
    pub dest_calc: Option<String>,
    #[serde(rename = "fluxo")]
    pub fluxo: Option<Fluxo>,
    #[serde(rename = "origCalc")]
    pub orig_calc: Option<String>,
    #[serde(rename = "xCaracAd")]
    pub x_carac_ad: Option<String>,
    #[serde(rename = "xCaracSer")]
    pub x_carac_ser: Option<String>,
    #[serde(rename = "xEmi")]
    pub x_emi: Option<String>,
    #[serde(rename = "xObs")]
    pub x_obs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entrega {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "comData")]
    pub com_data: Option<ComData>,
    #[serde(rename = "comHora")]
    pub com_hora: Option<ComHora>,
    #[serde(rename = "noInter")]
    pub no_inter: Option<NoInter>,
    #[serde(rename = "noPeriodo")]
    pub no_periodo: Option<NoPeriodo>,
    #[serde(rename = "semData")]
    pub sem_data: Option<SemData>,
    #[serde(rename = "semHora")]
    pub sem_hora: Option<SemHora>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComData {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dProg")]
    pub d_prog: Option<String>,
    #[serde(rename = "tpPer")]
    pub tp_per: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComHora {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "hProg")]
    pub h_prog: Option<String>,
    #[serde(rename = "tpHor")]
    pub tp_hor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoInter {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "hFim")]
    pub h_fim: Option<String>,
    #[serde(rename = "hIni")]
    pub h_ini: Option<String>,
    #[serde(rename = "tpHor")]
    pub tp_hor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoPeriodo {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dFim")]
    pub d_fim: Option<String>,
    #[serde(rename = "dIni")]
    pub d_ini: Option<String>,
    #[serde(rename = "tpPer")]
    pub tp_per: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemData {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "tpPer")]
    pub tp_per: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemHora {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "tpHor")]
    pub tp_hor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsCont {
    #[serde(rename = "@xCampo")]
    pub x_campo: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xTexto")]
    pub x_texto: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsFisco {
    #[serde(rename = "@xCampo")]
    pub x_campo: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xTexto")]
    pub x_texto: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fluxo {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "pass")]
    pub pass: Pass,
    #[serde(rename = "xDest")]
    pub x_dest: String,
    #[serde(rename = "xOrig")]
    pub x_orig: String,
    #[serde(rename = "xRota")]
    pub x_rota: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pass {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xPass")]
    pub x_pass: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteNorm {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "cobr")]
    pub cobr: Option<Cobr>,
    #[serde(rename = "docAnt")]
    pub doc_ant: Option<DocAnt>,
    #[serde(rename = "infCarga")]
    pub inf_carga: Option<InfCarga>,
    #[serde(rename = "infCteSub")]
    pub inf_cte_sub: Option<InfCteSub>,
    #[serde(rename = "infDoc")]
    pub inf_doc: Option<InfDoc>,
    #[serde(rename = "infGlobalizado")]
    pub inf_globalizado: Option<InfGlobalizado>,
    #[serde(rename = "infModal")]
    pub inf_modal: Option<InfModal>,
    #[serde(rename = "infServVinc")]
    pub inf_serv_vinc: Option<InfServVinc>,
    #[serde(rename = "veicNovos")]
    pub veic_novos: Option<VeicNovos>,
}

impl InfCteNorm {
    fn get_docs_anteriores(&self) -> Vec<String> {
        match self.doc_ant.as_ref() {
            Some(doc) => doc.get_emissao_de_docs_anteriores(),
            None => Vec::new(),
        }
    }

    fn get_info_de_documentos(&self) -> Vec<String> {
        match self.inf_doc.as_ref() {
            Some(info) => info.get_chaves_de_nfes(),
            None => Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cobr {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dup")]
    pub dup: Option<Dup>,
    #[serde(rename = "fat")]
    pub fat: Option<Fat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dup {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dVenc")]
    pub d_venc: Option<String>,
    #[serde(rename = "nDup")]
    pub n_dup: Option<String>,
    #[serde(rename = "vDup")]
    pub v_dup: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fat {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nFat")]
    pub n_fat: Option<String>,
    #[serde(rename = "vDesc")]
    pub v_desc: Option<String>,
    #[serde(rename = "vLiq")]
    pub v_liq: Option<String>,
    #[serde(rename = "vOrig")]
    pub v_orig: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocAnt {
    #[serde(rename = "emiDocAnt")]
    pub emi_doc_ant: Vec<EmiDocAnt>,
}

impl DocAnt {
    fn get_emissao_de_docs_anteriores(&self) -> Vec<String> {
        self
            .emi_doc_ant
            .iter()
            .flat_map(|emissao| {
                emissao
                    .id_doc_ant
                    .get_docs_anteriores_eletronicos()
            })
            .collect::<Vec<String>>()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmiDocAnt {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "idDocAnt")]
    pub id_doc_ant: IdDocAnt,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdDocAnt {
    #[serde(rename = "idDocAntEle")]
    pub id_doc_ant_ele: Option<Vec<IdDocAntEle>>,
    #[serde(rename = "idDocAntPap")]
    pub id_doc_ant_pap: Option<Vec<IdDocAntPap>>,
}

impl IdDocAnt {
    // <xs:element name="idDocAntEle" maxOccurs="unbounded">
    // <xs:documentation>Documentos de transporte anterior eletrônicos</xs:documentation>
    fn get_docs_anteriores_eletronicos(&self) -> Vec<String> {
        match self.id_doc_ant_ele.as_ref() {
            Some(vec_id_doc_anterior) => {
                vec_id_doc_anterior
                    .iter()
                    .map(|i| i.ch_cte.to_string())
                    .collect::<Vec<String>>()
            },
            None => Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdDocAntEle {
    #[serde(rename = "chCTe")]
    pub ch_cte: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdDocAntPap {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dEmi")]
    pub d_emi: Option<String>,
    #[serde(rename = "nDoc")]
    pub n_doc: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "subser")]
    pub subser: Option<String>,
    #[serde(rename = "tpDoc")]
    pub tp_doc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infQ")]
    pub inf_q: Option<Vec<InfQ>>,
    #[serde(rename = "proPred")]
    pub pro_pred: String,
    #[serde(rename = "vCarga")]
    pub v_carga: String,
    #[serde(rename = "vCargaAverb")]
    pub v_carga_averb: String,
    #[serde(rename = "xOutCat")]
    pub x_out_cat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfQ {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "cUnid")]
    pub c_unid: String,
    #[serde(rename = "qCarga")]
    pub q_carga: String,
    #[serde(rename = "tpMed")]
    pub tp_med: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteSub {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "chCte")]
    pub ch_cte: String,
    #[serde(rename = "indAlteraToma")]
    pub ind_altera_toma: String,
    #[serde(rename = "refCteAnu")]
    pub ref_cte_anu: String,
    #[serde(rename = "tomaICMS")]
    pub toma_icms: TomaIcms,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TomaIcms {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "refCte")]
    pub ref_cte: String,
    #[serde(rename = "refNF")]
    pub ref_nf: RefNf,
    #[serde(rename = "refNFe")]
    pub ref_nfe: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefNf {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    #[serde(rename = "CPF")]
    pub cpf: String,
    #[serde(rename = "dEmi")]
    pub d_emi: String,
    #[serde(rename = "mod")]
    pub modelo: String,
    #[serde(rename = "nro")]
    pub nro: String,
    #[serde(rename = "serie")]
    pub serie: String,
    #[serde(rename = "subserie")]
    pub subserie: String,
    #[serde(rename = "valor")]
    pub valor: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDoc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infNF")]
    pub inf_nf: Option<InfNf>,
    #[serde(rename = "infNFe")]
    pub inf_nfe: Option<Vec<InfNfe>>,
    #[serde(rename = "infOutros")]
    pub inf_outros: Option<InfOutros>,
}

impl InfDoc {
    fn get_chaves_de_nfes(&self) -> Vec<String> {
        match self.inf_nfe.as_ref() {
            Some(vec_info_nfe) => {
                vec_info_nfe
                    .iter()
                    .map(|i| i.chave.to_string())
                    .collect::<Vec<String>>()
            },
            None => Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNf {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PIN")]
    pub pin: Option<String>,
    #[serde(rename = "dEmi")]
    pub d_emi: Option<String>,
    #[serde(rename = "dPrev")]
    pub d_prev: Option<String>,
    #[serde(rename = "infUnidCarga")]
    pub inf_unid_carga: Option<InfDocCteInfNfCteInfUnidCarga>,
    #[serde(rename = "infUnidTransp")]
    pub inf_unid_transp: Option<InfNfCteInfUnidTransp>,
    #[serde(rename = "mod")]
    pub modelo: Option<String>,
    #[serde(rename = "nCFOP")]
    pub n_cfop: Option<String>,
    #[serde(rename = "nDoc")]
    pub n_doc: Option<String>,
    #[serde(rename = "nPed")]
    pub n_ped: Option<String>,
    #[serde(rename = "nPeso")]
    pub n_peso: Option<String>,
    #[serde(rename = "nRoma")]
    pub n_roma: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vNF")]
    pub v_nf: Option<String>,
    #[serde(rename = "vProd")]
    pub v_prod: Option<String>,
    #[serde(rename = "vST")]
    pub v_st: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfNfCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<InfDocCteInfNfCteInfUnidCargaCteLacUnidCarga>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidCarga")]
    pub tp_unid_carga: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfNfCteInfUnidCargaCteLacUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfCteInfUnidTransp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidTransp")]
    pub id_unid_transp: Option<String>,
    #[serde(rename = "infUnidCarga")]
    pub inf_unid_carga: Option<InfNfCteInfUnidTranspCteInfUnidCarga>,
    #[serde(rename = "lacUnidTransp")]
    pub lac_unid_transp: Option<InfNfCteInfUnidTranspCteLacUnidTransp>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidTransp")]
    pub tp_unid_transp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfCteInfUnidTranspCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<InfNfCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidCarga")]
    pub tp_unid_carga: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfCteInfUnidTranspCteLacUnidTransp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfe {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PIN")]
    pub pin: Option<String>,
    #[serde(rename = "chave")]
    pub chave: String,
    #[serde(rename = "dPrev")]
    pub d_prev: Option<String>,
    #[serde(rename = "infUnidCarga")]
    pub inf_unid_carga: Option<InfDocCteInfNfeCteInfUnidCarga>,
    #[serde(rename = "infUnidTransp")]
    pub inf_unid_transp: Option<InfNfeCteInfUnidTransp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfNfeCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<InfDocCteInfNfeCteInfUnidCargaCteLacUnidCarga>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidCarga")]
    pub tp_unid_carga: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfNfeCteInfUnidCargaCteLacUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfeCteInfUnidTransp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidTransp")]
    pub id_unid_transp: Option<String>,
    #[serde(rename = "infUnidCarga")]
    pub inf_unid_carga: Option<InfNfeCteInfUnidTranspCteInfUnidCarga>,
    #[serde(rename = "lacUnidTransp")]
    pub lac_unid_transp: Option<InfNfeCteInfUnidTranspCteLacUnidTransp>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidTransp")]
    pub tp_unid_transp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfeCteInfUnidTranspCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<InfNfeCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidCarga")]
    pub tp_unid_carga: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfeCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfeCteInfUnidTranspCteLacUnidTransp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfOutros {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "dEmi")]
    pub d_emi: Option<String>,
    #[serde(rename = "dPrev")]
    pub d_prev: Option<String>,
    #[serde(rename = "descOutros")]
    pub desc_outros: Option<String>,
    #[serde(rename = "infUnidCarga")]
    pub inf_unid_carga: Option<InfDocCteInfOutrosCteInfUnidCarga>,
    #[serde(rename = "infUnidTransp")]
    pub inf_unid_transp: Option<InfOutrosCteInfUnidTransp>,
    #[serde(rename = "nDoc")]
    pub n_doc: Option<String>,
    #[serde(rename = "tpDoc")]
    pub tp_doc: Option<String>,
    #[serde(rename = "vDocFisc")]
    pub v_doc_fisc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfOutrosCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<InfDocCteInfOutrosCteInfUnidCargaCteLacUnidCarga>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidCarga")]
    pub tp_unid_carga: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfOutrosCteInfUnidCargaCteLacUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfOutrosCteInfUnidTransp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidTransp")]
    pub id_unid_transp: Option<String>,
    #[serde(rename = "infUnidCarga")]
    pub inf_unid_carga: Option<InfOutrosCteInfUnidTranspCteInfUnidCarga>,
    #[serde(rename = "lacUnidTransp")]
    pub lac_unid_transp: Option<InfOutrosCteInfUnidTranspCteLacUnidTransp>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidTransp")]
    pub tp_unid_transp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfOutrosCteInfUnidTranspCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<InfOutrosCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga>,
    #[serde(rename = "qtdRat")]
    pub qtd_rat: Option<String>,
    #[serde(rename = "tpUnidCarga")]
    pub tp_unid_carga: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfOutrosCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfOutrosCteInfUnidTranspCteLacUnidTransp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfGlobalizado {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xObs")]
    pub x_obs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfModal {
    #[serde(rename = "@versaoModal")]
    pub versao_modal: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "AnyElement")]
    pub any_element: Option<AnyElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnyElement {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfServVinc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infCTeMultimodal")]
    pub inf_cte_multimodal: Option<InfCteMultimodal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteMultimodal {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "chCTeMultimodal")]
    pub ch_cte_multimodal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VeicNovos {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "cCor")]
    pub c_cor: Option<String>,
    #[serde(rename = "cMod")]
    pub c_mod: Option<String>,
    #[serde(rename = "chassi")]
    pub chassi: Option<String>,
    #[serde(rename = "vFrete")]
    pub v_frete: Option<String>,
    #[serde(rename = "vUnit")]
    pub v_unit: Option<String>,
    #[serde(rename = "xCor")]
    pub x_cor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteAnu {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "chCte")]
    pub ch_cte: Option<String>,
    #[serde(rename = "dEmi")]
    pub d_emi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteComp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfPaa {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CNPJPAA")]
    pub cnpjpaa: Option<String>,
    #[serde(rename = "PAASignature")]
    pub paasignature: Option<Paasignature>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Paasignature {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "RSAKeyValue")]
    pub rsakey_value: RsakeyValue,
    #[serde(rename = "SignatureValue")]
    pub signature_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RsakeyValue {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Exponent")]
    pub exponent: String,
    #[serde(rename = "Modulus")]
    pub modulus: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfSolicNff {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xSolic")]
    pub x_solic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VPrest {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Comp")]
    pub comp: Option<Vec<Comp>>,
    #[serde(rename = "vRec")]
    pub v_rec: f64,
    #[serde(rename = "vTPrest")]
    pub v_tprest: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comp {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "vComp")]
    pub v_comp: String,
    #[serde(rename = "xNome")]
    pub x_nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtCte {
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infProt")]
    pub inf_prot: InfProtocolo,
    #[serde(rename = "Signature")]
    pub signature: Option<ProtSignature>,
}

#[cfg(test)]
mod test_functions {
    use super::*;
    use crate::MyResult;
    use std::path::Path;

    // cargo test -- --help
    // cargo test -- --show-output
    // cargo test -- --show-output multiple_values

    #[test]
    /// `cargo test -- --show-output deserialize_xml_cte`
    fn deserialize_xml_cte() -> MyResult<()>{

        let mut cte_chaves = Vec::new();

        let xmls = [
            "35220998765432101234567894741048320396789012_CTe.xml",
            "41220878899001122334455667788990011223344555_CTe.xml",
        ];

        for xml in xmls {
            println!("xml: {xml}");
            let path = Path::new(xml);

            // Now, try to deserialize the XML in CteProc struct
            let cte_proc = CteProc::xml_parse(path)?;

            if xml == xmls[0] {
                println!("cte_proc: {cte_proc:#?}");
            }

            let cte = cte_proc.get_cte();
            println!("cte: {cte:?}");

            let cte_anteriores = cte_proc.get_cte_anteriores();
            println!("cte_anteriores: {cte_anteriores:#?}");

            let nfes: Vec<String> = cte_proc.get_nfes();
            println!("nfes: {nfes:#?}\n");

            cte_chaves.push(cte);
        }

        let result = [
            Some("35220998765432101234567894741048320396789012".to_string()),
            Some("41220878899001122334455667788990011223344555".to_string()),
        ];

        assert_eq!(cte_chaves, result);

        Ok(())
    }

    #[test]
    /// `cargo test -- --show-output documentos_anteriores_eletronicos`
    fn documentos_anteriores_eletronicos() -> MyResult<()>{

        let doc_anterior = IdDocAnt {
            id_doc_ant_ele: None,
            id_doc_ant_pap: None,
        };

        let docs_01 = doc_anterior.get_docs_anteriores_eletronicos();

        println!("docs_01 empty: {docs_01:?}");

        let doc_ant_ele_a = IdDocAntEle {
            ch_cte: "001".to_string(),
        };

        let doc_ant_ele_b = IdDocAntEle {
            ch_cte: "002".to_string(),
        };

        let doc_ant_ele_c = IdDocAntEle {
            ch_cte: "003".to_string(),
        };

        let docs_eletronicos: Vec<IdDocAntEle> = vec![
            doc_ant_ele_a,
            doc_ant_ele_b,
            doc_ant_ele_c,
        ];

        let doc_anterior = IdDocAnt {
            id_doc_ant_ele: Some(docs_eletronicos),
            id_doc_ant_pap: None,
        };

        let docs_02 = doc_anterior.get_docs_anteriores_eletronicos();

        println!("docs_02: {docs_02:?}");

        let result_01: Vec<String> = Vec::new();

        let result_02 = [
            "001",
            "002",
            "003",
        ];

        assert_eq!(docs_01, result_01);
        assert_eq!(docs_02, result_02);

        Ok(())
    }
}