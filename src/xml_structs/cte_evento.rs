//! # Eventos de Conhecimento de Transporte Eletrônico (CT-e)
//!
//! Este módulo gerencia a desserialização e estruturação dos dados de eventos do CT-e,
//! como o cancelamento, carta de correção (CC-e), registro de passagem, subcontratação,
//! substituição e complementação de valores.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    Arguments, GetKey, InfoExtension, Information, OptExt, StructExtension,
    get_naive_date_from_yyyy_mm_dd,
    xml_structs::{agente::Agente, assinaturas::Signature, ret_evento::RetEvento},
};

/// Representação intermediária e consolidada de um Evento de CT-e.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoCteEvento {
    /// Chave de acesso do CT-e vinculado ao evento.
    pub cte: Option<String>,
    /// Data e hora de emissão do evento.
    pub dh_emi: Option<NaiveDate>,
    /// Indicador se o evento correspondente é de cancelamento homologado.
    pub cancelado: bool,
    /// Chave de acesso do CT-e complementar se aplicável.
    pub cte_complementar: Option<String>,
    /// Chave de acesso do CT-e de redespacho se aplicável.
    pub cte_redespacho: Option<String>,
    /// Chave de acesso do CT-e de redespacho interno se aplicável.
    pub cte_redespacho_interno: Option<String>,
    /// Chave de acesso do CT-e de subcontratação se aplicável.
    pub cte_subcontratacao: Option<String>,
    /// Chave de acesso do CT-e substituído se aplicável.
    pub cte_substituido: Option<String>,
    /// Chave de acesso do CT-e vinculado multimodal se aplicável.
    pub cte_vinculado: Option<String>,
}

impl InfoCteEvento {
    /// Agrupa e retorna todas as chaves de CT-es complementares e relacionados
    /// contidas neste evento, eliminando duplicatas de forma eficiente.
    pub fn get_ctes(&self) -> Vec<String> {
        [
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
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
    }
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoCteEvento {}

/// Ver src/group_by_hashmap.rs
impl GetKey for InfoCteEvento {
    /// Implementação auxiliar para indexação do evento de CT-e.
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

/// Processo completo de registro de Evento de CT-e (`<procEventoCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcEventoCte {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do layout da distribuição de eventos.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Conteúdo textual bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Estrutura contendo o evento de CT-e de fato.
    #[serde(rename = "eventoCTe")]
    pub evento_cte: Evento,

    /// Retorno de registro emitido pelo webservice da SEFAZ.
    #[serde(rename = "retEventoCTe")]
    pub ret_evento_cte: RetEvento,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for ProcEventoCte {
    /// Transforma os dados deste protocolo de evento em uma estrutura consolidada `Information`.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("evento cte xml_path: {xml_path:?}");
            println!("proc_evento_cte: {self:#?}\n");
        }
        Information::EventoCte(Box::new(self.get_info()))
    }
}

impl ProcEventoCte {
    /// Extrai a chave de acesso do CT-e vinculado ao evento.
    pub fn get_cte(&self) -> Option<String> {
        self.evento_cte.inf_evento.ch_cte.get_key()
    }

    /// Determina se o evento trata-se de um cancelamento homologado.
    pub fn informacao_de_cancelamento(&self) -> bool {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_canc_cte // ev_canc_cte: Option<EvCancCte>, // Evento de Cancelamento
            .is_some()
    }

    /// Retorna a data de emissão do evento de forma estruturada.
    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.evento_cte.inf_evento.get_dh_evento()
    }

    /// Extrai a chave de acesso do CT-e complementar, se houver.
    pub fn get_cte_complementar(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_complementar
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_compl())
    }

    /// Extrai a chave de acesso do CT-e de redespacho, se houver.
    pub fn get_cte_redespacho(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_redespacho
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_redesp())
    }

    /// Extrai a chave de acesso do CT-e de redespacho interno, se houver.
    pub fn get_cte_redespacho_interno(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_redespacho_inter
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_redesp_inter())
    }

    /// Extrai a chave de acesso do CT-e de subcontratação, se houver.
    pub fn get_cte_subcontratacao(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_subcontratacao
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_subcon())
    }

    /// Extrai a chave de acesso do CT-e substituído, se houver.
    pub fn get_cte_substituido(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_substituido
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_substituicao())
    }

    /// Extrai a chave de acesso do CT-e de transporte multimodal vinculado, se houver.
    pub fn get_cte_vinculado(&self) -> Option<String> {
        self.evento_cte
            .inf_evento
            .det_evento
            .ev_cte_multimodal
            .as_ref()
            .and_then(|evento| evento.get_ch_cte_vinculado())
    }

    /// Consolida todos os metadados do processamento do evento de CT-e na estrutura `InfoCteEvento`.
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

/// Estrutura correspondente ao nó `<eventoCTe>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Evento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão declarada no leiaute do evento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Bloco contendo as informações de identificação do evento.
    #[serde(rename = "infEvento")]
    pub inf_evento: InfEvento,

    /// Bloco contendo a assinatura digital do evento.
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

/// Detalhes de Identificação do Evento (`<infEvento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfEvento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Identificador do nó do evento.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    /// Namespace XML.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Código do órgão de recepção do evento.
    #[serde(rename = "cOrgao", default)]
    pub c_orgao: Option<String>,

    /// Identificação do Ambiente: 1 - Produção; 2 - Homologação.
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// CNPJ do autor do evento.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Chave de acesso do CT-e vinculado ao evento.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Data e hora do evento.
    #[serde(rename = "dhEvento", default)]
    pub dh_evento: Option<String>,

    /// Tipo correspondente ao evento.
    #[serde(rename = "tpEvento", default)]
    pub tp_evento: Option<String>,

    /// Número sequencial do evento para a mesma chave.
    #[serde(rename = "nSeqEvento", default)]
    pub n_seq_evento: Option<String>,

    /// Versão do evento declarada.
    #[serde(rename = "verEvento", default)]
    pub ver_evento: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Bloco de dados de negócio do evento.
    #[serde(rename = "detEvento")]
    pub det_evento: DetEvento,
}

impl InfEvento {
    /// Extrai de forma estruturada a data em que ocorreu o evento.
    pub fn get_dh_evento(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_evento)
    }
}

/// Bloco contendo os detalhes específicos de negócio do evento de CT-e (`<detEvento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct DetEvento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do detalhamento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Versão correspondente à especificação do evento de negócio.
    #[serde(rename = "@versaoEvento", default)]
    pub versao_evento: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Detalhes de Carta de Correção Eletrônica (CC-e).
    #[serde(rename = "evCCeCTe", default)]
    pub ev_cce_cte: Option<EvCceCte>,

    /// Evento de autorização de uso do CT-e gerado pelo MDF-e.
    #[serde(rename = "evCTeAutorizadoMDFe", default)]
    pub ev_cte_autorizado_mdfe: Option<EvCteAutorizadoMdfe>,

    /// Evento correspondente ao Cancelamento do CT-e.
    #[serde(rename = "evCancCTe", default)]
    pub ev_canc_cte: Option<EvCancCte>,

    /// Evento correspondente a CT-e de Complementação de Valores.
    #[serde(rename = "evCTeComplementar", default)]
    pub ev_cte_complementar: Option<EvCteComplementar>,

    /// Evento correspondente a CT-e de Redespacho.
    #[serde(rename = "evCTeRedespacho", default)]
    pub ev_cte_redespacho: Option<EvCteRedespacho>,

    /// Evento correspondente a CT-e de Redespacho Intermediário.
    #[serde(rename = "evCTeRedespachoInter", default)]
    pub ev_cte_redespacho_inter: Option<EvCteRedespachoInter>,

    /// Evento de registro de passagem automática das mercadorias.
    #[serde(rename = "evCTeRegPassagemAuto", default)]
    pub ev_cte_reg_passagem_auto: Option<EvCteRegPassagemAuto>,

    /// Evento correspondente a CT-e de Subcontratação.
    #[serde(rename = "evCTeSubcontratacao", default)]
    pub ev_cte_subcontratacao: Option<EvCteSubcontratacao>,

    /// Evento correspondente a CT-e substituído.
    #[serde(rename = "evCTeSubstituido", default)]
    pub ev_cte_substituido: Option<EvCteSubstituido>,

    /// Evento de vinculação ao CT-e Multimodal se aplicável.
    #[serde(rename = "evCTeMultimodal", default)]
    pub ev_cte_multimodal: Option<EvCteMultimodal>,

    /// Evento de Prestação de Serviço em Desacordo (`evPrestDesacordo`).
    #[serde(rename = "evPrestDesacordo", default)]
    pub ev_prest_desacordo: Option<EvPrestDesacordo>,

    /// Evento de Insucesso na Entrega do CT-e (`evInsucessoEntregaCTe`).
    #[serde(rename = "evInsucessoEntregaCTe", default)]
    pub ev_insucesso_entrega_cte: Option<EvInsucessoEntregaCte>,

    /// Evento de Cancelamento do Insucesso na Entrega (`evCancInsucessoEntregaCTe`).
    #[serde(rename = "evCancInsucessoEntregaCTe", default)]
    pub ev_canc_insucesso_entrega_cte: Option<EvCancInsucessoEntregaCte>,

    /// Evento de Comprovante de Entrega do CT-e (`evIECTe`).
    #[serde(rename = "evIECTe", default)]
    pub ev_ie_cte: Option<EvIeCte>,

    /// Evento de Cancelamento do Comprovante de Entrega (`evCancCECTe`).
    #[serde(rename = "evCancCECTe", default)]
    pub ev_canc_ce_cte: Option<EvCancCeCte>,

    /// Evento de cancelamento de MDF-e vinculado ao CT-e (`evCTeCanceladoMDFe`).
    #[serde(rename = "evCTeCanceladoMDFe", default)]
    pub ev_cte_cancelado_mdfe: Option<EvCteCanceladoMdfe>,

    /// Evento de Comprovante de Entrega do CT-e (`evCECTe`).
    #[serde(rename = "evCECTe", default)]
    pub ev_ce_cte: Option<EvCeCte>,

    /// Cancelamento de Insucesso na Entrega (`evCancIECTe`).
    #[serde(rename = "evCancIECTe", default)]
    pub ev_canc_ie_cte: Option<EvCancInsucessoEntregaCte>,

    /// GTV-e Autorizado no CT-e OS (`evGTVeAutorizadoCTeOS`).
    #[serde(rename = "evGTVeAutorizadoCTeOS", default)]
    pub ev_gtve_autorizado_cteos: Option<EvGtveAutorizadoCteOs>,

    /// Evento Prévio de Emissão em Contingência (`evEPECCTe`).
    #[serde(rename = "evEPECCTe", default)]
    pub ev_epec_cte: Option<EvEpecCte>,

    /// Cancelamento de CT-e Complementar (`evCancCTeComplementar`).
    #[serde(rename = "evCancCTeComplementar", default)]
    pub ev_canc_cte_complementar: Option<EvCancCteComplementar>,

    /// Cancelamento de Prestação de Serviço em Desacordo (`evCancPrestDesacordo`).
    #[serde(rename = "evCancPrestDesacordo", default)]
    pub ev_canc_prest_desacordo: Option<EvCancPrestDesacordo>,

    /// Evento de registro de passagem em posto fiscal (`evCTeRegPassagem`).
    #[serde(rename = "evCTeRegPassagem", default)]
    pub ev_cte_reg_passagem: Option<EvCteRegPassagem>,
    /// Evento de registro de passagem automático via MDF-e (`evCTeRegPassagemAutoMDFe`).
    #[serde(rename = "evCTeRegPassagemAutoMDFe", default)]
    pub ev_cte_reg_passagem_auto_mdfe: Option<EvCteRegPassagemAuto>,
}

/// Evento de registro de passagem em posto fiscal (`<evCTeRegPassagem>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRegPassagem {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "cUnidFiscal", default)]
    pub c_unid_fiscal: Option<String>,
    #[serde(rename = "xUnidFiscal", default)]
    pub x_unid_fiscal: Option<String>,
    #[serde(rename = "CPFFunc", default)]
    pub cpf_func: Option<String>,
    #[serde(rename = "xFunc", default)]
    pub x_func: Option<String>,
    #[serde(rename = "tpTransm", default)]
    pub tp_transm: Option<String>,
    #[serde(rename = "infPass", default)]
    pub inf_pass: Option<InfPass>,
    #[serde(rename = "chMDFe", default)]
    pub ch_mdfe: Option<String>,
}

/// Evento GTV-e Autorizado no CT-e OS (`<evGTVeAutorizadoCTeOS>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvGtveAutorizadoCteOs {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeOS", default)]
    pub ch_cte_os: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "dhEmi", default)]
    pub dh_emi: Option<String>,
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,
}

/// Evento Prévio de Emissão em Contingência (`<evEPECCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvEpecCte {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "xJust", default)]
    pub x_just: Option<String>,
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    #[serde(rename = "vTPrest", default)]
    pub v_tprest: Option<String>,
    #[serde(rename = "vCarga", default)]
    pub v_carga: Option<String>,
    #[serde(rename = "toma4", default)]
    pub toma4: Option<Agente>,
    #[serde(rename = "modal", default)]
    pub modal: Option<String>,
    #[serde(rename = "UFIni", default)]
    pub uf_ini: Option<String>,
    #[serde(rename = "UFFim", default)]
    pub uf_fim: Option<String>,
    #[serde(rename = "tpCTe", default)]
    pub tp_cte: Option<String>,
    #[serde(rename = "dhEmi", default)]
    pub dh_emi: Option<String>,
}

/// Cancelamento de CT-e Complementar (`<evCancCTeComplementar>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCancCteComplementar {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "chCTeCompl", default)]
    pub ch_cte_compl: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,
}

/// Cancelamento de Prestação de Serviço em Desacordo (`<evCancPrestDesacordo>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCancPrestDesacordo {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProtEvPrestDes", default)]
    pub n_prot_ev_prest_des: Option<String>,
    #[serde(rename = "indDesacordoOper", default)]
    pub ind_desacordo_oper: Option<String>,
    #[serde(rename = "xObs", default)]
    pub x_obs: Option<String>,
}
impl Mdfe {
    /// Retorna a chave de acesso do MDF-e vinculada, se presente.
    pub fn get_ch_mdfe(&self) -> Option<String> {
        self.ch_mdfe.get_key()
    }
}

impl EvCeCte {
    /// Retorna as chaves de acesso das NF-es vinculadas ao comprovante de entrega.
    pub fn get_chaves_nfes(&self) -> Vec<String> {
        self.inf_entrega
            .iter()
            .flatten()
            .filter_map(|inf| inf.ch_nfe.get_key())
            .collect()
    }
}

impl EvEpecCte {
    /// Retorna o valor total da prestação informado no EPEC, se válido.
    pub fn get_valor_total(&self) -> Option<f64> {
        self.v_tprest.to_float64()
    }
}

/// Evento de cancelamento de MDF-e emitido para CT-e autorizado (`<evCTeCanceladoMDFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteCanceladoMdfe {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "MDFe", default)]
    pub mdfe: Option<Mdfe>,
    #[serde(rename = "emit", default)]
    pub emit: Option<Agente>,
}

/// Evento oficial de Comprovante de Entrega do CT-e (`<evCECTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCeCte {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "dhEntrega", default)]
    pub dh_entrega: Option<String>,
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
    #[serde(rename = "hashEntrega", default)]
    pub hash_entrega: Option<String>,
    #[serde(rename = "dhHashEntrega", default)]
    pub dh_hash_entrega: Option<String>,
    #[serde(rename = "infEntrega", default)]
    pub inf_entrega: Option<Vec<InfEntregaCeCte>>,
}

/// Documentos vinculados ao comprovante de entrega (`<infEntrega>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfEntregaCeCte {
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,
}

/// Evento de Prestação de Serviço em Desacordo (`<evPrestDesacordo>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvPrestDesacordo {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "indDesacordoOper", default)]
    pub ind_desacordo_oper: Option<String>,
    #[serde(rename = "xObs", default)]
    pub x_obs: Option<String>,
}

/// Evento de Insucesso na Entrega (`<evInsucessoEntregaCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvInsucessoEntregaCte {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "dhTentativaEntrega", default)]
    pub dh_tentativa_entrega: Option<String>,
    #[serde(rename = "nTentativa", default)]
    pub n_tentativa: Option<String>,
    #[serde(rename = "tpMotivo", default)]
    pub tp_motivo: Option<String>,
    #[serde(rename = "xJustMotivo", default)]
    pub x_just_motivo: Option<String>,
    #[serde(rename = "hashTentativaEntrega", default)]
    pub hash_tentativa_entrega: Option<String>,
    #[serde(rename = "dhHashTentativaEntrega", default)]
    pub dh_hash_tentativa_entrega: Option<String>,
}

/// Cancelamento de Insucesso na Entrega (`<evCancInsucessoEntregaCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCancInsucessoEntregaCte {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "nProtIE", default)]
    pub n_prot_ie: Option<String>,
}

/// Comprovante de Entrega do CT-e (`<evIECTe>` / `<evCECTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvIeCte {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "dhEntrega", default)]
    pub dh_entrega: Option<String>,
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
}

/// Cancelamento de Comprovante de Entrega (`<evCancCECTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCancCeCte {
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "nProtCE", default)]
    pub n_prot_ce: Option<String>,
}

/// Dados relativos à Carta de Correção Eletrônica do CT-e (`<evCCeCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCceCte {
    /// Descrição descritiva do tipo de evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Condição geral de uso e responsabilidades do emissor relativas à CC-e.
    #[serde(rename = "xCondUso", default)]
    pub x_cond_uso: Option<String>,

    /// Lista de correções de campos estruturadas do CT-e.
    #[serde(rename = "infCorrecao", default)]
    pub inf_correcao: Vec<InfCorrecao>,
}

/// Dados estruturados para detalhamento de uma retificação de campo (`<infCorrecao>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfCorrecao {
    /// Nome do nó pai do grupo XML alterado.
    #[serde(rename = "grupoAlterado", default)]
    pub grupo_alterado: Option<String>,

    /// Nome exato da tag correspondente ao campo retificado.
    #[serde(rename = "campoAlterado", default)]
    pub campo_alterado: Option<String>,

    /// Novo valor correspondente atribuído ao campo retificado.
    #[serde(rename = "valorAlterado", default)]
    pub valor_alterado: Option<String>,

    /// Número sequencial do item retificado (`nroItemAlterado`).
    #[serde(rename = "nroItemAlterado", default)]
    pub nro_item_alterado: Option<String>,
}

/// Evento correspondente à autorização emitida pelo webservice MDF-e (`<evCTeAutorizadoMDFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteAutorizadoMdfe {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Dados consolidados do MDF-e gerador.
    #[serde(rename = "MDFe")]
    pub mdfe: Mdfe,

    /// Dados do Emitente do MDF-e se aplicável.
    #[serde(rename = "emit", default)]
    pub emit: Option<Agente>,
}

/// Dados de identificação do Manifesto de Documentos Fiscais Eletrônicos (`<MDFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Mdfe {
    /// Chave de acesso correspondente ao MDF-e.
    #[serde(rename = "chMDFe", default)]
    pub ch_mdfe: Option<String>,

    /// Modal correspondente de transporte.
    #[serde(rename = "modal", default)]
    pub modal: Option<String>,

    /// Data de emissão do MDF-e.
    #[serde(rename = "dhEmi", default)]
    pub dh_emi: Option<String>,

    /// Número do protocolo de recebimento emitido pela SEFAZ.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Número do protocolo de cancelamento do MDF-e (`nProtCanc`).
    #[serde(rename = "nProtCanc", default)]
    pub n_prot_canc: Option<String>,

    /// Data e hora de recepção final da MDF-e pela SEFAZ.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,
}

/// Evento correspondente a registro de Redespacho vinculado (`<evCTeRedespacho>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRedespacho {
    /// Descrição descritiva do tipo de evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Chave de acesso do CT-e correspondente de Redespacho.
    #[serde(rename = "chCTeRedesp", default)]
    pub ch_cte_redesp: Option<String>,

    /// Data de recebimento de status do Redespacho.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número de protocolo SEFAZ relacionado.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}

impl EvCteRedespacho {
    /// Retorna a chave de acesso do CT-e de redespacho de forma limpa.
    pub fn get_ch_cte_redesp(&self) -> Option<String> {
        self.ch_cte_redesp.get_key()
    }
}

/// Evento correspondente a registro de Redespacho Intermediário (`<evCTeRedespachoInter>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRedespachoInter {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Chave de acesso do CT-e correspondente de Redespacho Intermediário.
    #[serde(rename = "chCTeRedespInter", default)]
    pub ch_cte_redesp_inter: Option<String>,

    /// Data de recebimento de status.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número de protocolo SEFAZ relacionado.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}

impl EvCteRedespachoInter {
    /// Retorna a chave de acesso do CT-e de redespacho intermediário de forma limpa.
    pub fn get_ch_cte_redesp_inter(&self) -> Option<String> {
        self.ch_cte_redesp_inter.get_key()
    }
}

/// Evento correspondente ao registro automático de passagem física de fronteiras (`<evCTeRegPassagemAuto>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteRegPassagemAuto {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Tipo de transmissão associada.
    #[serde(rename = "tpTransm", default)]
    pub tp_transm: Option<String>,

    /// Detalhes de passagem e geolocalização do veículo coletados de sensores.
    #[serde(rename = "infPass", default)]
    pub inf_pass: Option<InfPass>,

    /// Chave de acesso do MDF-e vinculado se aplicável.
    #[serde(rename = "chMDFe", default)]
    pub ch_mdfe: Option<String>,
}

/// Bloco de informações de geolocalização física de passagem de fronteira (`<infPass>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfPass {
    /// Código da Unidade Federativa correspondente à divisa de trânsito físico.
    #[serde(rename = "cUFTransito", default)]
    pub c_uftransito: Option<String>,

    /// Código identificador do equipamento leitor (RFID, OCR, etc.).
    #[serde(rename = "cIdEquip", default)]
    pub c_id_equip: Option<String>,

    /// Descrição literal do local do equipamento leitor.
    #[serde(rename = "xIdEquip", default)]
    pub x_id_equip: Option<String>,

    /// Tipo do equipamento leitor associado.
    #[serde(rename = "tpEquip", default)]
    pub tp_equip: Option<String>,

    /// Placa do veículo identificada.
    #[serde(rename = "placa", default)]
    pub placa: Option<String>,

    /// Sentido físico do tráfego do veículo.
    #[serde(rename = "tpSentido", default)]
    pub tp_sentido: Option<String>,

    /// Data e hora correspondentes em que ocorreu a passagem física.
    #[serde(rename = "dhPass", default)]
    pub dh_pass: Option<String>,

    /// Latitude geográfica registrada do leitor.
    #[serde(rename = "latitude", default)]
    pub latitude: Option<String>,

    /// Longitude geográfica registrada do leitor.
    #[serde(rename = "longitude", default)]
    pub longitude: Option<String>,

    /// Número sequencial único (NSU) atribuído ao processamento da passagem.
    #[serde(rename = "NSU", default)]
    pub nsu: Option<String>,
}

/// Detalhes específicos de registro de Cancelamento do CT-e (`<evCancCTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCancCte {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Número do protocolo da autorização original emitido pela SEFAZ.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Texto contendo a justificativa do cancelamento (mínimo de 15 caracteres).
    #[serde(rename = "xJust", default)]
    pub x_just: Option<String>,
}

/// Dados correspondentes a eventos de CT-e de Complementação de Valores (`<evCTeComplementar>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteComplementar {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Chave de acesso do CT-e complementar associado.
    #[serde(rename = "chCTeCompl", default)]
    pub ch_cte_compl: Option<String>,

    /// Data de processamento do recebimento de status do evento complementar.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número de protocolo SEFAZ correspondente.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}

impl EvCteComplementar {
    /// Retorna a chave de acesso do CT-e complementar de forma limpa.
    pub fn get_ch_cte_compl(&self) -> Option<String> {
        self.ch_cte_compl.get_key()
    }
}

/// Dados correspondentes a eventos de Subcontratação de frete (`<evCTeSubcontratacao>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteSubcontratacao {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Chave de acesso do CT-e de subcontratação correspondente.
    #[serde(rename = "chCTeSubcon", default)]
    pub ch_cte_subcon: Option<String>,

    /// Data de processamento do recebimento de status.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número de protocolo SEFAZ correspondente.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}

impl EvCteSubcontratacao {
    /// Retorna a chave de acesso do CT-e de subcontratação de forma limpa.
    pub fn get_ch_cte_subcon(&self) -> Option<String> {
        self.ch_cte_subcon.get_key()
    }
}

/// Dados correspondentes a eventos de Substituição de valores (`<evCTeSubstituido>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteSubstituido {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Chave de acesso correspondente ao CT-e que foi substituído na transação.
    #[serde(rename = "chCTeSubstituicao", default)]
    pub ch_cte_substituicao: Option<String>,

    /// Data de processamento do recebimento de status.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número de protocolo SEFAZ correspondente.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}

impl EvCteSubstituido {
    /// Retorna a chave de acesso do CT-e substituído de forma limpa.
    pub fn get_ch_cte_substituicao(&self) -> Option<String> {
        self.ch_cte_substituicao.get_key()
    }
}

/// Dados relativos a vinculações ao modal de Transporte Multimodal (`<evCTeMultimodal>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvCteMultimodal {
    /// Descrição literal correspondente ao evento.
    #[serde(rename = "descEvento", default)]
    pub desc_evento: Option<String>,

    /// Chave de acesso correspondente ao CT-e de vinculação multimodal.
    #[serde(rename = "chCTeVinculado", default)]
    pub ch_cte_vinculado: Option<String>,

    /// Data de processamento do recebimento de status.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Número de protocolo SEFAZ correspondente.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}

impl EvCteMultimodal {
    /// Retorna a chave de acesso do CT-e vinculado multimodal de forma limpa.
    pub fn get_ch_cte_vinculado(&self) -> Option<String> {
        self.ch_cte_vinculado.get_key()
    }
}
