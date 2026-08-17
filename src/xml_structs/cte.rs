//! # Processamento de XML de CT-e (versões 3.00 e 4.00)
//!
//! Módulo gerado originalmente a partir de múltiplos esquemas XML (arquivos `.xsd`)
//! para o Conhecimento de Transporte Eletrônico (CT-e). Fornece suporte robusto e retrocompatível
//! para as versões v3.00 e v4.00 ativas e em vigência no Brasil.
//!
//! O design tolerante a desordenamentos de tags e descontinuidades regulatórias é assegurado pelo
//! zoneamento de campos e pelo uso sistemático do atributo `default` em wrappers `Option<T>`.
//!
//! Foram utilizadas ferramentas do Apache XMLBeans (`xsd2inst`) para geração de instâncias
//! e o `read_xml` com a opção `-s` para conversão em estruturas Rust de desserialização.
//!
//! Generates XML file from Multiple XML schemas (xsd files).
//!
//! This feature requires Apache XMLBeans.
//! https://xmlbeans.apache.org/docs/2.0.0/guide/tools.html#xsd2inst
//! xsd2inst (Schema to Instance Tool)
//! Prints an XML instance from the specified global element using the specified schema.
//!
//! XSD to XML:
//! /home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst procCTe_v3.00.xsd -name cteProc -dl > procCTe_v3.00.xml
//!
//! Converter XML file to Rust struct:
//! read_xml -s procCTe_v3.00.xml > procCTe_v3.00.rs
//!
//! Mais informações em:
//! * <https://dfe-portal.svrs.rs.gov.br/Cte>
//! * <https://dfe-portal.svrs.rs.gov.br/CTE/ConsultaSchema>
//!
//! No Manjaro Linux, install qxmledit:
//! yay -S qxmledit
//! Make PDF files from XSD.

use chrono::NaiveDate;
use claudiofsr_lib::{BTreeSetExtension, UniqueElements};
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeSet;
use struct_iterable::Iterable;

use crate::{
    Arguments, GetFirst, GetID, GetKey, InfoExtension, Information, KeysExtension, OptExt,
    StructExtension, serialize_vec_string,
    xml_structs::{
        agente::{Agente, AgenteExtension, TOMADOR_DO_SERVICO},
        assinaturas::{ProtSignature, Signature},
        aut_xml::{AutXML, InfProtocolo, InfRespTec},
        // Re-exporta as estruturas de detalhamento secundárias do arquivo normalizado
        cte_detalhamento::*,
        impostos::Imposto,
        integrated_dev_env::Ide,
    },
};

/// Representação intermediária e unificada de um Conhecimento de Transporte Eletrônico (CT-e).
///
/// Esta estrutura centraliza os dados extraídos de diversas seções do XML do CT-e
/// para posterior exportação em formatos tabulares como planilhas Excel (XLSX) ou CSV.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoCte {
    /// Versão do layout declarada no documento fiscal.
    #[serde(rename = "Versão XML", default)]
    versao: Option<String>,

    /// CNPJ do Emitente formatado.
    #[serde(rename = "CNPJ do Emitente", default)]
    pub emitente_cnpj: Option<String>,

    /// CPF do Emitente formatado.
    #[serde(rename = "CPF do Emitente", default)]
    pub emitente_cpf: Option<String>,

    /// Código do Regime Tributário (CRT) do Emitente.
    #[serde(
        rename = "CRT (Código do Regime Tributário) conforme Emitente",
        default
    )]
    emitente_crt: Option<u8>,

    /// Nome ou Razão Social do Emitente.
    #[serde(rename = "Nome ou Razão Social do Emitente", default)]
    emitente_nome: Option<String>,

    /// Nome Fantasia do Emitente.
    #[serde(rename = "Nome Fantasia do Emitente", default)]
    emitente_fantasia: Option<String>,

    /// Município do endereço do Emitente.
    #[serde(rename = "Municípo do Emitente", default)]
    emitente_ender_municipio: Option<String>,

    /// Estado (UF) do Emitente.
    #[serde(rename = "Estado do Emitente", default)]
    emitente_ender_estado: Option<String>,

    /// CNPJ do Remetente formatado.
    #[serde(rename = "CNPJ do Remetente", default)]
    pub remetente_cnpj: Option<String>,

    /// CPF do Remetente formatado.
    #[serde(rename = "CPF do Remetente", default)]
    pub remetente_cpf: Option<String>,

    /// CRT do Remetente.
    #[serde(
        rename = "CRT (Código do Regime Tributário) conforme Remetente",
        default
    )]
    remetente_crt: Option<u8>,

    /// Nome ou Razão Social do Remetente.
    #[serde(rename = "Nome ou Razão Social do Remetente", default)]
    remetente_nome: Option<String>,

    /// Nome Fantasia do Remetente.
    #[serde(rename = "Nome Fantasia do Remetente", default)]
    remetente_fantasia: Option<String>,

    /// Município do endereço do Remetente.
    #[serde(rename = "Municípo do Remetente", default)]
    remetente_ender_municipio: Option<String>,

    /// Estado (UF) do Remetente.
    #[serde(rename = "Estado do Remetente", default)]
    remetente_ender_estado: Option<String>,

    /// Chaves das NF-es associadas ao Remetente.
    #[serde(
        rename = "NFes do Remetente",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub remetente_nfes: Vec<String>,

    /// CNPJ do Destinatário formatado.
    #[serde(rename = "CNPJ do Destinatário", default)]
    pub destinatario_cnpj: Option<String>,

    /// CPF do Destinatário formatado.
    #[serde(rename = "CPF do Destinatário", default)]
    pub destinatario_cpf: Option<String>,

    /// CRT do Destinatário.
    #[serde(
        rename = "CRT (Código do Regime Tributário) conforme Destinatário",
        default
    )]
    destinatario_crt: Option<u8>,

    /// Nome ou Razão Social do Destinatário.
    #[serde(rename = "Nome ou Razão Social do Destinatário", default)]
    destinatario_nome: Option<String>,

    /// Nome Fantasia do Destinatário.
    #[serde(rename = "Nome Fantasia do Destinatário", default)]
    destinatario_fantasia: Option<String>,

    /// Município do endereço do Destinatário.
    #[serde(rename = "Municípo do Destinatário", default)]
    destinatario_ender_municipio: Option<String>,

    /// Estado (UF) do Destinatário.
    #[serde(rename = "Estado do Destinatário", default)]
    destinatario_ender_estado: Option<String>,

    /// CNPJ do Expedidor formatado.
    #[serde(rename = "CNPJ do Expedidor", default)]
    pub expedidor_cnpj: Option<String>,

    /// CPF do Expedidor formatado.
    #[serde(rename = "CPF do Expedidor", default)]
    pub expedidor_cpf: Option<String>,

    /// CRT do Expedidor.
    #[serde(
        rename = "CRT (Código do Regime Tributário) conforme Expedidor",
        default
    )]
    expedidor_crt: Option<u8>,

    /// Nome ou Razão Social do Expedidor.
    #[serde(rename = "Nome ou Razão Social do Expedidor", default)]
    expedidor_nome: Option<String>,

    /// Nome Fantasia do Expedidor.
    #[serde(rename = "Nome Fantasia do Expedidor", default)]
    expedidor_fantasia: Option<String>,

    /// Município do endereço do Expedidor.
    #[serde(rename = "Municípo do Expedidor", default)]
    expedidor_ender_municipio: Option<String>,

    /// Estado (UF) do Expedidor.
    #[serde(rename = "Estado do Expedidor", default)]
    expedidor_ender_estado: Option<String>,

    /// CNPJ do Recebedor formatado.
    #[serde(rename = "CNPJ do Recebedor", default)]
    pub recebedor_cnpj: Option<String>,

    /// CPF do Recebedor formatado.
    #[serde(rename = "CPF do Recebedor", default)]
    pub recebedor_cpf: Option<String>,

    /// CRT do Recebedor.
    #[serde(
        rename = "CRT (Código do Regime Tributário) conforme Recebedor",
        default
    )]
    recebedor_crt: Option<u8>,

    /// Nome ou Razão Social do Recebedor.
    #[serde(rename = "Nome ou Razão Social do Recebedor", default)]
    recebedor_nome: Option<String>,

    /// Nome Fantasia do Recebedor.
    #[serde(rename = "Nome Fantasia do Recebedor", default)]
    recebedor_fantasia: Option<String>,

    /// Município do endereço do Recebedor.
    #[serde(rename = "Municípo do Recebedor", default)]
    recebedor_ender_municipio: Option<String>,

    /// Estado (UF) do Recebedor.
    #[serde(rename = "Estado do Recebedor", default)]
    recebedor_ender_estado: Option<String>,

    /// CNPJ do Tomador do serviço formatado.
    #[serde(rename = "CNPJ do Tomador", default)]
    tomador_cnpj: Option<String>,

    /// CPF do Tomador do serviço formatado.
    #[serde(rename = "CPF do Tomador", default)]
    tomador_cpf: Option<String>,

    /// Nome ou Razão Social do Tomador.
    #[serde(rename = "Nome ou Razão Social do Tomador", default)]
    tomador_nome: Option<String>,

    /// Nome Fantasia do Tomador.
    #[serde(rename = "Nome Fantasia do Tomador", default)]
    tomador_fantasia: Option<String>,

    /// Município do endereço do Tomador.
    #[serde(rename = "Municípo do Tomador", default)]
    tomador_ender_municipio: Option<String>,

    /// Estado (UF) do Tomador.
    #[serde(rename = "Estado do Tomador", default)]
    tomador_ender_estado: Option<String>,

    /// Código do papel do Tomador responsável pelo pagamento do transporte.
    #[serde(
        rename = "Responsável pelo pagamento do Transporte: Tomador",
        serialize_with = "serialize_tomador_do_servico",
        default
    )]
    pub tomador_codigo: Option<u8>,

    /// Chave de acesso única do CT-e (44 dígitos).
    #[serde(rename = "Chave do Documento Fiscal", default)]
    pub cte: Option<String>,

    /// Tipo de documento de origem (ex: "CTe").
    #[serde(rename = "Registro de Origem")]
    doc_tipo: String,

    /// Indicador se o documento foi cancelado.
    #[serde(rename = "Cancelado", default)]
    pub cancelado: Option<String>,

    /// Número sequencial da Nota Fiscal/Documento.
    #[serde(rename = "Nº do Documento Fiscal", default)]
    numero_da_nota: Option<u32>,

    /// Código CFOP aplicável ao serviço.
    #[serde(rename = "CFOP (Código Fiscal de Operações e Prestações)", default)]
    cfop: Option<u16>,

    /// Data de emissão oficial do CT-e.
    #[serde(
        rename = "Data de Emissão",
        serialize_with = "serialize_option_datetime_to_excel",
        default
    )]
    pub data_emissao: Option<NaiveDate>,

    /// Chaves de CT-es complementares e relacionados.
    #[serde(
        rename = "CTe Multimodal, Complementar, Redespacho, Subcontratação, Substituição ou Vinculado",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub cte_complementar: Vec<String>,

    /// Chaves de CT-es anteriores referenciados.
    #[serde(
        rename = "CTes Anteriores",
        serialize_with = "serialize_vec_string",
        default
    )]
    cte_anteriores: Vec<String>,

    /// Chaves de NF-es de terceiros vinculadas à operação.
    #[serde(
        rename = "NFes Vinculados",
        serialize_with = "serialize_vec_string",
        default
    )]
    nfes_vinculados: Vec<String>,

    /// Chaves de NF-es de terceiros cruzadas e correlacionadas.
    #[serde(
        rename = "Informações de NFes relacionados",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub nfes: Vec<String>,

    /// Agrupamento ordenado de NCM e descrição de itens fiscais das NF-es.
    #[serde(
        rename = "(NCM e Descrição) dos Itens de NFes com Valores e Porcentagens decrescentes",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub ncm_descricao: Vec<String>,

    /// Valor monetário total somado de todas as NF-es transportadas.
    #[serde(rename = "Valor Total de NFes", default)]
    pub valor_total_nfes: Option<f64>,

    /// Valor de prestação total apurado para o serviço de transporte (vPrest).
    #[serde(rename = "Valor Total do CTe", default)]
    pub valor_total: Option<f64>,

    /// Alíquota correspondente ao imposto PIS.
    #[serde(rename = "Alíquota de PIS/PASEP", default)]
    aliq_pis: Option<f64>,

    /// Alíquota correspondente ao imposto COFINS.
    #[serde(rename = "Alíquota de COFINS", default)]
    aliq_cofins: Option<f64>,

    /// Valor monetário calculado de PIS.
    #[serde(rename = "Valor de PIS/PASEP", default)]
    v_pis: Option<f64>,

    /// Valor monetário calculado de COFINS.
    #[serde(rename = "Valor de COFINS", default)]
    v_cofins: Option<f64>,

    /// Valor monetário calculado de IPI se aplicável.
    #[serde(rename = "Valor de IPI", default)]
    v_ipi: Option<f64>,

    /// Valor monetário calculado de ISS se aplicável.
    #[serde(rename = "Valor de ISS", default)]
    v_iss: Option<f64>,

    /// Base de cálculo monetária apurada para o ICMS de transporte.
    #[serde(rename = "Valor da Base de Cálculo do ICMS", default)]
    v_bc_icms: Option<f64>,

    /// Alíquota correspondente ao ICMS de transporte.
    #[serde(rename = "Alíquota de ICMS", default)]
    aliq_icms: Option<f64>,

    /// Valor calculado correspondente ao ICMS debitado.
    #[serde(rename = "Valor de ICMS", default)]
    v_icms: Option<f64>,
}

impl InfoCte {
    /// Obtém a chave de acesso única do CT-e se presente.
    pub fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }

    /// Determina se o documento de transporte é válido.
    pub fn is_valid(&self) -> bool {
        self.cte.is_some() && self.cancelado.is_none()
    }

    /// Determina se o documento de transporte foi cancelado.
    pub fn is_canceled(&self) -> bool {
        self.cte.is_some() && self.cancelado.is_some()
    }

    /// Recupera a identificação base do Remetente (CNPJ ou CPF).
    fn get_base_remetente(&self) -> Option<String> {
        [
            self.remetente_cnpj.get_first(10),
            self.remetente_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x)
        .get_not_empty()
    }

    /// Recupera a identificação base do Emitente (CNPJ ou CPF).
    fn get_base_emitente(&self) -> Option<String> {
        [
            self.emitente_cnpj.get_first(10),
            self.emitente_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x)
        .get_not_empty()
    }

    /// Recupera a identificação base do Recebedor (CNPJ ou CPF).
    fn get_base_recebedor(&self) -> Option<String> {
        [
            self.recebedor_cnpj.get_first(10),
            self.recebedor_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x)
        .get_not_empty()
    }

    /// Recupera a identificação base do Destinatário (CNPJ ou CPF).
    fn get_base_destinatario(&self) -> Option<String> {
        [
            self.destinatario_cnpj.get_first(10),
            self.destinatario_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x)
        .get_not_empty()
    }

    /// Recupera a identificação base do Tomador (CNPJ ou CPF).
    fn get_base_tomador(&self) -> Option<String> {
        [
            self.tomador_cnpj.get_first(10),
            self.tomador_cpf.get_first(11),
        ]
        .into_iter()
        .find_map(|x| x)
        .get_not_empty()
    }

    /// Corrige e alinha o código do Tomador comparando seu CNPJ/CPF base com os demais agentes.
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

    /// Retorna o CNPJ ou CPF base inferido correspondente ao tomador após correção de papéis.
    pub fn get_cnpj_cpf_base_do_tomador(&self) -> Option<String> {
        match &self.tomador_codigo {
            Some(0) => self.get_base_remetente(),
            Some(1) => self.get_base_emitente(),
            Some(2) => self.get_base_recebedor(),
            Some(3) => self.get_base_destinatario(),
            Some(4) => self.get_base_tomador(),
            _ => None,
        }
    }

    /// Remove duplicidades e ordena os vetores internos de referências de forma concorrente.
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

    /// Consolida as chaves de todos os CT-es correlacionados declarados.
    pub fn get_correlated_ctes(&self) -> Vec<String> {
        [&self.cte_complementar, &self.cte_anteriores]
            .into_iter()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .to_vec()
    }

    /// Consolida as chaves de todas as NF-es correlacionadas declaradas.
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
    /// Extrai e consolida as chaves de acesso únicas de uma listagem de CT-es.
    fn get_chaves(&self) -> BTreeSet<String> {
        self.iter().flat_map(|info| info.get_chave()).collect()
    }
}

impl InfoExtension for InfoCte {}

impl GetKey for InfoCte {
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

impl GetID<Option<String>> for InfoCte {
    fn get_id(&self) -> Option<String> {
        self.cte.clone()
    }
}

/// Serializador customizado para expor o papel do Tomador do Serviço em formato textual legível.
pub fn serialize_tomador_do_servico<S>(
    codigo: &Option<u8>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(cod) = codigo {
        let opt_tomador = TOMADOR_DO_SERVICO.get(&Some(*cod)).map(|&t| t.to_string());
        match opt_tomador {
            Some(t) => serializer.serialize_some(&t),
            None => serializer.serialize_none(),
        }
    } else {
        serializer.serialize_none()
    }
}

/// Mapeamento do envelope XML correspondente ao processamento de um CT-e (`<cteProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteProc {
    // Atributos de metadados de rede/protocolo
    #[serde(rename = "@dhConexao", default)]
    pub dh_conexao: Option<String>,
    #[serde(rename = "@ipTransmissor", default)]
    pub ip_transmissor: Option<String>,
    #[serde(rename = "@nPortaCon", default)]
    pub n_porta_con: Option<String>,
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // Nós filhos principais
    #[serde(rename = "CTe")]
    pub cte: Cte,
    #[serde(rename = "protCTe")]
    pub prot_cte: ProtCte,
}

impl StructExtension for CteProc {
    /// Converte as estruturas processadas do XML do CT-e para o enum centralizador `Information`.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cte xml_path: {xml_path:?}");
            println!("cte_proc: {self:#?}\n");
        }
        Information::Cte(Box::new(self.get_info()))
    }
}

impl CteProc {
    /// Retorna a versão de leiaute declarada no CT-e.
    pub fn get_versao(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.versao.clone())
    }

    // Emitente de serviços de transporte

    pub fn get_emitente_cnpj(&self) -> Option<String> {
        self.cte
            .inf_cte
            .as_ref()
            .and_then(|inf| inf.get_emit_cnpj())
    }

    pub fn get_emitente_cpf(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_emit_cpf())
    }

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

    // Remetente de mercadorias

    pub fn get_remetente_cnpj(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_rem_cnpj())
    }

    pub fn get_remetente_cpf(&self) -> Option<String> {
        self.cte.inf_cte.as_ref().and_then(|inf| inf.get_rem_cpf())
    }

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

    // Destinatário de mercadorias

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

    // Expedidor de mercadorias

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

    // Recebedor de mercadorias

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

    // Tomador do serviço de frete

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
        let cte_multimodal = self.get_cte_multimodal();
        let cte_complementar = self
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

    pub fn get_cte_anteriores(&self) -> Vec<String> {
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

    /// Executa a extração do subnó `InfCte` e monta a estrutura `InfoCte` consolidada.
    pub fn get_info(&self) -> InfoCte {
        let imposto = self.get_imposto();
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
            tomador_codigo: self.get_tomador_codigo(),

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

/// Representação estrutural direta da tag de envelope `<CTe>`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cte {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "infCte", default)]
    pub inf_cte: Option<InfCte>,
    #[serde(rename = "infCTeSupl", default)]
    pub inf_cte_supl: Option<InfCteSupl>,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

/// Representação direta da estrutura interna `<infCte>` do XML de um CT-e.
///
/// ### Arquitetura Tolerante a Ordenação (Order-Independent)
///
/// Por padrão, os analisadores baseados em stream sequencial processam elementos XML
/// de forma estritamente posicional. No entanto, os arquivos XML do CT-e podem apresentar
/// variações na ordenação de suas tags.
///
/// Para sanar essa limitação sem a necessidade de acoplamento rígido de layout, esta
/// estrutura adota o seguinte design com o Serde:
///
/// 1. **Atributos no Topo**: Mapeia metadados diretos do nó de abertura (`@Id`, `@versao`).
/// 2. **Campos Opcionais no Centro (`#[serde(default)]`)**: Todos os campos que podem ou não
///    aparecer estão marcados com `default`. Isso instrui o Serde a ler as tags como se fossem
///    chaves de um dicionário dinâmico. Se uma tag não for encontrada durante a leitura da stream,
///    seu valor é automaticamente preenchido como `None` (ou vetor vazio), evitando quebras de fluxo.
/// 3. **Campos Obrigatórios na Base**: Campos estritamente necessários (`ide`, `imp`, `vPrest`)
///    estão na base para validação final.
///
/// Com esta abordagem, a ordem das declarações na struct Rust é **totalmente irrelevante**
/// para a obtenção bem-sucedida de todos os valores informados no XML.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCte {
    // Atributos do nó XML (sempre mapeados no início da leitura da tag)
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    // Campos opcionais tolerantes a qualquer ordem ou ausência no XML

    // 5 Agentes:
    #[serde(rename = "emit", default)]
    pub emitente: Option<Agente>,
    #[serde(rename = "dest", default)]
    pub destinatario: Option<Agente>,
    #[serde(rename = "exped", default)]
    pub expedidor: Option<Agente>,
    #[serde(rename = "receb", default)]
    pub recebedor: Option<Agente>,
    #[serde(rename = "rem", default)]
    pub remetente: Option<Agente>,

    #[serde(rename = "autXML", default)]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "compl", default)]
    pub compl: Option<Compl>,
    #[serde(rename = "infCTeNorm", default)]
    pub inf_cte_norm: Option<InfCteNorm>,
    #[serde(rename = "infCteAnu", default)]
    pub inf_cte_anu: Option<InfCteAnu>,
    #[serde(rename = "infCteComp", default)]
    pub inf_cte_comp: Option<Vec<InfCteComp>>,
    #[serde(rename = "infPAA", default)]
    pub inf_paa: Option<InfPaa>,
    #[serde(rename = "infRespTec", default)]
    pub inf_resp_tec: Option<InfRespTec>,
    #[serde(rename = "infSolicNFF", default)]
    pub inf_solic_nff: Option<InfSolicNff>,

    // Campos estritamente obrigatórios (devem aparecer no XML, mas em qualquer posição)
    #[serde(rename = "ide")]
    pub ide: Ide,
    #[serde(rename = "imp")]
    pub imposto: Imposto,
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
            .flatten()
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

/// Bloco de Valores da Prestação de Serviço de Transporte (`<vPrest>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VPrest {
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    #[serde(rename = "Comp", default)]
    pub comp: Option<Vec<Comp>>,
    #[serde(rename = "vRec")]
    pub v_rec: f64,
    #[serde(rename = "vTPrest")]
    pub v_tprest: f64,
}

/// Componente de Valor da Prestação (`<Comp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comp {
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    #[serde(rename = "vComp", default)]
    pub v_comp: Option<String>,
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
}

/// Protocolo de Distribuição e Autorização de uso do CT-e (`<protCTe>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtCte {
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    #[serde(rename = "infProt")]
    pub inf_prot: InfProtocolo,
    #[serde(rename = "Signature", default)]
    pub signature: Option<ProtSignature>,
}

/// Informações complementares de distribuição do CT-e (`<infCTeSupl>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteSupl {
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    #[serde(rename = "qrCodCTe", default)]
    pub qr_cod_cte: Option<String>,
}

#[cfg(test)]
mod test_functions {
    use super::*;
    use crate::XmlParserResult;
    use std::path::Path;

    // cargo test -- --help
    // cargo test -- --show-output
    // cargo test -- --show-output multiple_values

    #[test]
    /// `cargo test -- --show-output deserialize_xml_cte`
    fn deserialize_xml_cte() -> XmlParserResult<()> {
        let mut cte_chaves = Vec::new();

        let xmls = [
            "35220998765432101234567894741048320396789012_CTe.xml",
            "41220878899001122334455667788990011223344555_CTe.xml",
        ];

        for (index, &xml) in xmls.iter().enumerate() {
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

            let info_cte = cte_proc.get_info();
            println!("InfoCte: {info_cte:#?}\n");

            if index == 0 {
                assert_eq!(nfes.len(), 1);
                assert!(info_cte.emitente_fantasia.is_none());
            }

            if index == 1 {
                assert_eq!(nfes.len(), 4);
                assert_eq!(
                    info_cte.emitente_fantasia,
                    Some("TRANSPORTADORA ALFA".to_string())
                );
            }

            cte_chaves.push(cte);
        }

        let result = [
            Some("12345678901234567890123456789012345678901234".to_string()),
            Some("12345678901234567890123456789012345678905555".to_string()),
        ];

        assert_eq!(cte_chaves, result);

        Ok(())
    }

    #[test]
    /// `cargo test -- --show-output documentos_anteriores_eletronicos`
    fn documentos_anteriores_eletronicos() -> XmlParserResult<()> {
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

#[cfg(test)]
mod tests_struct_order_independence {
    use quick_xml::de::from_reader;
    use serde::{Deserialize, Serialize};
    use std::io::{BufReader, Cursor};

    /// XML fictício baseado na estrutura do documento real (CT-e Complementar),
    /// porém com dados alterados de forma anônima para preservar a privacidade.
    ///
    /// Este XML contém:
    /// * Tags obrigatórias (`ide`, `vPrest`, `imp`) e opcionais (`emit`, `rem`, `receb`, `infCteComp`) em ordem embaralhada.
    /// * Elemento opcional vazio (`receb` auto-fechado como `<receb/>`).
    /// * Elemento opcional ausente (`dest` omitido).
    /// * Sibling elements duplicados (`infCteComp` repetido para testar coleções).
    const XML_TESTE_DIVERSO: &str = r#"<?xml version="1.0" encoding="utf-8"?>
    <infCte Id="CTe31250499999999999999570010000009991322100999" versao="4.00">
        <!-- 1. Campo obrigatório vPrest posicionado no início -->
        <vPrest>
            <vTPrest>850.50</vTPrest>
            <vRec>850.50</vRec>
            <Comp>
                <xNome>FRETE PESO</xNome>
                <vComp>850.50</vComp>
            </Comp>
        </vPrest>

        <!-- 2. Campo obrigatório imp -->
        <imp>
            <ICMS>
                <ICMS00>
                    <CST>00</CST>
                    <vBC>850.50</vBC>
                    <pICMS>12.00</pICMS>
                    <vICMS>102.06</vICMS>
                </ICMS00>
            </ICMS>
        </imp>

        <!-- 3. Campo opcional remetente (rem) -->
        <rem>
            <CNPJ>02999999000188</CNPJ>
            <IE>70218844200</IE>
            <xNome>OMEGA INDUSTRIAL DO BRASIL S/A</xNome>
            <enderReme>
                <xLgr>RODOVIA BR 365</xLgr>
                <nro>100</nro>
                <xBairro>DISTRITO INDUSTRIAL</xBairro>
                <cMun>3170206</cMun>
                <xMun>VILA VELHA</xMun>
                <CEP>29100000</CEP>
                <UF>ES</UF>
            </enderReme>
        </rem>

        <!-- 4. Campo obrigatório ide -->
        <ide>
            <cUF>31</cUF>
            <cCT>13221009</cCT>
            <CFOP>6352</CFOP>
            <natOp>PRESTACAO DE SERVICO DE TRANSPORTE</natOp>
            <mod>57</mod>
            <serie>1</serie>
            <nCT>999</nCT>
            <dhEmi>2025-04-01T09:45:00-03:00</dhEmi>
            <tpImp>1</tpImp>
            <tpEmis>1</tpEmis>
            <cDV>2</cDV>
            <tpAmb>1</tpAmb>
            <tpCTe>1</tpCTe>
            <procEmi>0</procEmi>
            <verProc>Sistema TMS</verProc>
            <cMunEnv>3145000</cMunEnv>
            <xMunEnv>PONTE NOVA</xMunEnv>
            <UFEnv>MG</UFEnv>
            <modal>01</modal>
            <tpServ>0</tpServ>
            <cMunIni>3170206</cMunIni>
            <xMunIni>UBERLANDIA</xMunIni>
            <UFIni>MG</UFIni>
            <cMunFim>3537008</cMunFim>
            <xMunFim>CAMPINAS</xMunFim>
            <UFFim>SP</UFFim>
            <retira>1</retira>
            <indIEToma>1</indIEToma>
            <toma3>
                <toma>3</toma>
            </toma3>
        </ide>

        <!-- 5. Repetição de elementos (sibling elements) infCteComp -->
        <infCteComp>
            <chCTe>31250399999999999999570010000008841791727999</chCTe>
        </infCteComp>
        <infCteComp>
            <chCTe>31250399999999999999570010000008851425084999</chCTe>
        </infCteComp>

        <!-- 6. Campo opcional emitente (emit) -->
        <emit>
            <CNPJ>04999999000199</CNPJ>
            <IE>493559300</IE>
            <xNome>ALFA TRANSPORTES DE CARGAS LTDA</xNome>
            <xFant>ALFA TRANSPORTES</xFant>
            <enderEmit>
                <xLgr>AVENIDA CENTRAL</xLgr>
                <nro>480</nro>
                <xBairro>MEDALHA MILAGROSA</xBairro>
                <cMun>3145000</cMun>
                <xMun>PONTE NOVA</xMun>
                <CEP>38160000</CEP>
                <UF>MG</UF>
            </enderEmit>
            <CRT>3</CRT>
        </emit>

        <!-- 7. Campo opcional recebedor (receb) vazio -->
        <receb/>

        <!-- Opcional destinatário (dest) omitido intencionalmente -->
    </infCte>"#;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    struct IdeMock {
        #[serde(rename = "cUF", default)]
        c_uf: Option<String>,
        #[serde(rename = "cCT", default)]
        c_ct: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    struct EmitMock {
        #[serde(rename = "CNPJ", default)]
        cnpj: Option<String>,
        #[serde(rename = "xNome", default)]
        x_nome: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    struct RemMock {
        #[serde(rename = "CNPJ", default)]
        cnpj: Option<String>,
        #[serde(rename = "xNome", default)]
        x_nome: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    struct RecebMock {
        #[serde(rename = "CNPJ", default)]
        cnpj: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    struct VPrestMock {
        #[serde(rename = "vTPrest")]
        v_tprest: f64,
        #[serde(rename = "vRec")]
        v_rec: f64,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    struct InfCteCompMock {
        #[serde(rename = "chCTe", default)]
        ch_cte: Option<String>,
    }

    /// Struct CteInfoA: Atributos no topo, opcionais no meio, obrigatórios na base.
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct CteInfoA {
        #[serde(rename = "@Id", default)]
        id: Option<String>,
        #[serde(rename = "@versao", default)]
        versao: Option<String>,
        #[serde(rename = "emit", default)]
        emitente: Option<EmitMock>,
        #[serde(rename = "rem", default)]
        remetente: Option<RemMock>,
        #[serde(rename = "receb", default)]
        recebedor: Option<RecebMock>,
        #[serde(rename = "dest", default)]
        destinatario: Option<String>, // Campo ausente
        #[serde(rename = "infCteComp", default)]
        inf_cte_comp: Option<Vec<InfCteCompMock>>,
        #[serde(rename = "ide")]
        ide: IdeMock,
        #[serde(rename = "vPrest")]
        v_prest: VPrestMock,
    }

    /// Struct CteInfoB: Ordem dos campos completamente invertida em relação a Struct A.
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct CteInfoB {
        #[serde(rename = "vPrest")]
        v_prest: VPrestMock,
        #[serde(rename = "ide")]
        ide: IdeMock,
        #[serde(rename = "infCteComp", default)]
        inf_cte_comp: Option<Vec<InfCteCompMock>>,
        #[serde(rename = "dest", default)]
        destinatario: Option<String>, // Campo ausente
        #[serde(rename = "receb", default)]
        recebedor: Option<RecebMock>,
        #[serde(rename = "rem", default)]
        remetente: Option<RemMock>,
        #[serde(rename = "emit", default)]
        emitente: Option<EmitMock>,
        #[serde(rename = "@versao", default)]
        versao: Option<String>,
        #[serde(rename = "@Id", default)]
        id: Option<String>,
    }

    #[test]
    fn test_independencia_de_ordem_com_dados_reais_anonimizados() {
        // 1. Desserializa o XML na CteInfoA
        let cursor_a = Cursor::new(XML_TESTE_DIVERSO);
        let mut reader_a = BufReader::new(cursor_a);
        let info_a: CteInfoA = from_reader(&mut reader_a).expect("Falha ao desserializar CteInfoA");

        // 2. Desserializa o mesmo XML na CteInfoB (que possui ordem de campos invertida)
        let cursor_b = Cursor::new(XML_TESTE_DIVERSO);
        let mut reader_b = BufReader::new(cursor_b);
        let info_b: CteInfoB = from_reader(&mut reader_b).expect("Falha ao desserializar CteInfoB");

        // 3. Valida se os dados extraídos de forma independente são idênticos entre as duas Structs
        assert_eq!(info_a.id, info_b.id);
        assert_eq!(info_a.versao, info_b.versao);
        assert_eq!(info_a.emitente, info_b.emitente);
        assert_eq!(info_a.remetente, info_b.remetente);
        assert_eq!(info_a.recebedor, info_b.recebedor);
        assert_eq!(info_a.destinatario, info_b.destinatario);
        assert_eq!(info_a.inf_cte_comp, info_b.inf_cte_comp);
        assert_eq!(info_a.ide, info_b.ide);
        assert_eq!(info_a.v_prest, info_b.v_prest);

        // 4. Valida se as regras de parsing e fallback default foram obedecidas
        assert_eq!(
            info_a.id,
            Some("CTe31250499999999999999570010000009991322100999".to_string())
        );
        assert_eq!(info_a.versao, Some("4.00".to_string()));

        // Emitente (emit)
        let emit = info_a.emitente.unwrap();
        assert_eq!(emit.cnpj, Some("04999999000199".to_string()));
        assert_eq!(
            emit.x_nome,
            Some("ALFA TRANSPORTES DE CARGAS LTDA".to_string())
        );

        // Remetente (rem)
        let rem = info_a.remetente.unwrap();
        assert_eq!(rem.cnpj, Some("02999999000188".to_string()));
        assert_eq!(
            rem.x_nome,
            Some("OMEGA INDUSTRIAL DO BRASIL S/A".to_string())
        );

        // Recebedor (receb) vazio -> resulta em Some com campos internos None
        assert!(info_a.recebedor.is_some());
        assert!(info_a.recebedor.unwrap().cnpj.is_none());

        // Destinatário (dest) ausente no XML -> resulta em None graças ao default
        assert!(info_a.destinatario.is_none());

        // Lista de CTes Complementados (infCteComp) carregada com sucesso
        let comps = info_a.inf_cte_comp.unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(
            comps[0].ch_cte,
            Some("31250399999999999999570010000008841791727999".to_string())
        );
        assert_eq!(
            comps[1].ch_cte,
            Some("31250399999999999999570010000008851425084999".to_string())
        );

        // Campos obrigatórios carregados com sucesso independente da posição física na stream
        assert_eq!(info_a.v_prest.v_tprest, 850.50);
        assert_eq!(info_a.ide.c_ct, Some("13221009".to_string()));
    }
}
