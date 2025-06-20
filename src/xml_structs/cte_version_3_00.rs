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
use claudiofsr_lib::{BTreeSetExtension, UniqueElements};
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeSet;
use struct_iterable::Iterable;

use crate::{
    Arguments, GetFirst, Information, KeysExtension, OptExt, StructExtension,
    excel::InfoExtension,
    group_by_hashmap::GetKey,
    serialize_vec_string,
    unique_with_cows::GetID,
    xml_structs::{
        agente::{Agente, AgenteExtension, TOMADOR_DO_SERVICO},
        assinaturas::{ProtSignature, Signature},
        aut_xml::{AutXML, InfProtocolo, InfRespTec},
        cobranca::Cobranca,
        entrega::Entrega,
        impostos::Imposto,
        integrated_dev_env::Ide,
    },
};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoCte {
    #[serde(rename = "Versão XML")]
    versao: Option<String>,

    /*
    Emitente:
    É o ator (pessoa), participante em um CT-e, responsável pelo gerenciamento do transporte,
    normalmente, quem executa a operação. Deverá ser informado obrigatoriamente para qualquer
    prestação de serviço.
    */
    #[serde(rename = "CNPJ do Emitente")]
    pub emitente_cnpj: Option<String>,
    #[serde(rename = "CPF do Emitente")]
    pub emitente_cpf: Option<String>,
    #[serde(rename = "CRT (Código do Regime Tributário) conforme Emitente")]
    emitente_crt: Option<u8>,
    #[serde(rename = "Nome ou Razão Social do Emitente")]
    emitente_nome: Option<String>,
    #[serde(rename = "Nome Fantasia do Emitente")]
    emitente_fantasia: Option<String>,
    #[serde(rename = "Municípo do Emitente")]
    emitente_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Emitente")]
    emitente_ender_estado: Option<String>,

    /*
    Remetente:
    É o ator (pessoa), participante em um CT-e, responsável por promover a saída
    inicial da carga. Poderá não ser informado quando o Tipo de Serviço for igual a
    "3 - Redespacho Intermediário" ou "4 - Serviço Vinculado a Multimodal".
    */
    #[serde(rename = "CNPJ do Remetente")]
    pub remetente_cnpj: Option<String>,
    #[serde(rename = "CPF do Remetente")]
    pub remetente_cpf: Option<String>,
    #[serde(rename = "CRT (Código do Regime Tributário) conforme Remetente")]
    remetente_crt: Option<u8>,
    #[serde(rename = "Nome ou Razão Social do Remetente")]
    remetente_nome: Option<String>,
    #[serde(rename = "Nome Fantasia do Remetente")]
    remetente_fantasia: Option<String>,
    #[serde(rename = "Municípo do Remetente")]
    remetente_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Remetente")]
    remetente_ender_estado: Option<String>,
    #[serde(rename = "NFes do Remetente", serialize_with = "serialize_vec_string")]
    pub remetente_nfes: Vec<String>,

    /*
    Destinatário:
    É o ator (pessoa), participante em um CT-e, para quem a mercadoria será enviada.
    Deverá ser informado obrigatoriamente quando o Tipo de Serviço for igual a
    "3 - Redespacho Intermediário" ou "4 - Serviço Vinculado a Multimodal".
    */
    #[serde(rename = "CNPJ do Destinatário")]
    pub destinatario_cnpj: Option<String>,
    #[serde(rename = "CPF do Destinatário")]
    pub destinatario_cpf: Option<String>,
    #[serde(rename = "CRT (Código do Regime Tributário) conforme Destinatário")]
    destinatario_crt: Option<u8>,
    #[serde(rename = "Nome ou Razão Social do Destinatário")]
    destinatario_nome: Option<String>,
    #[serde(rename = "Nome Fantasia do Destinatário")]
    destinatario_fantasia: Option<String>,
    #[serde(rename = "Municípo do Destinatário")]
    destinatario_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Destinatário")]
    destinatario_ender_estado: Option<String>,

    /*
    Expedidor:
    É o ator (pessoa), participante em um CT-e, responsável por entregar a carga ao transportador
    para efetuar o serviço de transporte, podendo ser essa entrega de transportador para
    transportador. Deverá ser informado obrigatoriamente quando o Tipo de Serviço for igual a
    "3 - Redespacho Intermediário" ou "4 - Serviço Vinculado a Multimodal".
    */
    #[serde(rename = "CNPJ do Expedidor")]
    pub expedidor_cnpj: Option<String>,
    #[serde(rename = "CPF do Expedidor")]
    pub expedidor_cpf: Option<String>,
    #[serde(rename = "CRT (Código do Regime Tributário) conforme Expedidor")]
    expedidor_crt: Option<u8>,
    #[serde(rename = "Nome ou Razão Social do Expedidor")]
    expedidor_nome: Option<String>,
    #[serde(rename = "Nome Fantasia do Expedidor")]
    expedidor_fantasia: Option<String>,
    #[serde(rename = "Municípo do Expedidor")]
    expedidor_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Expedidor")]
    expedidor_ender_estado: Option<String>,

    /*
    Recebedor:
    É o ator (pessoa), participante em um CT-e, responsável por receber a carga do transportador.
    Deverá ser informado obrigatoriamente quando o Tipo de Serviço for igual a
    "3 - Redespacho Intermediário" ou "4 - Serviço Vinculado a Multimodal".
    */
    #[serde(rename = "CNPJ do Recebedor")]
    pub recebedor_cnpj: Option<String>,
    #[serde(rename = "CPF do Recebedor")]
    pub recebedor_cpf: Option<String>,
    #[serde(rename = "CRT (Código do Regime Tributário) conforme Recebedor")]
    recebedor_crt: Option<u8>,
    #[serde(rename = "Nome ou Razão Social do Recebedor")]
    recebedor_nome: Option<String>,
    #[serde(rename = "Nome Fantasia do Recebedor")]
    recebedor_fantasia: Option<String>,
    #[serde(rename = "Municípo do Recebedor")]
    recebedor_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Recebedor")]
    recebedor_ender_estado: Option<String>,

    /*
    Tomador:
    Ator que paga o frete da operação de transporte.
    Ele pode ser o remetente, destinatário, recebedor ou outra empresa que não está presente no CTe.
    Neste último caso, é necessário que todos os dados do novo ator sejam informados no documento.
    */
    #[serde(rename = "CNPJ do Tomador")]
    tomador_cnpj: Option<String>,
    #[serde(rename = "CPF do Tomador")]
    tomador_cpf: Option<String>,
    #[serde(rename = "Nome ou Razão Social do Tomador")]
    tomador_nome: Option<String>,
    #[serde(rename = "Nome Fantasia do Tomador")]
    tomador_fantasia: Option<String>,
    #[serde(rename = "Municípo do Tomador")]
    tomador_ender_municipio: Option<String>,
    #[serde(rename = "Estado do Tomador")]
    tomador_ender_estado: Option<String>,

    /**
    Código do Tomador do Serviço:

    0. Remetente;
    1. Expedidor;
    2. Recebedor;
    3. Destinatário;
    4. Terceiro [adicionado em CTe versão 4.00].
    */
    #[serde(
        rename = "Tomador do Serviço",
        serialize_with = "serialize_tomador_do_servico"
    )]
    pub tomador_codigo: Option<u8>,

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
    #[serde(
        rename = "Data de Emissão",
        serialize_with = "serialize_option_datetime_to_excel"
    )]
    pub data_emissao: Option<NaiveDate>,
    #[serde(
        rename = "CTe Multimodal, Complementar, Redespacho, Subcontratação, Substituição ou Vinculado",
        serialize_with = "serialize_vec_string"
    )]
    pub cte_complementar: Vec<String>,
    #[serde(rename = "CTes Anteriores", serialize_with = "serialize_vec_string")]
    cte_anteriores: Vec<String>,
    #[serde(rename = "NFes Vinculados", serialize_with = "serialize_vec_string")]
    nfes_vinculados: Vec<String>,

    #[serde(
        rename = "Informações de NFes relacionados",
        serialize_with = "serialize_vec_string"
    )]
    pub nfes: Vec<String>,
    #[serde(
        rename = "(NCM e Descrição) dos Itens de NFes com Valores e Porcentagens decrescentes",
        serialize_with = "serialize_vec_string"
    )]
    pub ncm_descricao: Vec<String>,

    #[serde(rename = "Valor Total de NFes")]
    pub valor_total_nfes: Option<f64>,
    #[serde(rename = "Valor Total do CTe")]
    pub valor_total: Option<f64>,

    /*
    #[serde(rename = "Valor do Desconto")]
    v_desc: Option<f64>,
    #[serde(rename = "Valor do Frete")]
    v_frete: Option<f64>,
    #[serde(rename = "Valor do Seguro")]
    v_seg: Option<f64>,
    */
    #[serde(rename = "Alíquota de PIS/PASEP")]
    aliq_pis: Option<f64>,
    #[serde(rename = "Alíquota de COFINS")]
    aliq_cofins: Option<f64>,
    #[serde(rename = "Valor de PIS/PASEP")]
    v_pis: Option<f64>,
    #[serde(rename = "Valor de COFINS")]
    v_cofins: Option<f64>,
    #[serde(rename = "Valor de IPI")]
    v_ipi: Option<f64>,
    #[serde(rename = "Valor de ISS")]
    v_iss: Option<f64>,
    #[serde(rename = "Valor da Base de Cálculo do ICMS")]
    v_bc_icms: Option<f64>,
    #[serde(rename = "Alíquota de ICMS")]
    aliq_icms: Option<f64>,
    #[serde(rename = "Valor de ICMS")]
    v_icms: Option<f64>,
}

impl InfoCte {
    pub fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }

    /// CTe com chave existente não cancelada
    pub fn is_valid(&self) -> bool {
        self.cte.is_some() && self.cancelado.is_none()
    }

    /// CTe com chave existente, porém cancelada.
    pub fn is_canceled(&self) -> bool {
        self.cte.is_some() && self.cancelado.is_some()
    }

    fn get_base_remetente(&self) -> Option<String> {
        [
            self.remetente_cnpj.get_first(10),
            self.remetente_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
        .get_not_empty()
    }

    fn get_base_emitente(&self) -> Option<String> {
        [
            self.emitente_cnpj.get_first(10),
            self.emitente_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
        .get_not_empty()
    }

    fn get_base_recebedor(&self) -> Option<String> {
        [
            self.recebedor_cnpj.get_first(10),
            self.recebedor_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
        .get_not_empty()
    }

    fn get_base_destinatario(&self) -> Option<String> {
        [
            self.destinatario_cnpj.get_first(10),
            self.destinatario_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
        .get_not_empty()
    }

    fn get_base_tomador(&self) -> Option<String> {
        [
            self.tomador_cnpj.get_first(10),
            self.tomador_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
        .get_not_empty()
    }

    /**
        Corrigir o código do Tomador do Serviço após comparação entre:

        CNPJ_CPF Base do Tomador do Serviço

        e

        CNPJ_CPF Base do Remetente/Emitente/Recebedor/Destinatario

        Código do Tomador do Serviço:

        0. Remetente;
        1. Expedidor;
        2. Recebedor;
        3. Destinatário;
        4. Terceiro [adicionado em CTe versão 4.00].
    */
    pub fn corrigir_codigo_do_tomador(&mut self) {
        for (codigo, opt_base_agente) in [
            (0, self.get_base_remetente()),
            (1, self.get_base_emitente()),
            (2, self.get_base_recebedor()),
            (3, self.get_base_destinatario()),
        ] {
            match (self.get_base_tomador(), opt_base_agente) {
                (Some(base_toma), Some(base_agente)) if base_toma == base_agente => {
                    self.tomador_codigo = Some(codigo);
                    break;
                }
                _ => continue,
            }
        }
    }

    /// Obter CNPJ_CPF Base do Tomador do Serviço
    ///
    /// Inferir CNPJ_CPF Base a partir código (após correção) do Tomador
    pub fn get_cnpj_cpf_base_do_tomador(&self) -> Option<String> {
        let tomador_cnpj_cpf: Option<String> = match &self.tomador_codigo {
            Some(0) => self.get_base_remetente(),
            Some(1) => self.get_base_emitente(),
            Some(2) => self.get_base_recebedor(),
            Some(3) => self.get_base_destinatario(),
            Some(4) => self.get_base_tomador(),
            _ => None,
        };

        tomador_cnpj_cpf
    }

    /// Get unique and ordered elements
    ///
    /// Vetor com apenas 1 item não necessita ordenar
    pub fn get_unique_elements(&mut self) {
        [
            &mut self.remetente_nfes,
            &mut self.cte_complementar,
            &mut self.cte_anteriores,
            &mut self.nfes_vinculados,
        ]
        .into_iter()
        .filter(|vec| vec.len() > 1)
        .for_each(|vec| vec.unique_ordered());
    }

    /// Get correlated CTes
    pub fn get_correlated_ctes(&self) -> Vec<String> {
        [&self.cte_complementar, &self.cte_anteriores]
            .into_iter()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .to_vec()
    }

    /// Get correlated NFes
    pub fn get_correlated_nfes(&self) -> Vec<String> {
        [&self.remetente_nfes, &self.nfes_vinculados]
            .into_iter()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .to_vec()
    }
}

impl KeysExtension for [InfoCte] {
    fn get_chaves(&self) -> BTreeSet<String> {
        self.iter().flat_map(|info| info.get_chave()).collect()
    }
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCte {}

/// Ver src/group_by_hashmap.rs
impl GetKey for InfoCte {
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

/// Ver src/test_cow.rs
impl GetID<Option<String>> for InfoCte {
    fn get_id(&self) -> Option<String> {
        self.cte.clone()
    }
}

/// Substituir o Código pelo atributo do Tomador do Serviço do CTe
pub fn serialize_tomador_do_servico<S>(
    codigo: &Option<u8>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if codigo.is_none() {
        serializer.serialize_none()
    } else {
        let opt_tomador = TOMADOR_DO_SERVICO.get(codigo).map(|&t| t.to_string());
        match opt_tomador {
            Some(t) => serializer.serialize_some(&t),
            None => serializer.serialize_none(),
        }
    }
}

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
impl StructExtension for CteProc {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cte xml_path: {xml_path:?}");
            println!("cte_proc: {self:#?}\n");
        }
        Information::Cte(Box::new(self.get_info()))
    }
}

impl CteProc {
    pub fn get_versao(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.versao.clone())
    }

    // Emitente

    pub fn get_emitente_cnpj(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_cnpj())
    }

    pub fn get_emitente_cpf(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_emit_cpf())
    }

    /// CRT (Código do Regime Tributário):
    ///
    /// 1 - Simples Nacional;
    ///
    /// 2 - Simples Nacional - excesso de sublimite de receita bruta;
    ///
    /// 3 - Regime Normal.
    pub fn get_emitente_cod_regime_tributario(&self) -> Option<u8> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_cod_reg_trib())
    }

    pub fn get_emitente_nome(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_nome())
    }

    pub fn get_emitente_fantasia(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_fantasia())
    }

    pub fn get_emitente_ender_municipio(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_ender_municipio())
    }

    pub fn get_emitente_ender_estado(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_ender_estado())
    }

    // Remetente

    pub fn get_remetente_cnpj(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_rem_cnpj())
    }

    pub fn get_remetente_cpf(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_rem_cpf())
    }

    /// CRT (Código do Regime Tributário):
    ///
    /// 1 - Simples Nacional;
    ///
    /// 2 - Simples Nacional - excesso de sublimite de receita bruta;
    ///
    /// 3 - Regime Normal.
    pub fn get_remetente_cod_regime_tributario(&self) -> Option<u8> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_rem_cod_reg_trib())
    }

    pub fn get_remetente_nome(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_rem_nome())
    }

    pub fn get_remetente_fantasia(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_rem_fantasia())
    }

    pub fn get_remetente_ender_municipio(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_rem_ender_municipio())
    }

    pub fn get_remetente_ender_estado(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_rem_ender_estado())
    }

    pub fn get_remetente_nfes(&self) -> Vec<String> {
        self.cte
            .inf_cte
            .iter()
            .flat_map(|info| info.get_rem_nfes())
            .collect()
    }

    // Destinarário

    pub fn get_destinatario_cnpj(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_dest_cnpj())
    }

    pub fn get_destinatario_cpf(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_dest_cpf())
    }

    pub fn get_destinatario_cod_regime_tributario(&self) -> Option<u8> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_dest_cod_reg_trib())
    }

    pub fn get_destinatario_nome(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_dest_nome())
    }

    pub fn get_destinatario_fantasia(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_dest_fantasia())
    }

    pub fn get_destinatario_ender_municipio(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_dest_ender_municipio())
    }

    pub fn get_destinatario_ender_estado(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_dest_ender_estado())
    }

    // Expedidor

    pub fn get_expedidor_cnpj(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_cnpj())
    }

    pub fn get_expedidor_cpf(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_cpf())
    }

    pub fn get_expedidor_cod_regime_tributario(&self) -> Option<u8> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_cod_reg_trib())
    }

    pub fn get_expedidor_nome(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_nome())
    }

    pub fn get_expedidor_fantasia(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_fantasia())
    }

    pub fn get_expedidor_ender_municipio(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_ender_municipio())
    }

    pub fn get_expedidor_ender_estado(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_exped_ender_estado())
    }

    // Recebedor

    pub fn get_recebedor_cnpj(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_cnpj())
    }

    pub fn get_recebedor_cpf(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_cpf())
    }

    pub fn get_recebedor_cod_regime_tributario(&self) -> Option<u8> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_cod_reg_trib())
    }

    pub fn get_recebedor_nome(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_nome())
    }

    pub fn get_recebedor_fantasia(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_fantasia())
    }

    pub fn get_recebedor_ender_municipio(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_ender_municipio())
    }

    pub fn get_recebedor_ender_estado(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_receb_ender_estado())
    }

    // Tomador

    pub fn get_tomador_cnpj(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.ide.get_toma_cnpj())
    }

    pub fn get_tomador_cpf(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.ide.get_toma_cpf())
    }

    pub fn get_tomador_nome(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.ide.get_toma_nome())
    }

    pub fn get_tomador_fantasia(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.ide.get_toma_fantasia())
    }

    pub fn get_tomador_ender_municipio(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.ide.get_toma_ender_municipio())
    }

    pub fn get_tomador_ender_estado(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.ide.get_toma_ender_estado())
    }

    pub fn get_tomador_codigo(&self) -> Option<u8> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|info| info.ide.get_cod_tomador())
    }

    pub fn get_numero_da_nota(&self) -> Option<u32> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|info| info.ide.get_num_cte())
    }

    pub fn get_cfop(&self) -> Option<u16> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|info| info.ide.get_cfop())
    }

    pub fn get_cte(&self) -> Option<String> {
        self.prot_cte.inf_prot.ch_cte.get_key()
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|info| info.ide.get_dt_emissao())
    }

    fn get_cte_multimodal(&self) -> Vec<String> {
        self.cte
            .inf_cte
            .iter()
            .flat_map(|info| info.get_cte_mult())
            .collect()
    }

    pub fn get_cte_complementar(&self) -> Vec<String> {
        let cte_multimodal: Vec<String> = self.get_cte_multimodal();

        let cte_complementar: Vec<String> = self
            .cte
            .inf_cte
            .iter()
            .flat_map(|info| info.get_cte_comp())
            .collect();

        [cte_multimodal, cte_complementar]
            .into_iter()
            .flatten()
            .collect()
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
                .flat_map(|info| {
                    info.get_info_de_ctes()
                })
                .collect::<Vec<String>>()
        */

        match &self.cte.inf_cte {
            Some(info) => info.get_info_de_ctes(),
            None => Vec::new(),
        }
    }

    pub fn get_nfes_vinculados(&self) -> Vec<String> {
        self.cte
            .inf_cte
            .iter()
            .flat_map(|info| info.get_info_de_nfes())
            .collect()
    }

    pub fn get_value_total(&self) -> Option<f64> {
        self.cte.inf_cte.as_ref().map(|info| info.v_prest.v_tprest)
    }

    pub fn get_imposto(&self) -> Option<&Imposto> {
        self.cte.inf_cte.as_ref().map(|inf| &inf.imposto)
    }

    pub fn get_info(&self) -> InfoCte {
        let imposto = self.get_imposto();
        //let produto = self.get_produto();
        let mut info_cte = InfoCte {
            versao: self.get_versao(),

            emitente_cnpj: self.get_emitente_cnpj(),
            emitente_cpf: self.get_emitente_cpf(),
            emitente_crt: self.get_emitente_cod_regime_tributario(),
            emitente_nome: self.get_emitente_nome(),
            emitente_fantasia: self.get_emitente_fantasia(),
            emitente_ender_municipio: self.get_emitente_ender_municipio(),
            emitente_ender_estado: self.get_emitente_ender_estado(),

            remetente_cnpj: self.get_remetente_cnpj(),
            remetente_cpf: self.get_remetente_cpf(),
            remetente_crt: self.get_remetente_cod_regime_tributario(),
            remetente_nome: self.get_remetente_nome(),
            remetente_fantasia: self.get_remetente_fantasia(),
            remetente_ender_municipio: self.get_remetente_ender_municipio(),
            remetente_ender_estado: self.get_remetente_ender_estado(),
            remetente_nfes: self.get_remetente_nfes(),

            destinatario_cnpj: self.get_destinatario_cnpj(),
            destinatario_cpf: self.get_destinatario_cpf(),
            destinatario_crt: self.get_destinatario_cod_regime_tributario(),
            destinatario_nome: self.get_destinatario_nome(),
            destinatario_fantasia: self.get_destinatario_fantasia(),
            destinatario_ender_municipio: self.get_destinatario_ender_municipio(),
            destinatario_ender_estado: self.get_destinatario_ender_estado(),

            expedidor_cnpj: self.get_expedidor_cnpj(),
            expedidor_cpf: self.get_expedidor_cpf(),
            expedidor_crt: self.get_expedidor_cod_regime_tributario(),
            expedidor_nome: self.get_expedidor_nome(),
            expedidor_fantasia: self.get_expedidor_fantasia(),
            expedidor_ender_municipio: self.get_expedidor_ender_municipio(),
            expedidor_ender_estado: self.get_expedidor_ender_estado(),

            recebedor_cnpj: self.get_recebedor_cnpj(),
            recebedor_cpf: self.get_recebedor_cpf(),
            recebedor_crt: self.get_recebedor_cod_regime_tributario(),
            recebedor_nome: self.get_recebedor_nome(),
            recebedor_fantasia: self.get_recebedor_fantasia(),
            recebedor_ender_municipio: self.get_recebedor_ender_municipio(),
            recebedor_ender_estado: self.get_recebedor_ender_estado(),

            tomador_cnpj: self.get_tomador_cnpj(),
            tomador_cpf: self.get_tomador_cpf(),
            tomador_nome: self.get_tomador_nome(),
            tomador_fantasia: self.get_tomador_fantasia(),
            tomador_ender_municipio: self.get_tomador_ender_municipio(),
            tomador_ender_estado: self.get_tomador_ender_estado(),
            tomador_codigo: self.get_tomador_codigo(), // Valor inicial sem correção!

            cte: self.get_cte(),
            doc_tipo: "CTe".to_string(),
            cancelado: None,
            numero_da_nota: self.get_numero_da_nota(),
            cfop: self.get_cfop(),
            data_emissao: self.get_data_emissao(),
            cte_complementar: self.get_cte_complementar(),
            cte_anteriores: self.get_cte_anteriores(),
            nfes_vinculados: self.get_nfes_vinculados(),
            nfes: Vec::new(),
            ncm_descricao: Vec::new(),
            valor_total_nfes: None,
            valor_total: self.get_value_total(),

            /*
            v_desc: produto.v_desc,
            v_frete: produto.v_frete,
            v_seg: produto.v_seg,
            */
            aliq_pis: imposto.and_then(|i| i.get_aliq_pis()),
            aliq_cofins: imposto.and_then(|i| i.get_aliq_cofins()),
            v_pis: imposto.and_then(|i| i.get_v_pis()),
            v_cofins: imposto.and_then(|i| i.get_v_cofins()),
            v_ipi: imposto.and_then(|i| i.get_v_ipi()),
            v_iss: imposto.and_then(|i| i.get_v_iss()),
            v_bc_icms: imposto.and_then(|i| i.get_v_bc_icms()),
            aliq_icms: imposto.and_then(|i| i.get_aliq_icms()),
            v_icms: imposto.and_then(|i| i.get_v_icms()),
        };

        info_cte.corrigir_codigo_do_tomador();
        info_cte
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
pub struct InfCte {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@versao")]
    pub versao: Option<String>,

    // 5 Agentes:
    #[serde(rename = "emit")]
    pub emitente: Option<Agente>,
    #[serde(rename = "dest")]
    pub destinatario: Option<Agente>,
    #[serde(rename = "exped")]
    pub expedidor: Option<Agente>,
    #[serde(rename = "receb")]
    pub recebedor: Option<Agente>,
    #[serde(rename = "rem")]
    pub remetente: Option<Agente>,

    #[serde(rename = "autXML")]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "compl")]
    pub compl: Option<Compl>,
    #[serde(rename = "ide")]
    pub ide: Ide,
    #[serde(rename = "imp")]
    pub imposto: Imposto,
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
    #[serde(rename = "vPrest")]
    pub v_prest: VPrest,
}

impl InfCte {
    /// Emitente: CNPJ
    fn get_emit_cnpj(&self) -> Option<String> {
        self.emitente.get_ext_cnpj()
    }

    /// Emitente: CPF
    fn get_emit_cpf(&self) -> Option<String> {
        self.emitente.get_ext_cpf()
    }

    /// Emitente: Código do Regime Tributário
    fn get_emit_cod_reg_trib(&self) -> Option<u8> {
        self.emitente.get_ext_crt()
    }

    /// Emitente: None ou Razão Social do Emitente
    fn get_emit_nome(&self) -> Option<String> {
        self.emitente.get_ext_nome()
    }

    /// Emitente: None Fantasia do Emitente
    fn get_emit_fantasia(&self) -> Option<String> {
        self.emitente.get_ext_fantasia()
    }

    /// Emitente: Endereço do Município
    fn get_emit_ender_municipio(&self) -> Option<String> {
        self.emitente.get_ext_municipio()
    }

    /// Emitente: Endereço do Estado
    fn get_emit_ender_estado(&self) -> Option<String> {
        self.emitente.get_ext_estado()
    }

    /// Remetente: CNPJ
    fn get_rem_cnpj(&self) -> Option<String> {
        self.remetente.get_ext_cnpj()
    }

    /// Remetente: CPF
    fn get_rem_cpf(&self) -> Option<String> {
        self.remetente.get_ext_cpf()
    }

    /// Remetente: Código do Regime Tributário
    fn get_rem_cod_reg_trib(&self) -> Option<u8> {
        self.remetente.get_ext_crt()
    }

    /// Remetente: None ou Razão Social do Remetente
    fn get_rem_nome(&self) -> Option<String> {
        self.remetente.get_ext_nome()
    }

    /// Remetente: None Fantasia do Remetente
    fn get_rem_fantasia(&self) -> Option<String> {
        self.remetente.get_ext_fantasia()
    }

    /// Remetente: Endereço do Município
    fn get_rem_ender_municipio(&self) -> Option<String> {
        self.remetente.get_ext_municipio()
    }

    /// Remetente: Endereço do Estado
    fn get_rem_ender_estado(&self) -> Option<String> {
        self.remetente.get_ext_estado()
    }

    /// Remetente: NFe
    fn get_rem_nfes(&self) -> Vec<String> {
        self.remetente.get_ext_chaves()
    }

    /// Destinatário: CNPJ
    fn get_dest_cnpj(&self) -> Option<String> {
        self.destinatario.get_ext_cnpj()
    }

    /// Destinatário: CPF
    fn get_dest_cpf(&self) -> Option<String> {
        self.destinatario.get_ext_cpf()
    }

    /// Destinatário: Código do Regime Tributário
    fn get_dest_cod_reg_trib(&self) -> Option<u8> {
        self.destinatario.get_ext_crt()
    }

    /// Destinatário: Nome
    fn get_dest_nome(&self) -> Option<String> {
        self.destinatario.get_ext_nome()
    }

    /// Destinatário: None Fantasia do Destinatário
    fn get_dest_fantasia(&self) -> Option<String> {
        self.destinatario.get_ext_fantasia()
    }

    /// Destinatário: Endereço do Município
    fn get_dest_ender_municipio(&self) -> Option<String> {
        self.destinatario.get_ext_municipio()
    }

    /// Destinatário: Endereço do Estado
    fn get_dest_ender_estado(&self) -> Option<String> {
        self.destinatario.get_ext_estado()
    }

    /// Expedidor: CNPJ
    fn get_exped_cnpj(&self) -> Option<String> {
        self.expedidor.get_ext_cnpj()
    }

    /// Expedidor: CPF
    fn get_exped_cpf(&self) -> Option<String> {
        self.expedidor.get_ext_cpf()
    }

    /// Expedidor: Código do Regime Tributário
    fn get_exped_cod_reg_trib(&self) -> Option<u8> {
        self.expedidor.get_ext_crt()
    }

    /// Expedidor: Nome
    fn get_exped_nome(&self) -> Option<String> {
        self.expedidor.get_ext_nome()
    }

    /// Expedidor: None Fantasia do Expedidor
    fn get_exped_fantasia(&self) -> Option<String> {
        self.expedidor.get_ext_fantasia()
    }

    /// Expedidor: Endereço do Município
    fn get_exped_ender_municipio(&self) -> Option<String> {
        self.expedidor.get_ext_municipio()
    }

    /// Expedidor: Endereço do Estado
    fn get_exped_ender_estado(&self) -> Option<String> {
        self.expedidor.get_ext_estado()
    }

    /// Recebedor: CNPJ
    fn get_receb_cnpj(&self) -> Option<String> {
        self.recebedor.get_ext_cnpj()
    }

    /// Recebedor: CPF
    fn get_receb_cpf(&self) -> Option<String> {
        self.recebedor.get_ext_cpf()
    }

    /// Recebedor: Código do Regime Tributário
    fn get_receb_cod_reg_trib(&self) -> Option<u8> {
        self.recebedor.get_ext_crt()
    }

    /// Recebedor: Nome
    fn get_receb_nome(&self) -> Option<String> {
        self.recebedor.get_ext_nome()
    }

    /// Recebedor: None Fantasia do Recebedor
    fn get_receb_fantasia(&self) -> Option<String> {
        self.recebedor.get_ext_fantasia()
    }

    /// Recebedor: Endereço do Município
    fn get_receb_ender_municipio(&self) -> Option<String> {
        self.recebedor.get_ext_municipio()
    }

    /// Recebedor: Endereço do Estado
    fn get_receb_ender_estado(&self) -> Option<String> {
        self.recebedor.get_ext_estado()
    }

    fn get_cte_mult(&self) -> Vec<String> {
        self.inf_cte_norm
            .iter()
            .flat_map(|info_normal| {
                info_normal
                    .inf_serv_vinc
                    .iter()
                    .flat_map(|serv| serv.get_ctes_multimodal())
            })
            .collect()
    }

    fn get_cte_comp(&self) -> Vec<String> {
        self.inf_cte_comp
            .iter()
            .flat_map(|info_complementar| info_complementar.merge_keys())
            .collect()
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
    pub obs_fisco: Option<Vec<ObsFisco>>,
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
pub struct ObsCont {
    #[serde(rename = "@xCampo")]
    pub x_campo: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xTexto")]
    pub x_texto: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsFisco {
    #[serde(rename = "@xCampo")]
    pub x_campo: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xTexto")]
    pub x_texto: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fluxo {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "pass")]
    pub pass: Option<Vec<Pass>>,
    #[serde(rename = "xDest")]
    pub x_dest: Option<String>,
    #[serde(rename = "xOrig")]
    pub x_orig: Option<String>,
    #[serde(rename = "xRota")]
    pub x_rota: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pass {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xPass")]
    pub x_pass: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteNorm {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "cobr")]
    pub cobr: Option<Cobranca>,
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
    pub veic_novos: Option<Vec<VeicNovos>>,
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
pub struct DocAnt {
    #[serde(rename = "emiDocAnt")]
    pub emi_doc_ant: Option<Vec<EmiDocAnt>>,
}

impl DocAnt {
    fn get_emissao_de_docs_anteriores(&self) -> Vec<String> {
        self.emi_doc_ant
            .iter()
            .flat_map(|vec_doc_anterior| {
                vec_doc_anterior
                    .iter()
                    .flat_map(|emi| emi.get_id_de_docs_anteriores())
            })
            .collect()
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
    pub id_doc_ant: Option<Vec<IdDocAnt>>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
}

impl EmiDocAnt {
    fn get_id_de_docs_anteriores(&self) -> Vec<String> {
        self.id_doc_ant
            .iter()
            .flat_map(|vec_doc_anterior| {
                vec_doc_anterior
                    .iter()
                    .flat_map(|id| id.get_docs_anteriores_eletronicos())
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdDocAnt {
    #[serde(rename = "idDocAntEle")]
    pub id_doc_ant_ele: Option<Vec<IdDocAntEle>>,
    #[serde(rename = "idDocAntPap")]
    pub id_doc_ant_pap: Option<Vec<IdDocAntPap>>,
}

impl IdDocAnt {
    fn get_docs_anteriores_eletronicos(&self) -> Vec<String> {
        self.id_doc_ant_ele
            .iter()
            .flat_map(|vec_doc_anterior| vec_doc_anterior.iter().flat_map(|doc| doc.merge_keys()))
            .collect()
    }
}

// https://github.com/rust-lang/rfcs/pull/1546
// https://stackoverflow.com/questions/28219730/is-it-possible-to-access-struct-fields-from-within-a-trait

pub trait GetDocs {
    fn get_ch_cte(&self) -> Option<String>;

    fn get_chave(&self) -> Option<String>;

    fn merge_keys(&self) -> Vec<String> {
        let ch_cte: Option<String> = self.get_ch_cte();
        let chave: Option<String> = self.get_chave();

        let docs: Vec<String> = [ch_cte, chave].into_iter().flatten().collect();

        docs
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdDocAntEle {
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    pub chave: Option<String>,
}

impl GetDocs for IdDocAntEle {
    fn get_ch_cte(&self) -> Option<String> {
        self.ch_cte.get_key()
    }

    fn get_chave(&self) -> Option<String> {
        self.chave.get_key()
    }
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
    pub pro_pred: Option<String>,
    #[serde(rename = "vCarga")]
    pub v_carga: Option<String>,
    #[serde(rename = "vCargaAverb")]
    pub v_carga_averb: Option<String>,
    #[serde(rename = "xOutCat")]
    pub x_out_cat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfQ {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "cUnid")]
    pub c_unid: Option<String>,
    #[serde(rename = "qCarga")]
    pub q_carga: Option<String>,
    #[serde(rename = "tpMed")]
    pub tp_med: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteSub {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "chCte")]
    pub ch_cte: String,
    #[serde(rename = "indAlteraToma")]
    pub ind_altera_toma: Option<String>,
    #[serde(rename = "refCteAnu")]
    pub ref_cte_anu: Option<String>,
    #[serde(rename = "tomaICMS")]
    pub toma_icms: Option<TomaIcms>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TomaIcms {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "refCte")]
    pub ref_cte: Option<String>,
    #[serde(rename = "refNF")]
    pub ref_nf: Option<RefNf>,
    #[serde(rename = "refNFe")]
    pub ref_nfe: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefNf {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "dEmi")]
    pub d_emi: Option<String>,
    #[serde(rename = "mod")]
    pub modelo: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "subserie")]
    pub subserie: Option<String>,
    #[serde(rename = "valor")]
    pub valor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDoc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infNF")]
    pub inf_nf: Option<Vec<InfNf>>,
    #[serde(rename = "infNFe")]
    pub inf_nfe: Option<Vec<InfNfe>>,
    #[serde(rename = "infOutros")]
    pub inf_outros: Option<Vec<InfOutros>>,
}

impl InfDoc {
    fn get_chaves_de_nfes(&self) -> Vec<String> {
        match self.inf_nfe.as_ref() {
            Some(vec_info_nfe) => vec_info_nfe
                .iter()
                .map(|i| i.chave.to_string())
                .collect::<Vec<String>>(),
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
    pub inf_unid_carga: Option<Vec<InfDocCteInfNfCteInfUnidCarga>>,
    #[serde(rename = "infUnidTransp")]
    pub inf_unid_transp: Option<Vec<InfNfCteInfUnidTransp>>,
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
    pub lac_unid_carga: Option<Vec<InfDocCteInfNfCteInfUnidCargaCteLacUnidCarga>>,
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
    pub lac_unid_carga: Option<Vec<InfNfCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga>>,
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
    pub inf_unid_carga: Option<Vec<InfDocCteInfNfeCteInfUnidCarga>>,
    #[serde(rename = "infUnidTransp")]
    pub inf_unid_transp: Option<Vec<InfNfeCteInfUnidTransp>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfDocCteInfNfeCteInfUnidCarga {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "idUnidCarga")]
    pub id_unid_carga: Option<String>,
    #[serde(rename = "lacUnidCarga")]
    pub lac_unid_carga: Option<Vec<InfDocCteInfNfeCteInfUnidCargaCteLacUnidCarga>>,
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
    pub lac_unid_carga: Option<Vec<InfNfeCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga>>,
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
    pub inf_unid_carga: Option<Vec<InfDocCteInfOutrosCteInfUnidCarga>>,
    #[serde(rename = "infUnidTransp")]
    pub inf_unid_transp: Option<Vec<InfOutrosCteInfUnidTransp>>,
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
    pub lac_unid_carga: Option<Vec<InfDocCteInfOutrosCteInfUnidCargaCteLacUnidCarga>>,
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
    pub lac_unid_carga: Option<Vec<InfOutrosCteInfUnidTranspCteInfUnidCargaCteLacUnidCarga>>,
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
pub struct AnyElement {}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfServVinc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infCTeMultimodal")]
    pub inf_cte_multimodal: Option<Vec<InfCteMultimodal>>,
}

impl InfServVinc {
    fn get_ctes_multimodal(&self) -> Vec<String> {
        self.inf_cte_multimodal
            .iter()
            .flat_map(|vec_cte_multimodal| {
                vec_cte_multimodal
                    .iter()
                    .flat_map(|info| info.get_ch_cte_multimodal())
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteMultimodal {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "chCTeMultimodal")]
    pub ch_cte_multimodal: Option<String>,
}

impl InfCteMultimodal {
    pub fn get_ch_cte_multimodal(&self) -> Option<String> {
        self.ch_cte_multimodal.get_key()
    }
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
    #[serde(rename = "chCTe")]
    pub ch_cte: Option<String>,
    pub chave: Option<String>,
}

impl GetDocs for InfCteComp {
    fn get_ch_cte(&self) -> Option<String> {
        self.ch_cte.get_key()
    }

    fn get_chave(&self) -> Option<String> {
        self.chave.get_key()
    }
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
    pub signature_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RsakeyValue {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Exponent")]
    pub exponent: Option<String>,
    #[serde(rename = "Modulus")]
    pub modulus: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfSolicNff {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "xSolic")]
    pub x_solic: Option<String>,
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
    pub v_comp: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtCte {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "infProt")]
    pub inf_prot: InfProtocolo,
    #[serde(rename = "Signature")]
    pub signature: Option<ProtSignature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfCteSupl {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "qrCodCTe")]
    pub qr_cod_cte: Option<String>,
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
    fn deserialize_xml_cte() -> MyResult<()> {
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

            let nfes: Vec<String> = cte_proc.get_nfes_vinculados();
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
    fn documentos_anteriores_eletronicos() -> MyResult<()> {
        let doc_anterior = IdDocAnt {
            id_doc_ant_ele: None,
            id_doc_ant_pap: None,
        };

        let docs_01 = doc_anterior.get_docs_anteriores_eletronicos();

        println!("docs_01 empty: {docs_01:?}");

        let doc_ant_ele_a = IdDocAntEle {
            ch_cte: Some("001".to_string()),
            chave: None,
        };

        let doc_ant_ele_b = IdDocAntEle {
            ch_cte: Some("002".to_string()),
            chave: None,
        };

        let doc_ant_ele_c = IdDocAntEle {
            ch_cte: Some("003".to_string()),
            chave: None,
        };

        let docs_eletronicos: Vec<IdDocAntEle> = vec![doc_ant_ele_a, doc_ant_ele_b, doc_ant_ele_c];

        let doc_anterior = IdDocAnt {
            id_doc_ant_ele: Some(docs_eletronicos),
            id_doc_ant_pap: None,
        };

        let docs_02 = doc_anterior.get_docs_anteriores_eletronicos();

        println!("docs_02: {docs_02:?}");

        let result_01: Vec<String> = Vec::new();

        let result_02 = ["001", "002", "003"];

        assert_eq!(docs_01, result_01);
        assert_eq!(docs_02, result_02);

        Ok(())
    }
}
