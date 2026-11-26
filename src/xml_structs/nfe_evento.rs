//! # Processamento de XML de Eventos de NF-e
//!
//! Este módulo gerencia o mapeamento, desserialização e consolidação de dados
//! contidos em documentos XML de eventos associados à Nota Fiscal Eletrônica (NF-e),
//! tais como solicitações de cancelamento ou correções.
//!
//! Módulo gerado originalmente a partir de múltiplos esquemas XML (arquivos `.xsd`)
//! utilizando ferramentas de geração de instâncias do Apache XMLBeans (`xsd2inst`)
//! e o utilitário `read_xml` com a opção `-s` para conversão em estruturas Rust de desserialização.
//!
//! Para gerar instâncias XML a partir do XSD de validação do processo de Cancelamento:
//!
//! ```console
//! xsd2inst procEventoCancNFe_v1.00.xsd -name procEventoNFe -dl > procEventoCancNFe_v1.00.xml
//! ```
//!
//! E para converter as instâncias XML em estruturas Rust equivalentes:
//!
//! ```console
//! read-xml -s procEventoCancNFe_v1.00.xml > procEventoCancNFe_v1.00.rs
//! ```
//!
//! Maiores detalhes sobre os esquemas de eventos podem ser obtidos nos portais oficiais da SEFAZ:
//! * <https://dfe-portal.svrs.rs.gov.br/MDFE/ConsultaSchema>
//! * <https://dfe-portal.svrs.rs.gov.br/Nfe>

/*
Generates XML file from Multiple XML schemas (xsd files).

This feature requires Apache XMLBeans.
https://xmlbeans.apache.org/docs/2.0.0/guide/tools.html#xsd2inst
xsd2inst (Schema to Instance Tool)
Prints an XML instance from the specified global element using the specified schema.

XSD to XML:
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst procEventoCancNFe_v1.00.xsd -name procEventoNFe -dl > procEventoCancNFe_v1.00.xml

Converter XML file to Rust struct:
read-xml -s procEventoCancNFe_v1.00.xml > procEventoCancNFe_v1.00.rs

No Manjaro Linux, install qxmledit:
yay -S qxmledit
Make PDF files from XSD.
*/

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    Arguments, GetKey, InfoExtension, Information, OptExt, REGEX_CANCELAMENTO, StructExtension,
    get_naive_date_from_yyyy_mm_dd,
    xml_structs::{agente::Agente, assinaturas::Signature, ret_evento::RetEvento},
};

/// Representação intermediária e consolidada de um Evento de NF-e.
///
/// Esta estrutura unifica a identificação da NF-e vinculada, a data de emissão
/// do evento e o indicador correspondente a cancelamento.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct InfoNfeEvento {
    /// Chave de acesso correspondente à NF-e vinculada.
    pub nfe: Option<String>,
    /// Data de emissão em que ocorreu o registro do evento.
    pub dh_emi: Option<NaiveDate>,
    /// Indicador booleano que assinala se o evento corresponde a um cancelamento homologado.
    pub cancelado: bool,
    /// Chave de NF-e referenciada pelo evento (`chNFeRefte`).
    #[serde(default)]
    pub nfe_referenciada: Option<String>,
    /// Chave de CT-e vinculado ao evento (`CTe/chCTe` ou `MDFe/chCTe`).
    #[serde(default)]
    pub cte_relacionado: Option<String>,
}

impl InfoExtension for InfoNfeEvento {}

impl GetKey for InfoNfeEvento {
    /// Retorna a chave de acesso da NF-e vinculada a este evento.
    fn get_chave(&self) -> Option<String> {
        self.nfe.clone()
    }
}

/// Representação estrutural direta do XML de validação do processo de Evento da NF-e (`<procEventoNFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcEventoNfe {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do layout do protocolo correspondente.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente ao protocolo do evento.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Bloco contendo os dados do evento da NF-e.
    #[serde(rename = "evento")]
    pub evento: Evento,

    /// Bloco correspondente ao retorno do processamento do evento pela SEFAZ.
    #[serde(rename = "retEvento")]
    pub ret_evento: RetEvento,
}

impl StructExtension for ProcEventoNfe {
    /// Converte os nós desserializados do processo de evento para informações consolidadas.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("evento nfe xml_path: {xml_path:?}");
            println!("proc_evento_nfe: {self:#?}\n");
        }
        Information::EventoNfe(Box::new(self.get_info()))
    }
}

impl ProcEventoNfe {
    /// Obtém a chave de acesso da NF-e vinculada a partir dos dados do evento.
    pub fn get_nfe(&self) -> Option<String> {
        self.evento.inf_evento.ch_nfe.get_key()
    }

    /// Obtém a chave da NF-e referenciada em eventos `NFE_REFERENCIADA` (`chNFeRefte`).
    pub fn get_nfe_referenciada(&self) -> Option<String> {
        self.evento.inf_evento.det_evento.ch_nfe_refte.get_key()
    }

    /// Obtém a justificativa do evento, se informada.
    pub fn get_justificativa(&self) -> Option<String> {
        self.evento.inf_evento.det_evento.x_just.get_not_empty()
    }

    /// Obtém o texto da carta de correção (`xCorrecao`), se informado.
    pub fn get_correcao(&self) -> Option<String> {
        self.evento.inf_evento.det_evento.x_correcao.get_not_empty()
    }

    /// Determina se o evento em questão corresponde a uma homologação de cancelamento.
    pub fn informacao_de_cancelamento(&self) -> bool {
        let autorizado = self
            .ret_evento
            .inf_evento
            .as_ref()
            .is_none_or(crate::xml_structs::ret_evento::RetInfEvento::is_success);
        autorizado && self.evento.informacao_de_cancelamento()
    }

    /// Obtém a data de emissão associada ao registro deste evento.
    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.evento.inf_evento.get_dh_evento()
    }

    /// Retorna a estrutura consolidada `InfoNfeEvento` correspondente a este processo.
    pub fn get_info(&self) -> InfoNfeEvento {
        let mut info = self.evento.get_info();
        info.cancelado = self.informacao_de_cancelamento();
        info
    }
}

/// Dados estruturados correspondentes à tag `<evento>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Evento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do layout do evento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente ao evento.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Bloco de informações detalhadas do evento.
    #[serde(rename = "infEvento")]
    pub inf_evento: InfEvento,

    /// Bloco contendo a assinatura digital da emissão do evento.
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

impl StructExtension for Evento {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("evento nfe xml_path: {xml_path:?}");
            println!("evento_nfe: {self:#?}\n");
        }
        Information::EventoNfe(Box::new(self.get_info()))
    }
}

impl Evento {
    pub fn get_nfe(&self) -> Option<String> {
        self.inf_evento.ch_nfe.get_key()
    }

    /// Identifica cancelamento da NF-e (`tpEvento` 110111 ou 110112), evitando falsos positivos
    /// em eventos como "Cancelamento de Pedido de Prorrogação" ou "Cancelamento de EPEC".
    pub fn informacao_de_cancelamento(&self) -> bool {
        if let Some(tp) = self.inf_evento.tp_evento.as_deref() {
            return matches!(tp.trim(), "110111" | "110112");
        }
        REGEX_CANCELAMENTO.is_match(&self.inf_evento.det_evento.desc_evento)
    }

    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.inf_evento.get_dh_evento()
    }

    pub fn get_info(&self) -> InfoNfeEvento {
        let det = &self.inf_evento.det_evento;
        let cte_relacionado = det
            .cte
            .as_ref()
            .and_then(|c| c.ch_cte.get_key())
            .or_else(|| det.mdfe.as_ref().and_then(|m| m.ch_cte.get_key()));

        InfoNfeEvento {
            nfe: self.get_nfe(),
            dh_emi: self.get_data_emissao(),
            cancelado: self.informacao_de_cancelamento(),
            nfe_referenciada: det.ch_nfe_refte.get_key(),
            cte_relacionado,
        }
    }
}

/// Bloco de Informações do Evento (`<infEvento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfEvento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Identificador único do evento gerado pelo emitente.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Código do órgão de recepção do evento (UF correspondente ou "91" para RFB).
    #[serde(rename = "cOrgao", default)]
    pub c_orgao: Option<String>,

    /// Identificação do Ambiente: 1 - Produção; 2 - Homologação.
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// CPF do autor do evento se aplicável.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// CNPJ do autor do evento se aplicável.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Chave de acesso correspondente à NF-e vinculada.
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,

    /// Data e hora do evento, no formato AAAA-MM-DDTHH:MM:SSTZD.
    #[serde(rename = "dhEvento", default)]
    pub dh_evento: Option<String>,

    /// Código identificador correspondente ao tipo do evento.
    #[serde(rename = "tpEvento", default)]
    pub tp_evento: Option<String>,

    /// Número sequencial correspondente ao evento para a mesma chave de acesso.
    #[serde(rename = "nSeqEvento", default)]
    pub n_seq_evento: Option<String>,

    /// Versão correspondente do leiaute de eventos.
    #[serde(rename = "verEvento", default)]
    pub ver_evento: Option<String>,

    /// Conteúdo textual contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Bloco de detalhes específicos correspondentes ao tipo de evento.
    #[serde(rename = "detEvento")]
    pub det_evento: DetEvento,
}

impl InfEvento {
    /// Retorna a data de emissão correspondente ao registro do evento de forma estruturada.
    pub fn get_dh_evento(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_evento)
    }
}

/// Bloco de Detalhes Específicos do Evento (`<detEvento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct DetEvento {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    /// Versão do layout de detalhes do evento.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou desordem física de tags)
    // =========================================================================
    /// Código do órgão autorizador correspondente se aplicável.
    #[serde(rename = "cOrgaoAutor", default)]
    pub c_orgao_autor: Option<String>,

    /// Informações correspondentes a CT-e vinculado se aplicável.
    #[serde(rename = "CTe", default)]
    pub cte: Option<Cte>,

    /// Dados cadastrais de identificação do Emitente se aplicável.
    #[serde(rename = "emit", default)]
    pub emit: Option<Agente>,

    /// Tipo de autor do evento (p. ex., Emitente, Destinatário, Fisco).
    #[serde(rename = "tpAutor", default)]
    pub tp_autor: Option<String>,

    /// Informações correspondentes a MDF-e vinculado se aplicável.
    #[serde(rename = "MDFe", default)]
    pub mdfe: Option<Mdfe>,

    /// Número do protocolo de autorização de uso vinculado.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Detalhamento textual correspondente a correção registrada se aplicável.
    #[serde(rename = "xCorrecao", default)]
    pub x_correcao: Option<String>,

    /// Termo de condições de uso correspondente a carta de correção se aplicável.
    #[serde(rename = "xCondUso", default)]
    pub x_cond_uso: Option<String>,

    /// Texto correspondente à justificativa legal do evento.
    #[serde(rename = "xJust", default)]
    pub x_just: Option<String>,

    /// Versão correspondente do aplicativo que realizou o processamento.
    #[serde(rename = "verAplic", default)]
    pub ver_aplic: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Descrição textual correspondente ao tipo de evento (p. ex., "Cancelamento").
    #[serde(rename = "descEvento", default)]
    pub desc_evento: String,
    #[serde(rename = "UFDest", default)]
    pub uf_dest: Option<String>,
    #[serde(rename = "UFVeic", default)]
    pub uf_veic: Option<String>,
    #[serde(rename = "indRet", default)]
    pub ind_ret: Option<String>,
    #[serde(rename = "indOffline", default)]
    pub ind_offline: Option<String>,
    #[serde(rename = "emailDest", default)]
    pub email_dest: Option<String>,
    #[serde(rename = "idPedido", default)]
    pub id_pedido: Option<String>,
    #[serde(rename = "idPedidoCancelado", default)]
    pub id_pedido_cancelado: Option<String>,
    #[serde(rename = "respCancPedido", default)]
    pub resp_canc_pedido: Option<RespPedido>,
    #[serde(rename = "statCancPedido", default)]
    pub stat_canc_pedido: Option<String>,
    #[serde(rename = "justStatus", default)]
    pub just_status: Option<String>,
    #[serde(rename = "respPedido", default)]
    pub resp_pedido: Option<RespPedido>,
    #[serde(rename = "statPedido", default)]
    pub stat_pedido: Option<String>,
    #[serde(rename = "statPrazo", default)]
    pub stat_prazo: Option<String>,
    #[serde(rename = "justStaOutra", default)]
    pub just_sta_outra: Option<String>,
    #[serde(rename = "itemPedido", default)]
    pub item_pedido: Option<Vec<ItemPedido>>,
    #[serde(rename = "NSU", default)]
    pub nsu: Option<String>,
    #[serde(rename = "cIdEquip", default)]
    pub c_id_equip: Option<String>,
    #[serde(rename = "xIdEquip", default)]
    pub x_id_equip: Option<String>,
    #[serde(rename = "tpEquip", default)]
    pub tp_equip: Option<String>,
    #[serde(rename = "placaCarreta", default)]
    pub placa_carreta: Option<String>,
    #[serde(rename = "UFCarreta", default)]
    pub uf_carreta: Option<String>,
    #[serde(rename = "placaCarreta2", default)]
    pub placa_carreta2: Option<String>,
    #[serde(rename = "UFCarreta2", default)]
    pub uf_carreta2: Option<String>,
    /// Chave da NF-e referenciada no evento (`chNFeRefte`).
    #[serde(rename = "chNFeRefte", default)]
    pub ch_nfe_refte: Option<String>,

    #[serde(rename = "cPostoUF", default)]
    pub c_posto_uf: Option<String>,
    #[serde(rename = "xPostoUF", default)]
    pub x_posto_uf: Option<String>,
    #[serde(rename = "latGPS", default)]
    pub lat_gps: Option<String>,
    #[serde(rename = "longGPS", default)]
    pub long_gps: Option<String>,
    #[serde(rename = "CPFOper", default)]
    pub cpf_oper: Option<String>,
    #[serde(rename = "xNomeOper", default)]
    pub x_nome_oper: Option<String>,
    #[serde(rename = "dhPas", default)]
    pub dh_pas: Option<String>,
    #[serde(rename = "sentidoVia", default)]
    pub sentido_via: Option<String>,
    #[serde(rename = "placaVeic", default)]
    pub placa_veic: Option<String>,
    #[serde(rename = "xObs", default)]
    pub x_obs: Option<String>,
    #[serde(rename = "tpModal", default)]
    pub tp_modal: Option<String>,
    #[serde(rename = "modalRodov", default)]
    pub modal_rodov: Option<ModalRodov>,
    #[serde(rename = "modalOutro", default)]
    pub modal_outro: Option<ModalOutro>,
    #[serde(rename = "dhEntrega", default)]
    pub dh_entrega: Option<String>,
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
    #[serde(rename = "hashComprovante", default)]
    pub hash_comprovante: Option<String>,
    #[serde(rename = "dhHashComprovante", default)]
    pub dh_hash_comprovante: Option<String>,
    #[serde(rename = "nProtEvento", default)]
    pub n_prot_evento: Option<String>,
    #[serde(rename = "ctg", default)]
    pub ctg: Option<Ctg>,
    #[serde(rename = "diaEmi", default)]
    pub dia_emi: Option<String>,
    #[serde(rename = "indICMS", default)]
    pub ind_icms: Option<String>,
    #[serde(rename = "indICMSST", default)]
    pub ind_icmsst: Option<String>,
    #[serde(rename = "vTotalNFe", default)]
    pub v_total_nfe: Option<String>,
    /// Dados de pagamentos vinculados no evento 110300 (`pgto` - NT 2026.001 / `gPgto` - NT 2026.006).
    #[serde(rename = "pgto", alias = "gPgto", default)]
    pub pgto: Option<Vec<crate::xml_structs::cte::PgtoVincItem>>,
    /// Declaração de pagamento no evento 211110 (`xDecPag`).
    #[serde(rename = "xDecPag", default)]
    pub x_dec_pag: Option<String>,
    /// Grupos do evento 412140 (Tributação do Simples Nacional).
    #[serde(rename = "gTribItemSN", default)]
    pub g_trib_item_sn: Option<Vec<crate::xml_structs::impostos::GTribSn>>,
    #[serde(rename = "gTotalSN", default)]
    pub g_total_sn: Option<crate::xml_structs::impostos::GTribSnTot>,
}

/// Grupo de contingência/passagem em evento de registro de passagem (`<ctg>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Ctg {
    #[serde(rename = "nNF", default)]
    pub n_nf: Option<String>,
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,
    #[serde(rename = "cUF", default)]
    pub c_uf: Option<String>,
    #[serde(rename = "UFDest", default)]
    pub uf_dest: Option<String>,
    #[serde(rename = "diaEmi", default)]
    pub dia_emi: Option<String>,
    #[serde(rename = "vTotalNFe", default)]
    pub v_total_nfe: Option<String>,
    #[serde(rename = "indICMS", default)]
    pub ind_icms: Option<String>,
    #[serde(rename = "indICMSST", default)]
    pub ind_icmsst: Option<String>,
    #[serde(rename = "tpEmis", default)]
    pub tp_emis: Option<String>,
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Outro modal em evento de registro de passagem da NF-e (`<modalOutro>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ModalOutro {
    #[serde(rename = "tpModal", default)]
    pub tp_modal: Option<String>,
    #[serde(rename = "xModal", default)]
    pub x_modal: Option<String>,
    #[serde(rename = "xIdent", default)]
    pub x_ident: Option<String>,
    #[serde(rename = "tpTransp", default)]
    pub tp_transp: Option<String>,
    #[serde(rename = "chMDFe", default)]
    pub ch_mdfe: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Modal rodoviário em evento de registro de passagem da NF-e (`<modalRodov>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ModalRodov {
    #[serde(rename = "placaVeic", default)]
    pub placa_veic: Option<String>,
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,
    #[serde(rename = "UFVeic", default)]
    pub uf_veic: Option<String>,
    #[serde(rename = "placaCarreta", default)]
    pub placa_carreta: Option<String>,
    #[serde(rename = "UFCarreta", default)]
    pub uf_carreta: Option<String>,
    #[serde(rename = "placaCarreta2", default)]
    pub placa_carreta2: Option<String>,
    #[serde(rename = "UFCarreta2", default)]
    pub uf_carreta2: Option<String>,
    #[serde(rename = "tpRodov", default)]
    pub tp_rodov: Option<String>,
    #[serde(rename = "chMDFe", default)]
    pub ch_mdfe: Option<String>,
}

/// Item de Pedido de Prorrogação de ICMS (`<itemPedido>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemPedido {
    #[serde(rename = "@numItem", default)]
    pub num_item: Option<String>,
    #[serde(rename = "qtdeItem", default)]
    pub qtde_item: Option<String>,
    #[serde(rename = "statPedido", default)]
    pub stat_pedido: Option<String>,
    #[serde(rename = "justStaOutra", default)]
    pub just_sta_outra: Option<String>,
    /// Justificativa do status atribuído ao item pelo Fisco (`<justStatus>`).
    #[serde(rename = "justStatus", default)]
    pub just_status: Option<String>,
    /// Status do cancelamento do pedido informado pelo Fisco (`<statCancPedido>`).
    #[serde(rename = "statCancPedido", default)]
    pub stat_canc_pedido: Option<String>,
}

/// Resposta do Fisco a um pedido de ICMS (`<respPedido>` / `<respCancPedido>`).
///
/// O elemento é um **grupo**, não texto simples: agrega o status do prazo
/// (`statPrazo`) e a relação de itens analisados (`itemPedido`).
/// O campo `$text` preserva a tolerância caso o emissor informe apenas texto.
#[derive(Debug, Serialize, Deserialize)]
pub struct RespPedido {
    /// Status do prazo de prorrogação concedido pelo Fisco.
    #[serde(rename = "statPrazo", default)]
    pub stat_prazo: Option<String>,
    /// Status do cancelamento do pedido informado pelo Fisco (`<statCancPedido>`).
    #[serde(rename = "statCancPedido", default)]
    pub stat_canc_pedido: Option<String>,
    /// Itens do pedido analisados, com status e justificativa por item.
    #[serde(rename = "itemPedido", default)]
    pub item_pedido: Option<Vec<ItemPedido>>,
    /// Conteúdo textual do nó, caso informado como texto em vez de grupo.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados correspondentes a Conhecimento de Transporte vinculado ao Evento (`<CTe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Cte {
    /// Chave de acesso única do CT-e correspondente.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Tipo de modal de transporte rodoviário associado.
    #[serde(rename = "modal", default)]
    pub modal: Option<String>,

    /// Data de emissão correspondente ao CT-e vinculado.
    #[serde(rename = "dhEmi", default)]
    pub dh_emi: Option<String>,

    /// Data e hora de processamento do recebimento do CT-e.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,

    /// Protocolo de autorização de uso do CT-e.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Protocolo correspondente ao cancelamento de uso do CT-e.
    #[serde(rename = "nProtCanc", default)]
    pub n_prot_canc: Option<String>,

    /// Protocolo correspondente ao processamento de uso do CT-e.
    #[serde(rename = "nProtCTe", default)]
    pub n_prot_cte: Option<String>,

    /// Protocolo correspondente ao cancelamento do CT-e vinculado (`nProtCTeCanc`).
    #[serde(rename = "nProtCTeCanc", default)]
    pub n_prot_cte_canc: Option<String>,

    /// Data e hora em que ocorreu a entrega das mercadorias.
    #[serde(rename = "dhEntrega", default)]
    pub dh_entrega: Option<String>,

    /// Número identificador correspondente ao documento de entrega.
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,

    /// Nome correspondente ao recebedor das mercadorias.
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,

    /// Hash correspondente à confirmação eletrônica de entrega do CT-e.
    #[serde(rename = "hashEntregaCTe", default)]
    pub hash_entrega_cte: Option<String>,

    /// Data e hora de geração do hash de entrega.
    #[serde(rename = "dhHashEntregaCTe", default)]
    pub dh_hash_entrega_cte: Option<String>,
}

/// Dados correspondentes ao Manifesto de Documentos Fiscais vinculado ao Evento (`<MDFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Mdfe {
    /// Chave de acesso correspondente ao MDF-e vinculado.
    #[serde(rename = "chMDFe", default)]
    pub ch_mdfe: Option<String>,

    /// Chave de acesso correspondente ao CT-e vinculado.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Tipo de modal correspondente ao MDF-e.
    #[serde(rename = "modal", default)]
    pub modal: Option<String>,

    /// Data de emissão correspondente ao MDF-e.
    #[serde(rename = "dhEmi", default)]
    pub dh_emi: Option<String>,

    /// Protocolo de autorização de uso do MDF-e.
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,

    /// Protocolo correspondente ao cancelamento de uso do MDF-e.
    #[serde(rename = "nProtCanc", default)]
    pub n_prot_canc: Option<String>,

    /// Data e hora de processamento do recebimento do MDF-e.
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,
}

#[cfg(test)]
mod tests_prorrogacao_icms {
    use super::*;
    use quick_xml::de::from_str;

    /// Evento real (anonimizado) do Fisco: `<respPedido>` é grupo com
    /// `<statPrazo>` e `<itemPedido>` aninhados.
    ///
    /// `cargo test -- --show-output tests_prorrogacao_icms`
    #[test]
    fn test_resp_pedido_aninhado() {
        let xml = r#"<detEvento versao="1.00"><descEvento>Fisco - Prorrogação ICMS remessa para industrialização</descEvento><idPedido>ID1115003125109999999900019955001000153089142057576301</idPedido><respPedido><statPrazo>1</statPrazo><itemPedido numItem="1"><statPedido>1</statPedido><justStatus>1</justStatus></itemPedido></respPedido></detEvento>"#;

        let det: DetEvento = from_str(xml).expect("Falha ao parsear detEvento de prorrogação");

        assert_eq!(
            det.desc_evento,
            "Fisco - Prorrogação ICMS remessa para industrialização"
        );
        assert!(det.id_pedido.is_some());

        let resp = det
            .resp_pedido
            .expect("respPedido deve ser mapeado como grupo");
        assert_eq!(resp.stat_prazo.as_deref(), Some("1"));

        let itens = resp
            .item_pedido
            .expect("itemPedido aninhado em respPedido deve ser capturado");
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].num_item.as_deref(), Some("1"));
        assert_eq!(itens[0].stat_pedido.as_deref(), Some("1"));
        assert_eq!(itens[0].just_status.as_deref(), Some("1"));
    }

    /// Evento do Fisco: `<respCancPedido>` também é grupo, com
    /// `<statCancPedido>` e `<itemPedido>` aninhados (dados anonimizados).
    ///
    /// `cargo test -- --show-output test_resp_canc_pedido_aninhado`
    #[test]
    fn test_cancelamento_pedido_prorrogacao_nao_cancela_nfe() {
        let xml_prorr: &str = r#"<procEventoNFe versao="1.00"><evento versao="1.00"><infEvento Id="ID111502"><cOrgao>35</cOrgao><tpAmb>1</tpAmb><CNPJ>00000000000100</CNPJ><chNFe>35250100000000000100550010000000011000000011</chNFe><dhEvento>2025-10-01T10:00:00-03:00</dhEvento><tpEvento>111502</tpEvento><nSeqEvento>1</nSeqEvento><detEvento versao="1.00"><descEvento>Fisco - Cancelamento de Pedido de Prorrogação ICMS</descEvento></detEvento></infEvento></evento><retEvento versao="1.00"><infEvento><cStat>135</cStat></infEvento></retEvento></procEventoNFe>"#;
        let p: ProcEventoNfe = from_str(xml_prorr).unwrap();
        assert!(!p.get_info().cancelado);

        let xml_canc: &str = r#"<procEventoNFe versao="1.00"><evento versao="1.00"><infEvento Id="ID110111"><cOrgao>35</cOrgao><tpAmb>1</tpAmb><CNPJ>00000000000100</CNPJ><chNFe>35250100000000000100550010000000011000000011</chNFe><dhEvento>2025-10-01T10:00:00-03:00</dhEvento><tpEvento>110111</tpEvento><nSeqEvento>1</nSeqEvento><detEvento versao="1.00"><descEvento>Cancelamento</descEvento></detEvento></infEvento></evento><retEvento versao="1.00"><infEvento><cStat>135</cStat></infEvento></retEvento></procEventoNFe>"#;
        let p2: ProcEventoNfe = from_str(xml_canc).unwrap();
        assert!(p2.get_info().cancelado);
    }

    #[test]
    fn test_resp_canc_pedido_aninhado() {
        let xml = r#"<detEvento versao="1.00"><descEvento>Fisco - Cancelamento de Pedido de Prorrogação ICMS</descEvento><idPedidoCancelado>ID1115003125109999999900019955001000153089142057576301</idPedidoCancelado><respCancPedido><statCancPedido>1</statCancPedido><itemPedido numItem="1"><statCancPedido>1</statCancPedido><justStatus>1</justStatus></itemPedido></respCancPedido></detEvento>"#;

        let det: DetEvento = from_str(xml).expect("Falha ao parsear detEvento de cancelamento");

        let resp = det
            .resp_canc_pedido
            .expect("respCancPedido deve ser mapeado como grupo");
        assert_eq!(resp.stat_canc_pedido.as_deref(), Some("1"));

        let itens = resp
            .item_pedido
            .expect("itemPedido aninhado em respCancPedido deve ser capturado");
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].num_item.as_deref(), Some("1"));
        assert_eq!(itens[0].stat_canc_pedido.as_deref(), Some("1"));
        assert_eq!(itens[0].just_status.as_deref(), Some("1"));
    }
}
