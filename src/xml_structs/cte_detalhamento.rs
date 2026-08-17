//! # Estruturas Auxiliares e de Detalhamento do CT-e
//!
//! Este módulo gerencia os blocos de dados complementares do Conhecimento de Transporte
//! Eletrônico (CT-e). Abrange informações de carga, documentos transportados,
//! substituições fiscais, informações de tráfego, redespacho e assinaturas auxiliares.
//!
//! Todas as estruturas adotam o zoneamento Serde com o atributo `default`
//! para assegurar compatibilidade de leitura posicional em fluxo (streaming XML) e tolerância
//! a variações regulatórias dos leiautes v3.00 e v4.00.

use serde::{Deserialize, Serialize};

use crate::{
    OptExt,
    xml_structs::{cobranca::Cobranca, entrega::Entrega},
};

/// Trait utilitário para mesclar chaves de acesso de documentos anteriores.
pub trait GetDocs {
    /// Obtém a chave de acesso do CT-e se presente.
    fn get_ch_cte(&self) -> Option<String>;

    /// Obtém a chave de acesso genérica se presente.
    fn get_chave(&self) -> Option<String>;

    /// Consolida chaves de acesso válidas encontradas em um vetor.
    fn merge_keys(&self) -> Vec<String> {
        let ch_cte: Option<String> = self.get_ch_cte();
        let chave: Option<String> = self.get_chave();
        [ch_cte, chave].into_iter().flatten().collect()
    }
}

/// Dados complementares de fluxo e entrega do CT-e (`<compl>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Compl {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Informações de entrega física da carga.
    #[serde(rename = "Entrega", default)]
    pub entrega: Option<Entrega>,

    /// Observações de interesse do contribuinte.
    #[serde(rename = "ObsCont", default)]
    pub obs_cont: Option<Vec<ObsCont>>,

    /// Observações de interesse do fisco.
    #[serde(rename = "ObsFisco", default)]
    pub obs_fisco: Option<Vec<ObsFisco>>,

    /// Destinação do cálculo de frete.
    #[serde(rename = "destCalc", default)]
    pub dest_calc: Option<String>,

    /// Fluxo de tráfego físico da carga.
    #[serde(rename = "fluxo", default)]
    pub fluxo: Option<Fluxo>,

    /// Origem do cálculo de frete.
    #[serde(rename = "origCalc", default)]
    pub orig_calc: Option<String>,

    /// Características adicionais do transporte.
    #[serde(rename = "xCaracAd", default)]
    pub x_carac_ad: Option<String>,

    /// Características do serviço de transporte.
    #[serde(rename = "xCaracSer", default)]
    pub x_carac_ser: Option<String>,

    /// Identificação do emissor do documento.
    #[serde(rename = "xEmi", default)]
    pub x_emi: Option<String>,

    /// Observações gerais em texto livre.
    #[serde(rename = "xObs", default)]
    pub x_obs: Option<String>,
}

/// Dados estruturados de observações do Contribuinte (`<ObsCont>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObsCont {
    /// Identificador do campo de observação.
    #[serde(rename = "@xCampo", default)]
    pub x_campo: Option<String>,

    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Texto descritivo da observação.
    #[serde(rename = "xTexto", default)]
    pub x_texto: Option<String>,
}

/// Dados estruturados de observações do Fisco (`<ObsFisco>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObsFisco {
    /// Identificador do campo fiscal.
    #[serde(rename = "@xCampo", default)]
    pub x_campo: Option<String>,

    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Texto descritivo da observação fiscal.
    #[serde(rename = "xTexto", default)]
    pub x_texto: Option<String>,
}

/// Informações de fluxo físico de tráfego da carga (`<fluxo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fluxo {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Pontos de passagem da carga.
    #[serde(rename = "pass", default)]
    pub pass: Option<Vec<Pass>>,

    /// Destinação física declarada.
    #[serde(rename = "xDest", default)]
    pub x_dest: Option<String>,

    /// Origem física declarada.
    #[serde(rename = "xOrig", default)]
    pub x_orig: Option<String>,

    /// Rota física planejada.
    #[serde(rename = "xRota", default)]
    pub x_rota: Option<String>,
}

/// Ponto de passagem física da carga (`<pass>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pass {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Descrição literal do ponto de passagem.
    #[serde(rename = "xPass", default)]
    pub x_pass: Option<String>,
}

/// Grupo de informações do CT-e Normal (`<infCTeNorm>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteNorm {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados de cobrança (faturas e duplicatas).
    #[serde(rename = "cobr", default)]
    pub cobr: Option<Cobranca>,

    /// Documentos anteriores vinculados.
    #[serde(rename = "docAnt", default)]
    pub doc_ant: Option<DocAnt>,

    /// Informações sobre a carga transportada.
    #[serde(rename = "infCarga", default)]
    pub inf_carga: Option<InfCarga>,

    /// Informações de substituição fiscal se aplicável.
    #[serde(rename = "infCteSub", default)]
    pub inf_cte_sub: Option<InfCteSub>,

    /// Detalhamento dos documentos transportados (NF, NF-e, etc.).
    #[serde(rename = "infDoc", default)]
    pub inf_doc: Option<InfDoc>,

    /// Informações de transporte globalizado se aplicável.
    #[serde(rename = "infGlobalizado", default)]
    pub inf_globalizado: Option<InfGlobalizado>,

    /// Dados de modal de transporte utilizado.
    #[serde(rename = "infModal", default)]
    pub inf_modal: Option<InfModal>,

    /// Serviços de transporte vinculados.
    #[serde(rename = "infServVinc", default)]
    pub inf_serv_vinc: Option<InfServVinc>,

    /// Dados específicos de transporte de veículos novos se aplicável.
    #[serde(rename = "veicNovos", default)]
    pub veic_novos: Option<Vec<VeicNovos>>,
}

impl InfCteNorm {
    /// Obtém as chaves de acesso dos CT-es de documentos anteriores de forma consolidada.
    pub fn get_docs_anteriores(&self) -> Vec<String> {
        match self.doc_ant.as_ref() {
            Some(doc) => doc.get_emissao_de_docs_anteriores(),
            None => Vec::new(),
        }
    }

    /// Obtém as chaves de acesso das NF-es associadas aos documentos transportados.
    pub fn get_info_de_documentos(&self) -> Vec<String> {
        match self.inf_doc.as_ref() {
            Some(info) => info.get_chaves_de_nfes(),
            None => Vec::new(),
        }
    }
}

/// Grupo de Documentos Anteriores de Transporte (`<docAnt>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocAnt {
    /// Lista de emissores de documentos anteriores de transporte.
    #[serde(rename = "emiDocAnt", default)]
    pub emi_doc_ant: Option<Vec<EmiDocAnt>>,
}

impl DocAnt {
    /// Consolida as chaves de documentos anteriores localizadas nos emissores cadastrados.
    pub fn get_emissao_de_docs_anteriores(&self) -> Vec<String> {
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

/// Dados do Emissor de Documentos Anteriores (`<emiDocAnt>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmiDocAnt {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// CNPJ do emissor.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// CPF do emissor.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Inscrição Estadual do emissor.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,

    /// Sigla de Unidade Federativa.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Identificadores físicos e eletrônicos de documentos anteriores.
    #[serde(rename = "idDocAnt", default)]
    pub id_doc_ant: Option<Vec<IdDocAnt>>,

    /// Nome ou Razão Social do emissor.
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
}

impl EmiDocAnt {
    /// Consolida as chaves de documentos eletrônicos anteriores.
    pub fn get_id_de_docs_anteriores(&self) -> Vec<String> {
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

/// Identificadores de Documentos Anteriores (`<idDocAnt>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdDocAnt {
    /// Identificadores de documentos anteriores eletrônicos (CT-e).
    #[serde(rename = "idDocAntEle", default)]
    pub id_doc_ant_ele: Option<Vec<IdDocAntEle>>,

    /// Identificadores de documentos anteriores físicos (papel).
    #[serde(rename = "idDocAntPap", default)]
    pub id_doc_ant_pap: Option<Vec<IdDocAntPap>>,
}

impl IdDocAnt {
    /// Consolida chaves de acesso a partir de subnós eletrônicos.
    pub fn get_docs_anteriores_eletronicos(&self) -> Vec<String> {
        self.id_doc_ant_ele
            .iter()
            .flat_map(|vec_doc_anterior| vec_doc_anterior.iter().flat_map(|doc| doc.merge_keys()))
            .collect()
    }
}

/// Identificação de Documento Anterior Eletrônico (`<idDocAntEle>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdDocAntEle {
    /// Chave de acesso do CT-e anterior.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Chave de acesso auxiliar se declarada.
    #[serde(rename = "chave", default)]
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

/// Identificação de Documento Anterior em Papel (`<idDocAntPap>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdDocAntPap {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Data de emissão do documento físico.
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,

    /// Número identificador do documento físico.
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,

    /// Série do documento físico.
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,

    /// Subsérie do documento físico.
    #[serde(rename = "subser", default)]
    pub subser: Option<String>,

    /// Tipo de documento correspondente.
    #[serde(rename = "tpDoc", default)]
    pub tp_doc: Option<String>,
}

/// Informações da Carga Transportada (`<infCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Informações de quantidades e pesos da carga.
    #[serde(rename = "infQ", default)]
    pub inf_q: Option<Vec<InfQ>>,

    /// Descrição do produto predominante.
    #[serde(rename = "proPred", default)]
    pub pro_pred: Option<String>,

    /// Valor monetário total declarado da carga.
    #[serde(rename = "vCarga", default)]
    pub v_carga: Option<String>,

    /// Valor monetário averbado da carga se aplicável.
    #[serde(rename = "vCargaAverb", default)]
    pub v_carga_averb: Option<String>,

    /// Outras características da carga.
    #[serde(rename = "xOutCat", default)]
    pub x_out_cat: Option<String>,
}

/// Grupo de informações de quantidades faturadas da carga (`<infQ>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfQ {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Unidade de medida correspondente (p. ex., Kg, m³, UN).
    #[serde(rename = "cUnid", default)]
    pub c_unid: Option<String>,

    /// Quantidade medida faturada.
    #[serde(rename = "qCarga", default)]
    pub q_carga: Option<String>,

    /// Tipo de medição realizada (p. ex. peso bruto, peso líquido).
    #[serde(rename = "tpMed", default)]
    pub tp_med: Option<String>,
}

/// Dados específicos correspondentes a CT-e Substituído (`<infCteSub>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteSub {
    /// Chave de acesso do CT-e a ser substituído.
    #[serde(rename = "chCte")]
    pub ch_cte: String,

    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Indicador se houve alteração de tomador de serviço.
    #[serde(rename = "indAlteraToma", default)]
    pub ind_altera_toma: Option<String>,

    /// Chave de acesso do CT-e de anulação se aplicável.
    #[serde(rename = "refCteAnu", default)]
    pub ref_cte_anu: Option<String>,

    /// Dados do tomador e impostos ICMS relacionados.
    #[serde(rename = "tomaICMS", default)]
    pub toma_icms: Option<TomaIcms>,
}

/// Informações do tomador de ICMS para fins de substituição (`<tomaICMS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TomaIcms {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Referência a outro CT-e se aplicável.
    #[serde(rename = "refCte", default)]
    pub ref_cte: Option<String>,

    /// Referência a outra Nota Fiscal se aplicável.
    #[serde(rename = "refNF", default)]
    pub ref_nf: Option<RefNf>,

    /// Referência a outra NF-e se aplicável.
    #[serde(rename = "refNFe", default)]
    pub ref_nfe: Option<String>,
}

/// Dados estruturados de Nota Fiscal referenciada (`<refNF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefNf {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// CNPJ do emitente da nota referenciada.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// CPF do emitente da nota referenciada.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Data de emissão do documento.
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,

    /// Modelo do documento fiscal referenciado.
    #[serde(rename = "mod", default)]
    pub modelo: Option<String>,

    /// Número identificador do documento.
    #[serde(rename = "nro", default)]
    pub nro: Option<String>,

    /// Série do documento.
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,

    /// Sub-série do documento.
    #[serde(rename = "subser", default)]
    pub subserie: Option<String>,

    /// Valor monetário faturado declarado.
    #[serde(rename = "valor", default)]
    pub valor: Option<String>,
}

/// Detalhamento dos documentos transportados pelo CT-e (`<infDoc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfDoc {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Detalhamento de Notas Fiscais em papel.
    #[serde(rename = "infNF", default)]
    pub inf_nf: Option<Vec<InfNf>>,

    /// Detalhamento de Notas Fiscais Eletrônicas (NF-e).
    #[serde(rename = "infNFe", default)]
    pub inf_nfe: Option<Vec<InfNfe>>,

    /// Detalhamento de outros tipos de documentos fiscais.
    #[serde(rename = "infOutros", default)]
    pub inf_outros: Option<Vec<InfOutros>>,
}

impl InfDoc {
    /// Extrai e consolida as chaves de acesso de NF-es localizadas nos subnós.
    pub fn get_chaves_de_nfes(&self) -> Vec<String> {
        match self.inf_nfe.as_ref() {
            Some(vec_info_nfe) => vec_info_nfe
                .iter()
                .map(|i| i.chave.to_string())
                .collect::<Vec<String>>(),
            None => Vec::new(),
        }
    }
}

/// Detalhamento de Nota Fiscal física transportada (`<infNF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfNf {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Código PIN SUFRAMA se aplicável.
    #[serde(rename = "PIN", default)]
    pub pin: Option<String>,

    /// Data de emissão.
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,

    /// Data prevista de entrega.
    #[serde(rename = "dPrev", default)]
    pub d_prev: Option<String>,

    /// Informações de unidades de carga se declaradas.
    #[serde(rename = "infUnidCarga", default)]
    pub inf_unid_carga: Option<Vec<InfUnidCarga>>,

    /// Informações de unidades de transporte se declaradas.
    #[serde(rename = "infUnidTransp", default)]
    pub inf_unid_transp: Option<Vec<InfUnidTransp>>,

    /// Modelo do documento fiscal.
    #[serde(rename = "mod", default)]
    pub modelo: Option<String>,

    /// Código CFOP da operação de transporte.
    #[serde(rename = "nCFOP", default)]
    pub n_cfop: Option<String>,

    /// Número identificador da nota.
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,

    /// Número do pedido de compra relacionado.
    #[serde(rename = "nPed", default)]
    pub n_ped: Option<String>,

    /// Peso total faturado declarado.
    #[serde(rename = "nPeso", default)]
    pub n_peso: Option<String>,

    /// Número de romaneio associado.
    #[serde(rename = "nRoma", default)]
    pub n_roma: Option<String>,

    /// Série da nota.
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,

    /// Valor de base de cálculo de ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,

    /// Valor de base de cálculo de ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,

    /// Valor de ICMS tributado.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,

    /// Valor total faturado líquido declarado (vNF).
    #[serde(rename = "vNF", default)]
    pub v_nf: Option<String>,

    /// Valor total dos produtos declarados (vProd).
    #[serde(rename = "vProd", default)]
    pub v_prod: Option<String>,

    /// Valor do tributo de substituição fiscal.
    #[serde(rename = "vST", default)]
    pub v_st: Option<String>,
}

/// Unidade de carga associada a Notas Fiscais físicas (`<infUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfUnidCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de carga (p. ex., contêiner).
    #[serde(rename = "idUnidCarga", default)]
    pub id_unid_carga: Option<String>,

    /// Lacres de segurança aplicados.
    #[serde(rename = "lacUnidCarga", default)]
    pub lac_unid_carga: Option<Vec<CteLacUnidCarga>>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de carga correspondente.
    #[serde(rename = "tpUnidCarga", default)]
    pub tp_unid_carga: Option<String>,
}

/// Lacre de segurança aplicado em unidades de carga (`<lacUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteLacUnidCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Unidade de transporte de carga vinculada a Notas Fiscais físicas (`<infUnidTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfUnidTransp {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de transporte.
    #[serde(rename = "idUnidTransp", default)]
    pub id_unid_transp: Option<String>,

    /// Unidades de carga acopladas.
    #[serde(rename = "infUnidCarga", default)]
    pub inf_unid_carga: Option<InfoDeCarga>,

    /// Lacres de segurança físicos aplicados.
    #[serde(rename = "lacUnidTransp", default)]
    pub lac_unid_transp: Option<LacreAplicado>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de transporte tracionada (p. ex., semi-reboque).
    #[serde(rename = "tpUnidTransp", default)]
    pub tp_unid_transp: Option<String>,
}

/// Unidade de carga acoplada a unidades de transporte (`<infUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfoDeCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de carga.
    #[serde(rename = "idUnidCarga", default)]
    pub id_unid_carga: Option<String>,

    /// Lacres de segurança físicos de carga.
    #[serde(rename = "lacUnidCarga", default)]
    pub lac_unid_carga: Option<Vec<LacresDeUnidades>>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de carga.
    #[serde(rename = "tpUnidCarga", default)]
    pub tp_unid_carga: Option<String>,
}

/// Lacre de unidade de carga de transporte (`<lacUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacresDeUnidades {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Lacre físico aplicado na unidade de transporte (`<lacUnidTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacreAplicado {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Detalhamento de Nota Fiscal Eletrônica (NF-e) transportada pelo CT-e (`<infNFe>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfNfe {
    /// Chave de acesso correspondente à NF-e (44 dígitos).
    #[serde(rename = "chave")]
    pub chave: String,

    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Código PIN SUFRAMA se aplicável.
    #[serde(rename = "PIN", default)]
    pub pin: Option<String>,

    /// Data prevista de entrega física.
    #[serde(rename = "dPrev", default)]
    pub d_prev: Option<String>,

    /// Unidades de carga acopladas às notas eletrônicas.
    #[serde(rename = "infUnidCarga", default)]
    pub inf_unid_carga: Option<Vec<UnidadeCarga>>,

    /// Unidades de transporte acopladas às notas eletrônicas.
    #[serde(rename = "infUnidTransp", default)]
    pub inf_unid_transp: Option<Vec<UnidadeTransp>>,
}

/// Unidades de carga vinculadas a Notas Fiscais Eletrônicas (`<infUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnidadeCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de carga.
    #[serde(rename = "idUnidCarga", default)]
    pub id_unid_carga: Option<String>,

    /// Lacres de segurança aplicados.
    #[serde(rename = "lacUnidCarga", default)]
    pub lac_unid_carga: Option<Vec<LacresSeguranca>>,

    /// Quantidade rateada declarada.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de carga.
    #[serde(rename = "tpUnidCarga", default)]
    pub tp_unid_carga: Option<String>,
}

/// Lacre de segurança aplicado em unidade de carga de nota eletrônica (`<lacUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacresSeguranca {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Unidade de transporte vinculada a Notas Fiscais Eletrônicas (`<infUnidTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnidadeTransp {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de transporte.
    #[serde(rename = "idUnidTransp", default)]
    pub id_unid_transp: Option<String>,

    /// Unidades de carga acopladas.
    #[serde(rename = "infUnidCarga", default)]
    pub inf_unid_carga: Option<UnidCargaAcoplada>,

    /// Lacres aplicados na unidade de transporte.
    #[serde(rename = "lacUnidTransp", default)]
    pub lac_unid_transp: Option<LacresAplicados>,

    /// Quantidade rateada correspondente.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de transporte.
    #[serde(rename = "tpUnidTransp", default)]
    pub tp_unid_transp: Option<String>,
}

/// Unidade de carga vinculada a transporte de notas eletrônicas (`<infUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnidCargaAcoplada {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de carga.
    #[serde(rename = "idUnidCarga", default)]
    pub id_unid_carga: Option<String>,

    /// Lacres aplicados na unidade de carga.
    #[serde(rename = "lacUnidCarga", default)]
    pub lac_unid_carga: Option<Vec<LacreUnidCarga>>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de carga.
    #[serde(rename = "tpUnidCarga", default)]
    pub tp_unid_carga: Option<String>,
}

/// Lacre de unidade de carga de nota eletrônica (`<lacUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacreUnidCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Lacre físico de transporte de nota eletrônica (`<lacUnidTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacresAplicados {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Detalhamento de Outros documentos transportados pelo CT-e (`<infOutros>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfOutros {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Data de emissão.
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,

    /// Data prevista de entrega física.
    #[serde(rename = "dPrev", default)]
    pub d_prev: Option<String>,

    /// Descrição detalhada dos outros documentos.
    #[serde(rename = "descOutros", default)]
    pub desc_outros: Option<String>,

    /// Unidades de carga acopladas.
    #[serde(rename = "infUnidCarga", default)]
    pub inf_unid_carga: Option<Vec<OutrosInfUnidCarga>>,

    /// Unidades de transporte acopladas.
    #[serde(rename = "infUnidTransp", default)]
    pub inf_unid_transp: Option<Vec<InfOutrosUnidTransp>>,

    /// Número identificador do documento fiscal.
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,

    /// Tipo de documento correspondente.
    #[serde(rename = "tpDoc", default)]
    pub tp_doc: Option<String>,

    /// Valor de documento fiscal declarado.
    #[serde(rename = "vDocFisc", default)]
    pub v_doc_fisc: Option<String>,
}

/// Unidades de carga acopladas a outros documentos transportados (`<infUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutrosInfUnidCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de carga.
    #[serde(rename = "idUnidCarga", default)]
    pub id_unid_carga: Option<String>,

    /// Lacres físicos aplicados.
    #[serde(rename = "lacUnidCarga", default)]
    pub lac_unid_carga: Option<Vec<LacUnidCarga>>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de carga.
    #[serde(rename = "tpUnidCarga", default)]
    pub tp_unid_carga: Option<String>,
}

/// Lacre de unidade de carga de outros documentos (`<lacUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacUnidCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Unidade de transporte associada a outros documentos transportados (`<infUnidTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfOutrosUnidTransp {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de transporte.
    #[serde(rename = "idUnidTransp", default)]
    pub id_unid_transp: Option<String>,

    /// Unidades de carga associadas.
    #[serde(rename = "infUnidCarga", default)]
    pub inf_unid_carga: Option<UnidadeAssociada>,

    /// Lacres físicos aplicados.
    #[serde(rename = "lacUnidTransp", default)]
    pub lac_unid_transp: Option<LacresFisicos>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de transporte.
    #[serde(rename = "tpUnidTransp", default)]
    pub tp_unid_transp: Option<String>,
}

/// Unidade de carga de outros documentos (`<infUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnidadeAssociada {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Identificador da unidade de carga.
    #[serde(rename = "idUnidCarga", default)]
    pub id_unid_carga: Option<String>,

    /// Lacres físicos de carga aplicados.
    #[serde(rename = "lacUnidCarga", default)]
    pub lac_unid_carga: Option<Vec<OutrosLacUnidCarga>>,

    /// Quantidade rateada se aplicável.
    #[serde(rename = "qtdRat", default)]
    pub qtd_rat: Option<String>,

    /// Tipo de unidade de carga.
    #[serde(rename = "tpUnidCarga", default)]
    pub tp_unid_carga: Option<String>,
}

/// Lacre de unidade de carga de outros documentos (`<lacUnidCarga>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutrosLacUnidCarga {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Lacre físico de transporte de outros documentos (`<lacUnidTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LacresFisicos {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número do lacre físico.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,
}

/// Informações de transporte de cargas globalizado (`<infGlobalizado>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfGlobalizado {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Observações do processo de faturamento globalizado.
    #[serde(rename = "xObs", default)]
    pub x_obs: Option<String>,
}

/// Dados do leiaute específico do modal de transporte (`<infModal>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfModal {
    /// Versão do modal declarada.
    #[serde(rename = "@versaoModal", default)]
    pub versao_modal: Option<String>,

    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Detalhamento específico para o modal Rodoviário (rodo).
    #[serde(rename = "rodo", default)]
    pub rodo: Option<Rodo>,
}

/// Dados específicos do Modal Rodoviário (`<rodo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rodo {
    /// Registro Nacional de Transportadores Rodoviários de Carga (RNTRC).
    #[serde(rename = "RNTRC", default)]
    pub rntrc: Option<String>,

    /// Conteúdo de texto bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Grupo de informações sobre serviços de transporte vinculados (`<infServVinc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfServVinc {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista de informações vinculadas de transporte multimodal.
    #[serde(rename = "infCTeMultimodal", default)]
    pub inf_cte_multimodal: Option<Vec<InfCteMultimodal>>,
}

impl InfServVinc {
    /// Extrai as chaves de acesso vinculadas de multimodalidade.
    pub fn get_ctes_multimodal(&self) -> Vec<String> {
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

/// Informações do CT-e Multimodal vinculado (`<infCTeMultimodal>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteMultimodal {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Chave de acesso do CT-e multimodal correspondente.
    #[serde(rename = "chCTeMultimodal", default)]
    pub ch_cte_multimodal: Option<String>,
}

impl InfCteMultimodal {
    /// Retorna a chave de acesso do CT-e multimodal de forma sanitizada.
    pub fn get_ch_cte_multimodal(&self) -> Option<String> {
        self.ch_cte_multimodal.get_key()
    }
}

/// Dados específicos de transporte de Veículos Novos (`<veicNovos>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VeicNovos {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Código correspondente à cor do veículo.
    #[serde(rename = "cCor", default)]
    pub c_cor: Option<String>,

    /// Código correspondente ao modelo do veículo.
    #[serde(rename = "cMod", default)]
    pub c_mod: Option<String>,

    /// Chassi identificador de fabricação.
    #[serde(rename = "chassi", default)]
    pub chassi: Option<String>,

    /// Valor monetário calculado de frete.
    #[serde(rename = "vFrete", default)]
    pub v_frete: Option<String>,

    /// Valor unitário do veículo faturado.
    #[serde(rename = "vUnit", default)]
    pub v_unit: Option<String>,

    /// Descrição da cor do veículo.
    #[serde(rename = "xCor", default)]
    pub x_cor: Option<String>,
}

/// Dados específicos correspondentes a CT-e Anulado (`<infCteAnu>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteAnu {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Chave de acesso do CT-e a ser anulado.
    #[serde(rename = "chCte", default)]
    pub ch_cte: Option<String>,

    /// Data de emissão da anulação.
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,
}

/// Dados complementares de CT-e Complementado (`<infCteComp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteComp {
    /// Chave de acesso do CT-e de referência complementar.
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,

    /// Chave de acesso auxiliar se declarada.
    #[serde(rename = "chave", default)]
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

/// Informações do Provedor de Assinatura Autorizada (`<infPAA>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfPaa {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// CNPJ correspondente ao PAA.
    #[serde(rename = "CNPJPAA", default)]
    pub cnpjpaa: Option<String>,

    /// Assinatura do PAA se aplicável.
    #[serde(rename = "PAASignature", default)]
    pub paasignature: Option<Paasignature>,
}

/// Assinatura criptográfica do Provedor de Assinatura Autorizada (`<PAASignature>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paasignature {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Chave pública RSA correspondente ao PAA.
    #[serde(rename = "RSAKeyValue", default)]
    pub rsakey_value: Option<RsakeyValue>,

    /// Valor bruto da assinatura digital.
    #[serde(rename = "SignatureValue", default)]
    pub signature_value: Option<String>,
}

/// Parâmetros de chave pública RSA (`<RSAKeyValue>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RsakeyValue {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Expoente da chave RSA.
    #[serde(rename = "Exponent", default)]
    pub exponent: Option<String>,

    /// Módulo de cálculo da chave RSA.
    #[serde(rename = "Modulus", default)]
    pub modulus: Option<String>,
}

/// Dados correspondentes a solicitações de Nota Fiscal Fácil (`<infSolicNFF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfSolicNff {
    /// Conteúdo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Código de solicitação de emissão da NFF.
    #[serde(rename = "xSolic", default)]
    pub x_solic: Option<String>,
}
