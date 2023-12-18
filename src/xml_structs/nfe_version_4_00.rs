/**
Generates XML file from Multiple XML schemas (xsd files).

This feature requires Apache XMLBeans.
https://xmlbeans.apache.org/docs/2.0.0/guide/tools.html#xsd2inst
xsd2inst (Schema to Instance Tool)
Prints an XML instance from the specified global element using the specified schema.

git clone https://github.com/nfephp-org/sped-nfe.git

XSD to XML:
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst procNFe_v4.00.xsd -name nfeProc -dl > procNFe_v4.00.xml
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst nfe_v4.00.xsd -name NFe -dl > nfe_v4.00.xml
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst procNFe_v4.00.xsd -name ProcInutNFe procInutNFe_v4.00.xsd -dl > procInutNFe_v4.00.xml

Converter XML file to Rust struct:
read_xml -s procNFe_v4.00.xml > procNFe_v4.00.rs
read_xml -s nfe_v4.00.xml > nfe_v4.00.rs
read_xml -s procInutNFe_v4.00.xml > procInutNFe_v4.00.rs

https://dfe-portal.svrs.rs.gov.br/Nfe

No Manjaro Linux, install qxmledit:
yay -S qxmledit
Make PDF files from XSD.
*/

// Regex:
// :\s+(\w+),
// : Option<$1>,

use claudiofsr_lib::StrExtension;
use chrono::NaiveDate;
use rust_xlsxwriter::serialize_chrono_option_naive_to_excel;
use serde::{Serialize, Deserialize};
use struct_iterable::Iterable;

use crate::{
    StructExtension,
    excel::InfoExtension,
    xml_structs::integrated_dev_env::Ide,
    xml_structs::emitente::Emitente,
    xml_structs::destinatario::Destinatario,
    xml_structs::assinaturas::Signature,
    xml_structs::impostos::{Imposto, Total},
    xml_structs::aut_xml::{AutXML, InfProtocolo, InfRespTec},
};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoNfe {
    #[serde(rename = "CNPJ do Emitente")]
    emitente_cnpj: Option<String>,
    #[serde(rename = "Nome do Emitente")]
    emitente_nome: Option<String>,
    #[serde(rename = "Municípo do Emitente")]
    emitente_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Emitente")]
    emitente_ender_estado: Option<String>,
    #[serde(rename = "CNPJ do Destinatário")]
    destinatario_cnpj: Option<String>,
    #[serde(rename = "Nome do Destinatário")]
    destinatario_nome: Option<String>,
    #[serde(rename = "Municípo do Destinatário")]
    destinatario_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Destinatário")]
    destinatario_ender_estado: Option<String>,
    #[serde(rename = "Chave do Documento Fiscal")]
    pub nfe: Option<String>,
    #[serde(rename = "Registro de Origem")]
    doc_tipo: String,
    #[serde(rename = "Cancelado")]
    pub cancelado: Option<String>,
    #[serde(rename = "Nº do Documento Fiscal")]
    numero_da_nota: Option<u32>,
    #[serde(rename = "Data de Emissão", serialize_with = "serialize_chrono_option_naive_to_excel")]
    dh_emi: Option<NaiveDate>,
    #[serde(rename = "Data de Saída / Entrega", serialize_with = "serialize_chrono_option_naive_to_excel")]
    dh_sai_ent: Option<NaiveDate>,
    #[serde(rename = "Nº do Item")]
    n_item: Option<u32>,
    #[serde(rename = "Nº de Itens")]
    numero_de_itens: usize,
    #[serde(rename = "Descrição")]
    x_prod: String,
    #[serde(rename = "CFOP (Código Fiscal de Operações e Prestações)")]
    cfop: Option<u16>,
    #[serde(rename = "NCM (Nomenclatura Comum do Mercosul)")]
    ncm: Option<String>,
    #[serde(rename = "Valor do Item")]
    v_prod: Option<f64>,
    #[serde(rename = "Valor Total")]
    valor_total: Option<f64>,
    #[serde(rename = "Valor PIS/PASEP")]
    v_pis: Option<f64>,
    #[serde(rename = "Valor COFINS")]
    v_cofins: Option<f64>,
    #[serde(rename = "Valor ICMS")]
    v_icms: Option<f64>,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoNfe {}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Item {
    n_item: Option<u32>,
    x_prod: String,
    cfop: Option<u16>,
    ncm: Option<String>,
    v_prod: Option<f64>,
    v_pis: Option<f64>,
    v_cofins: Option<f64>,
    v_icms: Option<f64>,
}

/// NF-e processada
#[derive(Debug, Serialize, Deserialize)]
pub struct NfeProc {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns:nfe")]
    pub xmlns_nfe: Option<String>,
    #[serde(rename = "@xmlns:xd")]
    pub xmlns_xd: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "NFe")]
    pub nfe: Nfe,
    #[serde(rename = "protNFe")]
    pub prot_nfe: ProtNfe,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for NfeProc {}

impl NfeProc {
    pub fn get_nfe(&self) -> Option<String> {
        self
            .prot_nfe
            .inf_prot
            .ch_nfe
            .as_ref()
            .map(|s| s.remove_non_digits())
    }

    pub fn get_emitente_cnpj(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_emit_cnpj()
            })
    }

    pub fn get_emitente_nome(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_emit_nome()
            })
    }

    pub fn get_emitente_ender_municipio(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_emit_ender_municipio()
            })
    }

    pub fn get_emitente_ender_estado(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_emit_ender_estado()
            })
    }

    pub fn get_destinatario_cnpj(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_cnpj()
            })
    }

    pub fn get_destinatario_nome(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_nome()
            })
    }

    pub fn get_destinatario_ender_municipio(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_ender_municipio()
            })
    }

    pub fn get_destinatario_ender_estado(&self) -> Option<String> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|inf| {
                inf.get_dest_ender_estado()
            })
    }

    pub fn get_numero_da_nota(&self) -> Option<u32> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|info| {
                info.ide.get_n_nf()
            })
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|info| {
                info.ide.get_dh_emi()
            })
    }

    pub fn get_data_saida(&self) -> Option<NaiveDate> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|info| {
                info.ide.get_dh_sai()
            })
    }

    pub fn get_value_total(&self) -> Option<f64> {
        self
            .nfe
            .inf_nfe
            .as_ref()
            .and_then(|info| info.total.get_valor_total_da_nota_fiscal())
    }

    pub fn get_itens(&self) -> Vec<Item> {
        match self.nfe.inf_nfe.as_ref() {
            Some(info) => {
                info
                    .det
                    .iter()
                    .map(|item| {
                        let produto = &item.prod;
                        let imposto = &item.imposto;
                        Item {
                            n_item: item.n_item.trim().parse().ok(),
                            x_prod: produto.x_prod.replace_multiple_whitespaces().to_uppercase(),
                            cfop: produto.cfop.as_ref().and_then(|c| c.remove_non_digits().parse().ok()),
                            ncm: produto.ncm.as_ref().map(|n| n.trim().format_ncm()),
                            v_prod: produto.v_prod,
                            v_pis: imposto.get_v_pis(),
                            v_cofins: imposto.get_v_cofins(),
                            v_icms: imposto.get_v_icms(),
                        }
                    })
                    .collect()
            }
            None => Vec::new()
        }
    }

    pub fn get_infos(&self) -> Vec<InfoNfe> {

        let mut infos = Vec::new();
        let itens: Vec<Item> = self.get_itens();
        let numero_de_itens = itens.len();

        for item in itens {
            let info_nfe = InfoNfe {
                emitente_cnpj: self.get_emitente_cnpj(),
                emitente_nome: self.get_emitente_nome(),
                emitente_ender_municipio: self.get_emitente_ender_municipio(),
                emitente_ender_estado: self.get_emitente_ender_estado(),
                destinatario_cnpj: self.get_destinatario_cnpj(),
                destinatario_nome: self.get_destinatario_nome(),
                destinatario_ender_municipio: self.get_destinatario_ender_municipio(),
                destinatario_ender_estado: self.get_destinatario_ender_estado(),
                nfe: self.get_nfe(),
                doc_tipo: "NFe".to_string(),
                cancelado: None,
                numero_da_nota: self.get_numero_da_nota(),
                dh_emi: self.get_data_emissao(),
                dh_sai_ent: self.get_data_saida(),
                n_item: item.n_item,
                numero_de_itens,
                x_prod: item.x_prod,
                cfop: item.cfop,
                ncm: item.ncm,
                v_prod: item.v_prod,
                valor_total: self.get_value_total(),
                v_pis: item.v_pis,
                v_cofins: item.v_cofins,
                v_icms: item.v_icms,
            };
            infos.push(info_nfe.clone());
        }

        infos
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nfe {
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "infNFe")]
    pub inf_nfe: Option<InfNfe>,
    #[serde(rename = "infNFeSupl")]
    pub inf_nfe_supl: Option<InfNfeSupl>,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfe {
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ide")] // identificação da NF-e
    pub ide: Ide,
    #[serde(rename = "emit")]
    pub emit: Option<Emitente>,
    #[serde(rename = "avulsa")]
    pub avulsa: Option<Avulsa>,
    #[serde(rename = "dest")]
    pub dest: Option<Destinatario>,
    #[serde(rename = "retirada")]
    pub retirada: Option<Retirada>,
    #[serde(rename = "entrega")]
    pub entrega: Option<Entrega>,
    #[serde(rename = "autXML")]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "det")]
    pub det: Vec<Detalhes>,
    #[serde(rename = "total")]
    pub total: Total,
    #[serde(rename = "transp")]
    pub transp: Transp,
    #[serde(rename = "cobr")]
    pub cobr: Option<Cobr>,
    #[serde(rename = "pag")]
    pub pag: Pagamento,
    #[serde(rename = "infIntermed")]
    pub inf_intermed: Option<InfIntermed>,
    #[serde(rename = "infAdic")]
    pub inf_adic: Option<InfAdic>,
    #[serde(rename = "exporta")]
    pub exporta: Option<Exporta>,
    #[serde(rename = "compra")]
    pub compra: Option<Compra>,
    #[serde(rename = "cana")]
    pub cana: Option<Cana>,
    #[serde(rename = "infRespTec")]
    pub inf_resp_tec: Option<InfRespTec>,
}

impl InfNfe {

    /// Emitente: CNPJ
    fn get_emit_cnpj(&self) -> Option<String> {
        self
            .emit
            .as_ref()
            .and_then(|emitente| {
                emitente.get_cnpj()
            })
    }

    /// Emitente: Nome
    fn get_emit_nome(&self) -> Option<String> {
        self
            .emit
            .as_ref()
            .and_then(|emitente| {
                emitente.get_nome()
            })
    }

    /// Emitente: Endereço do Município
    fn get_emit_ender_municipio(&self) -> Option<String> {
        self
            .emit
            .as_ref()
            .and_then(|emitente| {
                emitente.get_endereco_municipio()
            })
    }

    /// Emitente: Endereço do Estado
    fn get_emit_ender_estado(&self) -> Option<String> {
        self
            .emit
            .as_ref()
            .and_then(|emitente| {
                emitente.get_endereco_uf()
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Avulsa {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "xOrgao")]
    pub x_orgao: Option<String>,
    #[serde(rename = "matr")]
    pub matr: Option<String>,
    #[serde(rename = "xAgente")]
    pub x_agente: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "nDAR")]
    pub n_dar: Option<String>,
    #[serde(rename = "dEmi")]
    pub d_emi: Option<String>,
    #[serde(rename = "vDAR")]
    pub v_dar: Option<String>,
    #[serde(rename = "repEmi")]
    pub rep_emi: Option<String>,
    #[serde(rename = "dPag")]
    pub d_pag: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Retirada {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "xCpl")]
    pub x_cpl: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entrega {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "nro")]
    pub nro: String,
    #[serde(rename = "xCpl")]
    pub x_cpl: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: String,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
    #[serde(rename = "fone")]
    pub fone: Option<String>,
    #[serde(rename = "email")]
    pub email: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Detalhes {
    #[serde(rename = "infAdProd")]
    pub inf_ad_prod: Option<String>,
    #[serde(rename = "@nItem")]
    pub n_item: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "prod")]
    pub prod: Produto,
    #[serde(rename = "imposto")]
    pub imposto: Imposto,
    #[serde(rename = "impostoDevol")]
    pub imposto_devol: Option<ImpostoDevol>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Produto {
    #[serde(rename = "cProd")]
    pub c_prod: Option<String>,
    #[serde(rename = "cEAN")]
    pub c_ean: Option<String>,
    #[serde(rename = "xProd")]
    pub x_prod: String,
    #[serde(rename = "NCM")]
    pub ncm: Option<String>,
    #[serde(rename = "NVE")]
    pub nve: Option<String>,
    #[serde(rename = "CEST")]
    pub cest: Option<String>,
    #[serde(rename = "indEscala")]
    pub ind_escala: Option<String>,
    #[serde(rename = "CNPJFab")]
    pub cnpjfab: Option<String>,
    #[serde(rename = "cBenef")]
    pub c_benef: Option<String>,
    #[serde(rename = "EXTIPI")]
    pub extipi: Option<String>,
    #[serde(rename = "CFOP")]
    pub cfop: Option<String>,
    #[serde(rename = "uCom")]
    pub u_com: Option<String>,
    #[serde(rename = "qCom")]
    pub q_com: Option<String>,
    #[serde(rename = "vUnCom")]
    pub v_un_com: Option<String>,
    #[serde(rename = "vProd")]
    pub v_prod: Option<f64>,
    #[serde(rename = "cEANTrib")]
    pub c_eantrib: Option<String>,
    #[serde(rename = "uTrib")]
    pub u_trib: Option<String>,
    #[serde(rename = "qTrib")]
    pub q_trib: Option<String>,
    #[serde(rename = "vUnTrib")]
    pub v_un_trib: Option<String>,
    #[serde(rename = "vFrete")]
    pub v_frete: Option<String>,
    #[serde(rename = "vSeg")]
    pub v_seg: Option<String>,
    #[serde(rename = "vDesc")]
    pub v_desc: Option<String>,
    #[serde(rename = "vOutro")]
    pub v_outro: Option<String>,
    #[serde(rename = "indTot")]
    pub ind_tot: Option<String>,
    #[serde(rename = "xPed")]
    pub x_ped: Option<String>,
    #[serde(rename = "nItemPed")]
    pub n_item_ped: Option<String>,
    #[serde(rename = "nFCI")]
    pub n_fci: Option<String>,
    #[serde(rename = "nRECOPI")]
    pub n_recopi: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "DI")]
    pub di: Option<Di>,
    #[serde(rename = "detExport")]
    pub det_export: Option<DetExport>,
    #[serde(rename = "rastro")]
    pub rastro: Option<Rastro>,
    #[serde(rename = "veicProd")]
    pub veic_prod: Option<VeicProd>,
    #[serde(rename = "med")]
    pub med: Option<Med>,
    #[serde(rename = "arma")]
    pub arma: Option<Arma>,
    #[serde(rename = "comb")]
    pub comb: Option<Comb>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Di {
    #[serde(rename = "nDI")]
    pub n_di: Option<String>,
    #[serde(rename = "dDI")]
    pub d_di: Option<String>,
    #[serde(rename = "xLocDesemb")]
    pub x_loc_desemb: Option<String>,
    #[serde(rename = "UFDesemb")]
    pub ufdesemb: Option<String>,
    #[serde(rename = "dDesemb")]
    pub d_desemb: Option<String>,
    #[serde(rename = "tpViaTransp")]
    pub tp_via_transp: Option<String>,
    #[serde(rename = "vAFRMM")]
    pub v_afrmm: Option<String>,
    #[serde(rename = "tpIntermedio")]
    pub tp_intermedio: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "UFTerceiro")]
    pub ufterceiro: Option<String>,
    #[serde(rename = "cExportador")]
    pub c_exportador: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "adi")]
    pub adi: Option<Adi>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Adi {
    #[serde(rename = "nAdicao")]
    pub n_adicao: Option<String>,
    #[serde(rename = "nSeqAdic")]
    pub n_seq_adic: Option<String>,
    #[serde(rename = "cFabricante")]
    pub c_fabricante: Option<String>,
    #[serde(rename = "vDescDI")]
    pub v_desc_di: Option<String>,
    #[serde(rename = "nDraw")]
    pub n_draw: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetExport {
    #[serde(rename = "nDraw")]
    pub n_draw: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "exportInd")]
    pub export_ind: Option<ExportInd>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportInd {
    #[serde(rename = "nRE")]
    pub n_re: Option<String>,
    #[serde(rename = "chNFe")]
    pub ch_nfe: Option<String>,
    #[serde(rename = "qExport")]
    pub q_export: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rastro {
    #[serde(rename = "nLote")]
    pub n_lote: Option<String>,
    #[serde(rename = "qLote")]
    pub q_lote: Option<String>,
    #[serde(rename = "dFab")]
    pub d_fab: Option<String>,
    #[serde(rename = "dVal")]
    pub d_val: Option<String>,
    #[serde(rename = "cAgreg")]
    pub c_agreg: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VeicProd {
    #[serde(rename = "tpOp")]
    pub tp_op: Option<String>,
    #[serde(rename = "chassi")]
    pub chassi: Option<String>,
    #[serde(rename = "cCor")]
    pub c_cor: Option<String>,
    #[serde(rename = "xCor")]
    pub x_cor: Option<String>,
    #[serde(rename = "pot")]
    pub pot: Option<String>,
    #[serde(rename = "cilin")]
    pub cilin: Option<String>,
    #[serde(rename = "pesoL")]
    pub peso_l: Option<String>,
    #[serde(rename = "pesoB")]
    pub peso_b: Option<String>,
    #[serde(rename = "nSerie")]
    pub n_serie: Option<String>,
    #[serde(rename = "tpComb")]
    pub tp_comb: Option<String>,
    #[serde(rename = "nMotor")]
    pub n_motor: Option<String>,
    #[serde(rename = "CMT")]
    pub cmt: Option<String>,
    #[serde(rename = "dist")]
    pub dist: Option<String>,
    #[serde(rename = "anoMod")]
    pub ano_mod: Option<String>,
    #[serde(rename = "anoFab")]
    pub ano_fab: Option<String>,
    #[serde(rename = "tpPint")]
    pub tp_pint: Option<String>,
    #[serde(rename = "tpVeic")]
    pub tp_veic: Option<String>,
    #[serde(rename = "espVeic")]
    pub esp_veic: Option<String>,
    #[serde(rename = "VIN")]
    pub vin: Option<String>,
    #[serde(rename = "condVeic")]
    pub cond_veic: Option<String>,
    #[serde(rename = "cMod")]
    pub c_mod: Option<String>,
    #[serde(rename = "cCorDENATRAN")]
    pub c_cor_denatran: Option<String>,
    #[serde(rename = "lota")]
    pub lota: Option<String>,
    #[serde(rename = "tpRest")]
    pub tp_rest: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Med {
    #[serde(rename = "cProdANVISA")]
    pub c_prod_anvisa: Option<String>,
    #[serde(rename = "xMotivoIsencao")]
    pub x_motivo_isencao: Option<String>,
    #[serde(rename = "vPMC")]
    pub v_pmc: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arma {
    #[serde(rename = "tpArma")]
    pub tp_arma: Option<String>,
    #[serde(rename = "nSerie")]
    pub n_serie: Option<String>,
    #[serde(rename = "nCano")]
    pub n_cano: Option<String>,
    #[serde(rename = "descr")]
    pub descr: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comb {
    #[serde(rename = "cProdANP")]
    pub c_prod_anp: Option<String>,
    #[serde(rename = "descANP")]
    pub desc_anp: Option<String>,
    #[serde(rename = "pGLP")]
    pub p_glp: Option<String>,
    #[serde(rename = "pGNn")]
    pub p_gnn: Option<String>,
    #[serde(rename = "pGNi")]
    pub p_gni: Option<String>,
    #[serde(rename = "vPart")]
    pub v_part: Option<String>,
    #[serde(rename = "CODIF")]
    pub codif: Option<String>,
    #[serde(rename = "qTemp")]
    pub q_temp: Option<String>,
    #[serde(rename = "UFCons")]
    pub ufcons: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CIDE")]
    pub cide: Option<Cide>,
    #[serde(rename = "encerrante")]
    pub encerrante: Option<Encerrante>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cide {
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vCIDE")]
    pub v_cide: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Encerrante {
    #[serde(rename = "nBico")]
    pub n_bico: Option<String>,
    #[serde(rename = "nBomba")]
    pub n_bomba: Option<String>,
    #[serde(rename = "nTanque")]
    pub n_tanque: Option<String>,
    #[serde(rename = "vEncIni")]
    pub v_enc_ini: Option<String>,
    #[serde(rename = "vEncFin")]
    pub v_enc_fin: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ImpostoDevol {
    #[serde(rename = "pDevol")]
    pub p_devol: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "IPI")]
    pub ipi: Option<ImpostoDevolNfeIpi>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpostoDevolNfeIpi {
    #[serde(rename = "vIPIDevol")]
    pub v_ipidevol: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transp {
    #[serde(rename = "modFrete")]
    pub mod_frete: Option<String>,
    #[serde(rename = "vagao")]
    pub vagao: Option<String>,
    #[serde(rename = "balsa")]
    pub balsa: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "transporta")]
    pub transporta: Option<Transporta>,
    #[serde(rename = "retTransp")]
    pub ret_transp: Option<RetTransp>,
    #[serde(rename = "veicTransp")]
    pub veic_transp: Option<VeicTransp>,
    #[serde(rename = "reboque")]
    pub reboque: Option<Reboque>,
    #[serde(rename = "vol")]
    pub vol: Option<Vec<Vol>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Transporta {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "xEnder")]
    pub x_ender: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetTransp {
    #[serde(rename = "vServ")]
    pub v_serv: Option<String>,
    #[serde(rename = "vBCRet")]
    pub v_bcret: Option<String>,
    #[serde(rename = "pICMSRet")]
    pub p_icmsret: Option<String>,
    #[serde(rename = "vICMSRet")]
    pub v_icmsret: Option<String>,
    #[serde(rename = "CFOP")]
    pub cfop: Option<String>,
    #[serde(rename = "cMunFG")]
    pub c_mun_fg: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VeicTransp {
    #[serde(rename = "placa")]
    pub placa: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "RNTC")]
    pub rntc: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reboque {
    #[serde(rename = "placa")]
    pub placa: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "RNTC")]
    pub rntc: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Vol {
    #[serde(rename = "qVol")]
    pub q_vol: Option<String>,
    #[serde(rename = "esp")]
    pub esp: Option<String>,
    #[serde(rename = "marca")]
    pub marca: Option<String>,
    #[serde(rename = "nVol")]
    pub n_vol: Option<String>,
    #[serde(rename = "pesoL")]
    pub peso_l: Option<String>,
    #[serde(rename = "pesoB")]
    pub peso_b: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "lacres")]
    pub lacres: Option<Lacres>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lacres {
    #[serde(rename = "nLacre")]
    pub n_lacre: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cobr {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "fat")]
    pub fat: Option<Fat>,
    #[serde(rename = "dup")]
    pub dup: Option<Vec<Dup>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fat {
    #[serde(rename = "nFat")]
    pub n_fat: Option<String>,
    #[serde(rename = "vOrig")]
    pub v_orig: Option<String>,
    #[serde(rename = "vDesc")]
    pub v_desc: Option<String>,
    #[serde(rename = "vLiq")]
    pub v_liq: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dup {
    #[serde(rename = "nDup")]
    pub n_dup: Option<String>,
    #[serde(rename = "dVenc")]
    pub d_venc: Option<String>,
    #[serde(rename = "vDup")]
    pub v_dup: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagamento {
    #[serde(rename = "vTroco")]
    pub v_troco: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "detPag")]
    pub det_pag: Vec<DetalhesPagamento>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetalhesPagamento {
    #[serde(rename = "indPag")]
    pub ind_pag: Option<String>,
    #[serde(rename = "tPag")]
    pub t_pag: Option<String>,
    #[serde(rename = "xPag")]
    pub x_pag: Option<String>,
    #[serde(rename = "vPag")]
    pub v_pag: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "card")]
    pub card: Option<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    #[serde(rename = "tpIntegra")]
    pub tp_integra: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "tBand")]
    pub t_band: Option<String>,
    #[serde(rename = "cAut")]
    pub c_aut: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfIntermed {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "idCadIntTran")]
    pub id_cad_int_tran: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfAdic {
    #[serde(rename = "infAdFisco")]
    pub inf_ad_fisco: Option<String>,
    #[serde(rename = "infCpl")]
    pub inf_cpl: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "obsCont")]
    pub obs_cont: Option<Vec<ObsContribuinte>>,
    #[serde(rename = "obsFisco")]
    pub obs_fisco: Option<ObsFisco>,
    #[serde(rename = "procRef")]
    pub proc_ref: Option<Vec<ProcRef>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsContribuinte {
    #[serde(rename = "xTexto")]
    pub x_texto: Option<String>,
    #[serde(rename = "@xCampo")]
    pub x_campo: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsFisco {
    #[serde(rename = "xTexto")]
    pub x_texto: Option<String>,
    #[serde(rename = "@xCampo")]
    pub x_campo: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcRef {
    #[serde(rename = "nProc")]
    pub n_proc: Option<String>,
    #[serde(rename = "indProc")]
    pub ind_proc: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Exporta {
    #[serde(rename = "UFSaidaPais")]
    pub ufsaida_pais: Option<String>,
    #[serde(rename = "xLocExporta")]
    pub x_loc_exporta: Option<String>,
    #[serde(rename = "xLocDespacho")]
    pub x_loc_despacho: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Compra {
    #[serde(rename = "xNEmp")]
    pub x_nemp: Option<String>,
    #[serde(rename = "xPed")]
    pub x_ped: Option<String>,
    #[serde(rename = "xCont")]
    pub x_cont: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cana {
    #[serde(rename = "safra")]
    pub safra: Option<String>,
    #[serde(rename = "ref")]
    pub refer: Option<String>,
    #[serde(rename = "qTotMes")]
    pub q_tot_mes: Option<String>,
    #[serde(rename = "qTotAnt")]
    pub q_tot_ant: Option<String>,
    #[serde(rename = "qTotGer")]
    pub q_tot_ger: Option<String>,
    #[serde(rename = "vFor")]
    pub v_for: Option<String>,
    #[serde(rename = "vTotDed")]
    pub v_tot_ded: Option<String>,
    #[serde(rename = "vLiqFor")]
    pub v_liq_for: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "forDia")]
    pub for_dia: Option<ForDia>,
    #[serde(rename = "deduc")]
    pub deduc: Option<Deduc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForDia {
    #[serde(rename = "qtde")]
    pub qtde: Option<String>,
    #[serde(rename = "@dia")]
    pub dia: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deduc {
    #[serde(rename = "xDed")]
    pub x_ded: Option<String>,
    #[serde(rename = "vDed")]
    pub v_ded: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtNfe {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "infProt")] // Dados do protocolo de status
    pub inf_prot: InfProtocolo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfeSupl {
    #[serde(rename = "qrCode")]
    pub qr_code: Option<String>,
    #[serde(rename = "urlChave")]
    pub url_chave: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
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
    /// `cargo test -- --show-output deserialize_xml_nfe`
    fn deserialize_xml_nfe() -> MyResult<()>{

        let mut nfe_chaves = Vec::new();

        let xmls = [
            "35220412345678901234567890123456789012345678_NFe.xml",
            "35220956789012345678901234560030301250352046_NFe.xml",
        ];

        for xml in xmls {
            println!("xml: {xml}");
            let path = Path::new(xml);

            // Now, try to deserialize the XML in NfeProc struct
            let nfe_proc = NfeProc::xml_parse(path)?;

            if xml == xmls[0] {
                println!("nfe_proc: {nfe_proc:#?}");
            }

            let nfe = nfe_proc.get_nfe();
            println!("nfe: {nfe:?}");

            nfe_chaves.push(nfe);
        }

        let result = [
            Some("35220412345678901234567890123456789012345678".to_string()),
            Some("35220956789012345678901234560030301250352046".to_string()),
        ];

        assert_eq!(nfe_chaves, result);

        Ok(())
    }
}