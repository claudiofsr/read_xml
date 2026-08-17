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
//! read_xml -s procEventoCancNFe_v1.00.xml > procEventoCancNFe_v1.00.rs
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
read_xml -s procEventoCancNFe_v1.00.xml > procEventoCancNFe_v1.00.rs

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

    /// Determina se o evento em questão corresponde a uma homologação de cancelamento.
    pub fn informacao_de_cancelamento(&self) -> bool {
        REGEX_CANCELAMENTO.is_match(&self.evento.inf_evento.det_evento.desc_evento)
    }

    /// Obtém a data de emissão associada ao registro deste evento.
    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.evento.inf_evento.get_dh_evento()
    }

    /// Retorna a estrutura consolidada `InfoNfeEvento` correspondente a este processo.
    pub fn get_info(&self) -> InfoNfeEvento {
        InfoNfeEvento {
            nfe: self.get_nfe(),
            dh_emi: self.get_data_emissao(),
            cancelado: self.informacao_de_cancelamento(),
        }
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
    #[serde(rename = "Signature")]
    pub signature: Signature,
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
    #[serde(rename = "descEvento")]
    pub desc_evento: String,
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
