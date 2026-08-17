//! # Processamento de XML de NF-e
//!
//! Este módulo gerencia o mapeamento, a desserialização e a estruturação de dados
//! extraídos de documentos fiscais eletrônicos correspondentes à Nota Fiscal Eletrônica (NF-e).
//!
//! Fornece suporte robusto e retrocompatível para todas as versões ativas no Brasil,
//! tirando proveito de wrappers `Option<T>` e do atributo `default` para neutralizar
//! variações de leiaute e desordenações de tags no fluxo de leitura.
//!
//! Módulo gerado originalmente a partir de múltiplos esquemas XML (arquivos `.xsd`)
//! utilizando ferramentas como Apache XMLBeans (`xsd2inst`) para geração de instâncias
//! e o utilitário `read_xml` com a opção `-s` para conversão em estruturas Rust de desserialização.
//!
//! Para gerar esquemas a partir do XSD original (requer Apache XMLBeans):
//!
//! ```console
//! xsd2inst procNFe_v4.00.xsd -name nfeProc -dl > procNFe_v4.00.xml
//! xsd2inst nfe_v4.00.xsd -name NFe -dl > nfe_v4.00.xml
//! xsd2inst procNFe_v4.00.xsd -name ProcInutNFe procInutNFe_v4.00.xsd -dl > procInutNFe_v4.00.xml
//! ```
//!
//! E para converter as instâncias XML em estruturas Rust equivalentes:
//!
//! ```console
//! read_xml -s procNFe_v4.00.xml > procNFe_v4.00.rs
//! read_xml -s nfe_v4.00.xml > nfe_v4.00.rs
//! read_xml -s procInutNFe_v4.00.xml > procInutNFe_v4.00.rs
//! ```
//!
//! Maiores detalhes sobre os esquemas podem ser obtidos nos portais oficiais da SEFAZ:
//! * <https://dfe-portal.svrs.rs.gov.br/Nfe>
//! * <https://dfe-portal.svrs.rs.gov.br/NFE/ConsultaSchema>

use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use struct_iterable::Iterable;

use crate::{
    Arguments, GetID, GetKey, InfoExtension, Information, KeysExtension, OptExt, StructExtension,
    serialize_vec_string,
    xml_structs::{
        agente::{Agente, AgenteExtension},
        assinaturas::Signature,
        aut_xml::{AutXML, InfRespTec},
        cobranca::Cobranca,
        entrega::Entrega,
        impostos::Total,
        integrated_dev_env::Ide,
        nfe_detalhamento::*,
        pagamento::Pagamento,
    },
};

/// Representação intermediária e consolidada de uma Nota Fiscal Eletrônica (NF-e).
///
/// Esta estrutura unifica dados do emitente, destinatário, valores de tributos e itens
/// individuais para facilitar a exportação subsequente para planilhas (XLSX) ou arquivos CSV.
#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoNfe {
    /// Versão do leiaute XML correspondente.
    #[serde(rename = "Versão XML", default)]
    versao: Option<String>,

    /// CNPJ do Emitente formatado.
    #[serde(rename = "CNPJ do Emitente", default)]
    pub emitente_cnpj: Option<String>,

    /// CPF do Emitente formatado.
    #[serde(rename = "CPF do Emitente", default)]
    pub emitente_cpf: Option<String>,

    /// Código do Regime Tributário (CRT) do Emitente.
    #[serde(rename = "CRT (Código do Regime Tributário) do Emitente", default)]
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

    /// Sigla do Estado (UF) do Emitente.
    #[serde(rename = "Estado do Emitente", default)]
    emitente_ender_estado: Option<String>,

    /// CNPJ do Destinatário formatado.
    #[serde(rename = "CNPJ do Destinatário", default)]
    destinatario_cnpj: Option<String>,

    /// CPF do Destinatário formatado.
    #[serde(rename = "CPF do Destinatário", default)]
    destinatario_cpf: Option<String>,

    /// Nome ou Razão Social do Destinatário.
    #[serde(rename = "Nome ou Razão Social do Destinatário", default)]
    destinatario_nome: Option<String>,

    /// Município do endereço do Destinatário.
    #[serde(rename = "Municípo do Destinatário", default)]
    destinatario_ender_municipio: Option<String>,

    /// Sigla do Estado (UF) do Destinatário.
    #[serde(rename = "Estado do Destinatário", default)]
    destinatario_ender_estado: Option<String>,

    /// Chave de acesso única do documento fiscal contendo 44 dígitos.
    #[serde(rename = "Chave do Documento Fiscal", default)]
    pub nfe: Option<String>,

    /// Registro indicador da origem do documento (p. ex., "NFe").
    #[serde(rename = "Registro de Origem")]
    doc_tipo: String,

    /// Indicador se o documento fiscal correspondente foi cancelado.
    #[serde(rename = "Cancelado", default)]
    pub cancelado: Option<String>,

    /// Número sequencial da Nota Fiscal.
    #[serde(rename = "Nº do Documento Fiscal", default)]
    numero_da_nota: Option<u32>,

    /// Data de emissão da Nota Fiscal.
    #[serde(
        rename = "Data de Emissão",
        serialize_with = "serialize_option_datetime_to_excel",
        default
    )]
    pub data_emissao: Option<NaiveDate>,

    /// Data em que ocorreu a saída ou a entrega da mercadoria.
    #[serde(
        rename = "Data de Saída / Entrega",
        serialize_with = "serialize_option_datetime_to_excel",
        default
    )]
    data_saida: Option<NaiveDate>,

    /// Número do item dentro da Nota Fiscal.
    #[serde(rename = "Nº do Item", default)]
    pub n_item: Option<u32>,

    /// Quantidade de itens totais contidos na Nota Fiscal.
    #[serde(rename = "Nº de Itens")]
    numero_de_itens: usize,

    /// Número do Documento de Importação (DI/DSI/DA) se aplicável.
    #[serde(rename = "Nº do Documento de Importação", default)]
    n_di: Option<u64>,

    /// Descrição detalhada do produto ou serviço correspondente ao item.
    #[serde(rename = "Descrição", default)]
    pub descricao: Option<String>,

    /// Código Fiscal de Operações e Prestações (CFOP) do item.
    #[serde(rename = "CFOP (Código Fiscal de Operações e Prestações)", default)]
    cfop: Option<u16>,

    /// Código da Nomenclatura Comum do Mercosul (NCM).
    #[serde(rename = "NCM (Nomenclatura Comum do Mercosul)", default)]
    pub ncm: Option<String>,

    /// Informações adicionais de interesse do Contribuinte.
    #[serde(
        rename = "Informações complementares de interesse do Contribuinte",
        default
    )]
    info_adic_contribuinte: Option<String>,

    /// Informações adicionais de interesse do Fisco.
    #[serde(rename = "Informações adicionais de interesse do Fisco", default)]
    info_adic_fisco: Option<String>,

    /// Código de Situação Tributária (CST) do PIS.
    #[serde(rename = "CST de PIS/PASEP", default)]
    cst_pis: Option<u8>,

    /// Código de Situação Tributária (CST) do COFINS.
    #[serde(rename = "CST de COFINS", default)]
    cst_cofins: Option<u8>,

    /// Chaves de CT-e relacionados a este documento fiscal.
    #[serde(
        rename = "Informações de CTes relacionados",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub ctes: Vec<String>,

    /// Detalhes de tomadores vinculados aos CT-es relacionados.
    #[serde(
        rename = "(CNPJ_CPF e Atributo) dos Tomadores dos CTes com Valores e Porcentagens decrescentes",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub tomadores: Vec<String>,

    /// Valor consolidado dos fretes de CT-es correlacionados.
    #[serde(rename = "Valor Total de CTes", default)]
    pub valor_total_ctes: Option<f64>,

    /// Valor Total da Nota Fiscal Eletrônica (vNF).
    #[serde(rename = "Valor Total da NFe", default)]
    pub valor_total_nfe: Option<f64>,

    /// Valor somado correspondente a todos os produtos listados na NF-e.
    #[serde(rename = "Valor Total dos Itens", default)]
    valor_total_itens: Option<f64>,

    /// Valor do produto correspondente a este item específico (vProd).
    #[serde(rename = "Valor do Item", default)]
    pub v_prod: Option<f64>,

    /// Valor de desconto aplicado sobre o item.
    #[serde(rename = "Valor do Desconto", default)]
    v_desc: Option<f64>,

    /// Valor de frete imputado a este item fiscal.
    #[serde(rename = "Valor do Frete", default)]
    v_frete: Option<f64>,

    /// Valor de seguro imputado a este item fiscal.
    #[serde(rename = "Valor do Seguro", default)]
    v_seg: Option<f64>,

    /// Alíquota correspondente ao imposto PIS.
    #[serde(rename = "Alíquota de PIS/PASEP", default)]
    aliq_pis: Option<f64>,

    /// Alíquota correspondente ao imposto COFINS.
    #[serde(rename = "Alíquota de COFINS", default)]
    aliq_cofins: Option<f64>,

    /// Valor monetário calculado correspondente ao PIS do item.
    #[serde(rename = "Valor de PIS/PASEP", default)]
    v_pis: Option<f64>,

    /// Valor monetário calculado correspondente ao COFINS do item.
    #[serde(rename = "Valor de COFINS", default)]
    v_cofins: Option<f64>,

    /// Valor monetário calculado correspondente ao IPI do item.
    #[serde(rename = "Valor de IPI", default)]
    v_ipi: Option<f64>,

    /// Valor monetário calculado correspondente ao ISS do item.
    #[serde(rename = "Valor de ISS", default)]
    v_iss: Option<f64>,

    /// Base de cálculo monetária adotada para apuração do ICMS do item.
    #[serde(rename = "Valor da Base de Cálculo do ICMS", default)]
    v_bc_icms: Option<f64>,

    /// Alíquota percentual do ICMS correspondente.
    #[serde(rename = "Alíquota de ICMS", default)]
    aliq_icms: Option<f64>,

    /// Valor calculado correspondente ao ICMS debitado para o item.
    #[serde(rename = "Valor de ICMS", default)]
    v_icms: Option<f64>,
}

impl InfoNfe {
    /// Retorna a chave de acesso da NF-e caso esteja presente.
    pub fn get_chave(&self) -> Option<String> {
        self.nfe.clone()
    }

    /// Determina se o documento é válido (chave presente e sem marcação de cancelamento).
    pub fn is_valid(&self) -> bool {
        self.nfe.is_some() && self.cancelado.is_none()
    }

    /// Determina se o documento foi cancelado.
    pub fn is_canceled(&self) -> bool {
        self.nfe.is_some() && self.cancelado.is_some()
    }
}

impl KeysExtension for [InfoNfe] {
    /// Extrai e consolida todas as chaves de acesso únicas de uma coleção de notas fiscais.
    fn get_chaves(&self) -> BTreeSet<String> {
        self.iter().flat_map(|info| info.get_chave()).collect()
    }
}

impl InfoExtension for InfoNfe {}

impl GetKey for InfoNfe {
    /// Implementação auxiliar para agregação segura em hash maps de dados cruzados.
    fn get_chave(&self) -> Option<String> {
        self.nfe.clone()
    }
}

impl GetID<Option<(String, u32)>> for InfoNfe {
    /// Gera um identificador único composto pela combinação da chave de acesso e do número do item fiscal.
    fn get_id(&self) -> Option<(String, u32)> {
        if let (Some(nfe), Some(n_item)) = (&self.nfe, self.n_item) {
            Some((nfe.clone(), n_item))
        } else {
            None
        }
    }
}

/// Representa a extração primária dos dados consolidados de um item da NF-e.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Item {
    /// Número sequencial do item.
    n_item: Option<u32>,
    /// Documento de Importação vinculado.
    n_di: Option<u64>,
    /// Nome descritivo do produto.
    x_prod: Option<String>,
    /// CFOP aplicável à transação deste item.
    cfop: Option<u16>,
    /// NCM do produto.
    ncm: Option<String>,
    /// CST correspondente ao imposto PIS.
    cst_pis: Option<u8>,
    /// CST correspondente ao imposto COFINS.
    cst_cofins: Option<u8>,
    /// Valor monetário bruto do produto.
    v_prod: Option<f64>,
    /// Desconto proporcional concedido ao item.
    v_desc: Option<f64>,
    /// Valor proporcional do frete.
    v_frete: Option<f64>,
    /// Valor proporcional do seguro.
    v_seg: Option<f64>,
    /// Alíquota do imposto PIS.
    aliq_pis: Option<f64>,
    /// Alíquota do imposto COFINS.
    aliq_cofins: Option<f64>,
    /// Valor acumulado apurado para o PIS.
    v_pis: Option<f64>,
    /// Valor acumulado apurado para o COFINS.
    v_cofins: Option<f64>,
    /// Valor acumulado apurado para o IPI.
    v_ipi: Option<f64>,
    /// Valor acumulado apurado para o ISS.
    v_iss: Option<f64>,
    /// Base de cálculo apurada para o ICMS.
    v_bc_icms: Option<f64>,
    /// Alíquota adotada para o ICMS.
    aliq_icms: Option<f64>,
    /// Valor de ICMS correspondente.
    v_icms: Option<f64>,
}

/// Estrutura correspondente ao protocolo de recepção e processamento de NF-e (`<nfeProc>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NfeProc {
    /// Versão do layout da distribuição.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,
    /// Namespace XML correspondente ao protocolo.
    #[serde(rename = "@xmlns:nfe", default)]
    pub xmlns_nfe: Option<String>,
    /// Namespace XML correspondente às assinaturas digitais.
    #[serde(rename = "@xmlns:xd", default)]
    pub xmlns_xd: Option<String>,
    /// Texto bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    /// Estrutura contendo o documento fiscal eletrônico principal.
    #[serde(rename = "NFe")]
    pub nfe: Nfe,
    /// Protocolo de autorização de uso do documento fiscal.
    #[serde(rename = "protNFe")]
    pub prot_nfe: ProtNfe,
}

impl StructExtension for NfeProc {
    /// Converte os nós desserializados deste protocolo de NF-e para informações consolidadas.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("nfe xml_path: {xml_path:?}");
            println!("nfe_proc: {self:#?}\n");
        }
        Information::Nfe(self.get_infos())
    }
}

impl NfeProc {
    /// Retorna a versão declarada no nó inicial do protocolo de NF-e.
    pub fn get_versao(&self) -> Option<String> {
        self.versao.clone()
    }

    /// Extrai a chave de acesso da NF-e a partir dos metadados do protocolo de status.
    pub fn get_nfe(&self) -> Option<String> {
        self.prot_nfe.inf_prot.ch_nfe.get_key()
    }

    /// Extrai o CNPJ do Emitente associado à Nota Fiscal.
    pub fn get_emitente_cnpj(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_emit_cnpj())
    }

    /// Extrai o CPF do Emitente associado à Nota Fiscal.
    pub fn get_emitente_cpf(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_emit_cpf())
    }

    /// Extrai a Razão Social ou Nome do Emitente.
    pub fn get_emitente_nome(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_emit_nome())
    }

    /// Extrai o Nome Fantasia do Emitente.
    pub fn get_emitente_fantasia(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_emit_fantasia())
    }

    /// Extrai o Município correspondente ao endereço do Emitente.
    pub fn get_emitente_ender_municipio(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_emit_ender_municipio())
    }

    /// Extrai a sigla da UF correspondente ao endereço do Emitente.
    pub fn get_emitente_ender_estado(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_emit_ender_estado())
    }

    /// Extrai o CNPJ formatado do Destinatário.
    pub fn get_destinatario_cnpj(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_dest_cnpj())
    }

    /// Extrai o CPF formatado do Destinatário.
    pub fn get_destinatario_cpf(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_dest_cpf())
    }

    /// Extrai o Nome ou Razão Social do Destinatário.
    pub fn get_destinatario_nome(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_dest_nome())
    }

    /// Extrai o Município correspondente ao endereço do Destinatário.
    pub fn get_destinatario_ender_municipio(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_dest_ender_municipio())
    }

    /// Extrai a sigla da UF correspondente ao endereço do Destinatário.
    pub fn get_destinatario_ender_estado(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_dest_ender_estado())
    }

    /// Obtém o número do documento fiscal contido na tag de identificação.
    pub fn get_numero_da_nota(&self) -> Option<u32> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.ide.get_num_nfe())
    }

    /// Obtém o Código do Regime Tributário (CRT) do Emitente.
    pub fn get_cod_regime_tributario(&self) -> Option<u8> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_cod_reg_trib())
    }

    /// Obtém a data em que ocorreu a emissão da NF-e.
    pub fn get_data_emissao(&self) -> Option<NaiveDate> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.ide.get_dt_emissao())
    }

    /// Obtém a data de saída das mercadorias/entrega da NF-e.
    pub fn get_data_saida(&self) -> Option<NaiveDate> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.ide.get_dt_saida())
    }

    /// Extrai o valor consolidado final do documento fiscal (vNF).
    pub fn get_total_da_nfe(&self) -> Option<f64> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.total.get_valor_nfe())
    }

    /// Mapeia sequencialmente todos os produtos associados aos itens fiscais contidos no documento.
    pub fn get_itens(&self) -> Vec<Item> {
        match self.nfe.inf_nfe.as_ref() {
            Some(info) => info
                .det
                .iter()
                .map(|item| {
                    let produto = &item.prod;
                    let imposto = &item.imposto;
                    Item {
                        n_item: item.n_item.trim().parse().ok(),
                        n_di: produto.get_num_dec_importacao(),
                        x_prod: produto.get_descricao(),
                        cfop: produto.get_cfop(),
                        ncm: produto.get_ncm(),
                        cst_pis: imposto.get_cst_pis(),
                        cst_cofins: imposto.get_cst_cofins(),
                        v_prod: produto.v_prod,
                        v_desc: produto.v_desc,
                        v_frete: produto.v_frete,
                        v_seg: produto.v_seg,
                        aliq_pis: imposto.get_aliq_pis(),
                        aliq_cofins: imposto.get_aliq_cofins(),
                        v_pis: imposto.get_v_pis(),
                        v_cofins: imposto.get_v_cofins(),
                        v_ipi: imposto.get_v_ipi(),
                        v_iss: imposto.get_v_iss(),
                        v_bc_icms: imposto.get_v_bc_icms(),
                        aliq_icms: imposto.get_aliq_icms(),
                        v_icms: imposto.get_v_icms(),
                    }
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /// Obtém as informações complementares destinadas ao contribuinte.
    pub fn get_info_adic_cpl(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_info_adic_contribuinte())
    }

    /// Obtém as informações adicionais de interesse do fisco.
    pub fn get_info_adic_fisco(&self) -> Option<String> {
        self.nfe
            .inf_nfe
            .as_ref()
            .and_then(|information| information.get_info_adic_fisco())
    }

    /// Retorna uma coleção de estruturas `InfoNfe` unificadas a partir dos itens do XML da NF-e.
    pub fn get_infos(&self) -> Vec<InfoNfe> {
        let mut infos = Vec::new();
        let itens: Vec<Item> = self.get_itens();
        let numero_de_itens = itens.len();
        let valor_total_itens: Option<f64> = itens.iter().map(|item| item.v_prod).sum();

        for item in itens {
            let info_nfe = InfoNfe {
                versao: self.get_versao(),
                emitente_cnpj: self.get_emitente_cnpj(),
                emitente_cpf: self.get_emitente_cpf(),
                emitente_crt: self.get_cod_regime_tributario(),
                emitente_nome: self.get_emitente_nome(),
                emitente_fantasia: self.get_emitente_fantasia(),
                emitente_ender_municipio: self.get_emitente_ender_municipio(),
                emitente_ender_estado: self.get_emitente_ender_estado(),
                destinatario_cnpj: self.get_destinatario_cnpj(),
                destinatario_cpf: self.get_destinatario_cpf(),
                destinatario_nome: self.get_destinatario_nome(),
                destinatario_ender_municipio: self.get_destinatario_ender_municipio(),
                destinatario_ender_estado: self.get_destinatario_ender_estado(),
                nfe: self.get_nfe(),
                doc_tipo: "NFe".to_string(),
                cancelado: None,
                numero_da_nota: self.get_numero_da_nota(),
                data_emissao: self.get_data_emissao(),
                data_saida: self.get_data_saida(),
                n_item: item.n_item,
                numero_de_itens,
                n_di: item.n_di,
                descricao: item.x_prod,
                cfop: item.cfop,
                ncm: item.ncm,
                info_adic_contribuinte: self.get_info_adic_cpl(),
                info_adic_fisco: self.get_info_adic_fisco(),
                cst_pis: item.cst_pis,
                cst_cofins: item.cst_cofins,
                ctes: Vec::new(),
                tomadores: Vec::new(),
                valor_total_ctes: None,
                valor_total_nfe: self.get_total_da_nfe(),
                valor_total_itens,
                v_prod: item.v_prod, // valor do Item
                v_desc: item.v_desc,
                v_frete: item.v_frete,
                v_seg: item.v_seg,
                aliq_pis: item.aliq_pis,
                aliq_cofins: item.aliq_cofins,
                v_pis: item.v_pis,
                v_cofins: item.v_cofins,
                v_ipi: item.v_ipi,
                v_iss: item.v_iss,
                v_bc_icms: item.v_bc_icms,
                aliq_icms: item.aliq_icms,
                v_icms: item.v_icms,
            };
            infos.push(info_nfe.clone());
        }

        infos
    }
}

/// Representação estrutural direta da tag `<NFe>` correspondente ao documento.
#[derive(Debug, Serialize, Deserialize)]
pub struct Nfe {
    /// Atributo do namespace XML do documento.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    /// Bloco de informações detalhadas da NF-e.
    #[serde(rename = "infNFe", default)]
    pub inf_nfe: Option<InfNfe>,
    /// Informações suplementares da NF-e (p. ex. QR Code).
    #[serde(rename = "infNFeSupl", default)]
    pub inf_nfe_supl: Option<InfNfeSupl>,
    /// Bloco contendo a assinatura digital da NF-e.
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

/// Bloco de Informações da Nota Fiscal Eletrônica (`<infNFe>`).
///
/// ### Organização de Zoneamento Estrutural
///
/// Para suportar flexibilidade total a ordenações variante de tags vindas de múltiplos
/// emissores, esta estrutura adota o seguinte zoneamento de desserialização posicional do Serde:
///
/// 1. **Atributos XML no Topo (`@`)**: Capturados prioritariamente no início do parse,
///    mapeados para campos `Option<T>` com `default`.
/// 2. **Campos Opcionais no Centro (`#[serde(default)]`)**: Todos os blocos opcionais que
///    podem omitir sua presença no XML estão centralizados aqui e parametrizados de forma robusta.
/// 3. **Campos Estritamente Obrigatórios na Base**: Todos os nós declarados como indispensáveis
///    pela especificação nacional da SEFAZ para consistência física do documento (`ide`, `det`, `total`, `transp`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfe {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Sempre mapeados no topo com fallback default)
    // =========================================================================
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Tolerantes a ausência ou inversão física de tags)
    // =========================================================================
    /// Emitente da NF-e.
    #[serde(rename = "emit", default)]
    pub emitente: Option<Agente>,

    /// Destinatário da NF-e.
    #[serde(rename = "dest", default)]
    pub destinatario: Option<Agente>,

    /// Lista de pessoas autorizadas a acessar o arquivo XML.
    #[serde(rename = "autXML", default)]
    pub aut_xml: Option<Vec<AutXML>>,

    /// Informações de Nota Fiscal emitida por órgão avulso.
    #[serde(rename = "avulsa", default)]
    pub avulsa: Option<Avulsa>,

    /// Informações do local de retirada física da mercadoria.
    #[serde(rename = "retirada", default)]
    pub retirada: Option<Retirada>,

    /// Informações do local de entrega física da mercadoria.
    #[serde(rename = "entrega", default)]
    pub entrega: Option<Entrega>,

    /// Dados de Cobrança (Faturas, duplicatas, etc.).
    #[serde(rename = "cobr", default)]
    pub cobr: Option<Cobranca>,

    /// Dados de Pagamento (Formas de pagamento, troco, etc.).
    #[serde(rename = "pag", default)]
    pub pag: Option<Pagamento>,

    /// Informações do intermediador de transação financeira.
    #[serde(rename = "infIntermed", default)]
    pub inf_intermed: Option<InfIntermed>,

    /// Bloco para inserção de informações adicionais (Fisco e Contribuinte).
    #[serde(rename = "infAdic", default)]
    pub inf_adic: Option<InfAdic>,

    /// Detalhes de exportação vinculados.
    #[serde(rename = "exporta", default)]
    pub exporta: Option<Exporta>,

    /// Detalhes de compra pública ou corporativa.
    #[serde(rename = "compra", default)]
    pub compra: Option<Compra>,

    /// Grupo de dados de cana-de-açúcar para produtores rurais.
    #[serde(rename = "cana", default)]
    pub cana: Option<Cana>,

    /// Detalhes cadastrais de responsabilidade técnica de software houses.
    #[serde(rename = "infRespTec", default)]
    pub inf_resp_tec: Option<InfRespTec>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS (Sem default, posicionados na base)
    // =========================================================================
    /// Dados de identificação do documento fiscal eletrônico.
    #[serde(rename = "ide")]
    pub ide: Ide,

    /// Lista contendo os nós de detalhes de itens de mercadorias.
    #[serde(rename = "det")]
    pub det: Vec<Detalhes>,

    /// Totais consolidados de tributos, produtos e valores finais.
    #[serde(rename = "total")]
    pub total: Total,

    /// Detalhes sobre o transporte de cargas associado à nota.
    #[serde(rename = "transp")]
    pub transp: Transp,
}

impl InfNfe {
    /// Extrai o CNPJ cadastrado do Emitente.
    fn get_emit_cnpj(&self) -> Option<String> {
        self.emitente.get_ext_cnpj()
    }

    /// Extrai o CPF cadastrado do Emitente.
    fn get_emit_cpf(&self) -> Option<String> {
        self.emitente.get_ext_cpf()
    }

    /// Extrai a Razão Social cadastrada do Emitente.
    fn get_emit_nome(&self) -> Option<String> {
        self.emitente.get_ext_nome()
    }

    /// Extrai o Nome Fantasia cadastrado do Emitente.
    fn get_emit_fantasia(&self) -> Option<String> {
        self.emitente.get_ext_fantasia()
    }

    /// Extrai o Município cadastrado do Emitente.
    fn get_emit_ender_municipio(&self) -> Option<String> {
        self.emitente.get_ext_municipio()
    }

    /// Extrai o Estado correspondente ao endereço do Emitente.
    fn get_emit_ender_estado(&self) -> Option<String> {
        self.emitente.get_ext_estado()
    }

    /// Código do Regime Tributário.
    fn get_cod_reg_trib(&self) -> Option<u8> {
        self.emitente.get_ext_crt()
    }

    /// Extrai o CNPJ cadastrado do Destinatário.
    fn get_dest_cnpj(&self) -> Option<String> {
        self.destinatario.get_ext_cnpj()
    }

    /// Extrai o CPF cadastrado do Destinatário.
    fn get_dest_cpf(&self) -> Option<String> {
        self.destinatario.get_ext_cpf()
    }

    /// Extrai o Nome ou Razão Social do Destinatário.
    fn get_dest_nome(&self) -> Option<String> {
        self.destinatario.get_ext_nome()
    }

    /// Extrai o Município cadastrado do Destinatário.
    fn get_dest_ender_municipio(&self) -> Option<String> {
        self.destinatario.get_ext_municipio()
    }

    /// Extrai o Estado correspondente ao endereço do Destinatário.
    fn get_dest_ender_estado(&self) -> Option<String> {
        self.destinatario.get_ext_estado()
    }

    /// Retorna as informações adicionais de interesse do contribuinte de forma sanitizada.
    fn get_info_adic_contribuinte(&self) -> Option<String> {
        self.inf_adic.as_ref().and_then(|information| {
            information
                .inf_cpl
                .as_ref()
                .map(|i| i.trim().replace_multiple_whitespaces())
        })
    }

    /// Retorna as informações adicionais de interesse do fisco de forma sanitizada.
    fn get_info_adic_fisco(&self) -> Option<String> {
        self.inf_adic.as_ref().and_then(|information| {
            information
                .inf_ad_fisco
                .as_ref()
                .map(|i| i.trim().replace_multiple_whitespaces())
        })
    }
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
    /// `cargo test -- --show-output deserialize_xml_nfe_a`
    fn deserialize_xml_nfe_a() -> XmlParserResult<()> {
        let mut nfe_chaves = Vec::new();

        let xmls = [
            "35220412345678901234567890123456789012345678_NFe.xml",
            "35220956789012345678901234560030301250352046_NFe.xml",
        ];

        for xml in xmls {
            println!("xml: {xml}");
            let path = Path::new(xml);

            // Tenta desserializar o XML na estrutura NfeProc correspondente.
            let nfe_proc = NfeProc::xml_parse(path)?;

            if xml == xmls[0] {
                println!("nfe_proc: {nfe_proc:#?}");
            }

            let nfe = nfe_proc.get_nfe();
            println!("nfe: {nfe:?}");

            nfe_chaves.push(nfe);
        }

        let result = [
            Some("45678901234567890123456789012345678901234567".to_string()),
            Some("12345678901234567890123456789012345678901234".to_string()),
        ];

        assert_eq!(nfe_chaves, result);

        Ok(())
    }

    #[test]
    /// `cargo test -- --show-output deserialize_xml_nfe_b`
    fn deserialize_xml_nfe_b() -> XmlParserResult<()> {
        let mut nfe_chaves = Vec::new();

        let xmls = [
            "35250147964911003894550050003122171001027340_NFe.xml",
            "35250199999999999999550000000000021001034139_NFe.xml",
        ];

        // ---------------------------------------------------------------------
        // VALIDAÇÃO: Amostra XML 1 (Campos ausentes: 'dest', 'cobr'; Ordem: 'total' no topo)
        // ---------------------------------------------------------------------
        {
            let path_1 = Path::new(xmls[0]);
            let nfe_proc_1 = NfeProc::xml_parse(path_1)?;

            // Recupera a chave e adiciona ao vetor de controle
            let chave_1 = nfe_proc_1.get_nfe();
            nfe_chaves.push(chave_1);

            // Valida atributos mapeados no topo da InfNfe
            let inf_nfe_1 = nfe_proc_1.nfe.inf_nfe.as_ref().unwrap();
            assert_eq!(
                inf_nfe_1.id,
                Some("NFe12345678901234567890123456789012345678901234".to_string())
            );
            assert_eq!(inf_nfe_1.versao, Some("4.00".to_string()));

            // Valida se o bloco "emit" (opcional no meio) foi mapeado mesmo em ordem aleatória
            let emit_1 = inf_nfe_1.emitente.as_ref().unwrap();
            assert_eq!(emit_1.cnpj, Some("12345678901234".to_string()));
            assert_eq!(
                emit_1.x_nome,
                Some("BETA COMERCIO DE ALIMENTOS LTDA".to_string())
            );

            // Valida se "destinatario" (dest) e "cobranca" (cobr) foram resolvidos como None (ausentes)
            assert!(inf_nfe_1.destinatario.is_none());
            assert!(inf_nfe_1.cobr.is_none());

            // Valida campos obrigatórios (ide, det, total, transp)
            assert_eq!(inf_nfe_1.ide.num_cte, None); // Esta estrutura mapeia NF-e, não CT-e
            assert_eq!(inf_nfe_1.det.len(), 1);
            assert_eq!(inf_nfe_1.det[0].prod.c_prod, Some("ITEM123".to_string()));
            assert_eq!(inf_nfe_1.det[0].prod.v_prod, Some(1234.56));
            assert_eq!(
                inf_nfe_1.total.icmstot.as_ref().unwrap().v_nf,
                Some("123.45".to_string())
            );
            assert_eq!(inf_nfe_1.transp.mod_frete, Some("4".to_string()));

            // Valida bloco opcional "infAdic"
            let inf_adic_1 = inf_nfe_1.inf_adic.as_ref().unwrap();
            assert_eq!(
                inf_adic_1.inf_cpl,
                Some("INFORMACOES ADICIONAIS FICTICIAS - AGRADECEMOS A PARCERIA.".to_string())
            );
        }

        // ---------------------------------------------------------------------
        // VALIDAÇÃO: Amostra XML 2 (Campos ausentes: 'infAdic'; Ordem: 'pag' no topo)
        // ---------------------------------------------------------------------
        {
            let path_2 = Path::new(xmls[1]);
            let nfe_proc_2 = NfeProc::xml_parse(path_2)?;

            // Recupera a chave e adiciona ao vetor de controle
            let chave_2 = nfe_proc_2.get_nfe();
            nfe_chaves.push(chave_2);

            let inf_nfe_2 = nfe_proc_2.nfe.inf_nfe.as_ref().unwrap();
            assert_eq!(
                inf_nfe_2.id,
                Some("NFe12345678901234567890123456789012345678901234".to_string())
            );

            // Valida bloco "dest" presente neste XML
            let dest_2 = inf_nfe_2.destinatario.as_ref().unwrap();
            assert_eq!(dest_2.cnpj, Some("12345678901234".to_string()));
            assert_eq!(
                dest_2.x_nome,
                Some("COMERCIO DE ALIMENTOS FANTASIA LTDA".to_string())
            );

            // Valida bloco "cobr" presente neste XML
            let cobr_2 = inf_nfe_2.cobr.as_ref().unwrap();
            assert_eq!(
                cobr_2.fat.as_ref().unwrap().n_fat,
                Some("FAT-123".to_string())
            );
            assert_eq!(
                cobr_2.fat.as_ref().unwrap().v_orig,
                Some("1234.56".to_string())
            );

            // Valida que "infAdic" foi resolvido como None (ausente)
            assert!(inf_nfe_2.inf_adic.is_none());

            // Valida campos obrigatórios da base mapeados com sucesso
            assert_eq!(inf_nfe_2.det[0].prod.cfop, Some("1234".to_string())); // CFOP em IDE mapeado se presente, ou tratado
            assert_eq!(inf_nfe_2.det[0].prod.c_prod, Some("ITEM001".to_string()));
            assert_eq!(
                inf_nfe_2.total.icmstot.as_ref().unwrap().v_nf,
                Some("1234.56".to_string())
            );
        }

        // Validação final do comportamento das chaves extraídas
        let result = [
            Some("12345678901234567890123456789012345678901234".to_string()),
            Some("12345678901234567890123456789012345678904444".to_string()),
        ];

        assert_eq!(nfe_chaves, result);

        Ok(())
    }
}
