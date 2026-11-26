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
//! read-xml -s procCTe_v3.00.xml > procCTe_v3.00.rs
//!
//! Mais informações em:
//! * <https://dfe-portal.svrs.rs.gov.br/Cte>
//! * <https://dfe-portal.svrs.rs.gov.br/CTE/ConsultaSchema>
//!
//! No Manjaro Linux, install qxmledit:
//! yay -S qxmledit
//! Make PDF files from XSD.

use chrono::NaiveDate;
use claudiofsr_lib::{BTreeSetExtension, UniqueElements as _};
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
        cobranca::Cobranca,
        cte_detalhamento::*,
        endereco::{Endereco, EnderecoExtension},
        impostos::{Ibscbs, Ibscbstot, Imposto},
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
    pub versao: Option<String>,

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
    pub emitente_crt: Option<u8>,

    /// Nome ou Razão Social do Emitente.
    #[serde(rename = "Nome ou Razão Social do Emitente", default)]
    pub emitente_nome: Option<String>,

    /// Nome Fantasia do Emitente.
    #[serde(rename = "Nome Fantasia do Emitente", default)]
    pub emitente_fantasia: Option<String>,

    /// Município do endereço do Emitente.
    #[serde(rename = "Municípo do Emitente", default)]
    pub emitente_ender_municipio: Option<String>,

    /// Estado (UF) do Emitente.
    #[serde(rename = "Estado do Emitente", default)]
    pub emitente_ender_estado: Option<String>,

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
    pub remetente_crt: Option<u8>,

    /// Nome ou Razão Social do Remetente.
    #[serde(rename = "Nome ou Razão Social do Remetente", default)]
    pub remetente_nome: Option<String>,

    /// Nome Fantasia do Remetente.
    #[serde(rename = "Nome Fantasia do Remetente", default)]
    pub remetente_fantasia: Option<String>,

    /// Município do endereço do Remetente.
    #[serde(rename = "Municípo do Remetente", default)]
    pub remetente_ender_municipio: Option<String>,

    /// Estado (UF) do Remetente.
    #[serde(rename = "Estado do Remetente", default)]
    pub remetente_ender_estado: Option<String>,

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
    pub destinatario_crt: Option<u8>,

    /// Nome ou Razão Social do Destinatário.
    #[serde(rename = "Nome ou Razão Social do Destinatário", default)]
    pub destinatario_nome: Option<String>,

    /// Nome Fantasia do Destinatário.
    #[serde(rename = "Nome Fantasia do Destinatário", default)]
    pub destinatario_fantasia: Option<String>,

    /// Município do endereço do Destinatário.
    #[serde(rename = "Municípo do Destinatário", default)]
    pub destinatario_ender_municipio: Option<String>,

    /// Estado (UF) do Destinatário.
    #[serde(rename = "Estado do Destinatário", default)]
    pub destinatario_ender_estado: Option<String>,

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
    pub expedidor_crt: Option<u8>,

    /// Nome ou Razão Social do Expedidor.
    #[serde(rename = "Nome ou Razão Social do Expedidor", default)]
    pub expedidor_nome: Option<String>,

    /// Nome Fantasia do Expedidor.
    #[serde(rename = "Nome Fantasia do Expedidor", default)]
    pub expedidor_fantasia: Option<String>,

    /// Município do endereço do Expedidor.
    #[serde(rename = "Municípo do Expedidor", default)]
    pub expedidor_ender_municipio: Option<String>,

    /// Estado (UF) do Expedidor.
    #[serde(rename = "Estado do Expedidor", default)]
    pub expedidor_ender_estado: Option<String>,

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
    pub recebedor_crt: Option<u8>,

    /// Nome ou Razão Social do Recebedor.
    #[serde(rename = "Nome ou Razão Social do Recebedor", default)]
    pub recebedor_nome: Option<String>,

    /// Nome Fantasia do Recebedor.
    #[serde(rename = "Nome Fantasia do Recebedor", default)]
    pub recebedor_fantasia: Option<String>,

    /// Município do endereço do Recebedor.
    #[serde(rename = "Municípo do Recebedor", default)]
    pub recebedor_ender_municipio: Option<String>,

    /// Estado (UF) do Recebedor.
    #[serde(rename = "Estado do Recebedor", default)]
    pub recebedor_ender_estado: Option<String>,

    /// CNPJ do Tomador do serviço formatado.
    #[serde(rename = "CNPJ do Tomador", default)]
    pub tomador_cnpj: Option<String>,

    /// CPF do Tomador do serviço formatado.
    #[serde(rename = "CPF do Tomador", default)]
    pub tomador_cpf: Option<String>,

    /// Nome ou Razão Social do Tomador.
    #[serde(rename = "Nome ou Razão Social do Tomador", default)]
    pub tomador_nome: Option<String>,

    /// Nome Fantasia do Tomador.
    #[serde(rename = "Nome Fantasia do Tomador", default)]
    pub tomador_fantasia: Option<String>,

    /// Município do endereço do Tomador.
    #[serde(rename = "Municípo do Tomador", default)]
    pub tomador_ender_municipio: Option<String>,

    /// Estado (UF) do Tomador.
    #[serde(rename = "Estado do Tomador", default)]
    pub tomador_ender_estado: Option<String>,

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
    pub doc_tipo: String,

    /// Indicador se o documento foi cancelado.
    #[serde(rename = "Cancelado", default)]
    pub cancelado: Option<String>,

    /// Número sequencial da Nota Fiscal/Documento.
    #[serde(rename = "Nº do Documento Fiscal", default)]
    pub numero_da_nota: Option<u32>,

    /// Natureza da Operação (`<natOp>`).
    #[serde(rename = "Natureza da Operação", default)]
    pub nat_operacao: Option<String>,

    /// UF de início da prestação (`<UFIni>`).
    #[serde(rename = "UF de Início da Prestação", default)]
    pub uf_inicio: Option<String>,

    /// Município de início da prestação (`<xMunIni>`).
    #[serde(rename = "Município de Início da Prestação", default)]
    pub mun_inicio: Option<String>,

    /// UF de término da prestação (`<UFFim>`).
    #[serde(rename = "UF de Término da Prestação", default)]
    pub uf_termino: Option<String>,

    /// Município de término da prestação (`<xMunFim>`).
    #[serde(rename = "Município de Término da Prestação", default)]
    pub mun_termino: Option<String>,

    /// Nome do local de entrega (`<compl><Entrega><xNome>`).
    #[serde(rename = "Local de Entrega: Nome", default)]
    pub local_entrega_nome: Option<String>,

    /// Observações gerais do conhecimento (`<compl><xObs>`).
    #[serde(rename = "Observações Gerais", default)]
    pub obs_gerais: Option<String>,

    /// Código CFOP aplicável ao serviço.
    #[serde(rename = "CFOP (Código Fiscal de Operações e Prestações)", default)]
    pub cfop: Option<u16>,

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
    pub cte_anteriores: Vec<String>,

    /// Chaves de NF-es de terceiros vinculadas à operação.
    #[serde(
        rename = "NFes Vinculados",
        serialize_with = "serialize_vec_string",
        default
    )]
    pub nfes_vinculados: Vec<String>,

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
    pub aliq_pis: Option<f64>,

    /// Alíquota correspondente ao imposto COFINS.
    #[serde(rename = "Alíquota de COFINS", default)]
    pub aliq_cofins: Option<f64>,

    /// Valor monetário calculado de PIS.
    #[serde(rename = "Valor de PIS/PASEP", default)]
    pub v_pis: Option<f64>,

    /// Valor monetário calculado de COFINS.
    #[serde(rename = "Valor de COFINS", default)]
    pub v_cofins: Option<f64>,

    /// Valor monetário calculado de IPI se aplicável.
    #[serde(rename = "Valor de IPI", default)]
    pub v_ipi: Option<f64>,

    /// Base de cálculo monetária do ISS se aplicável.
    #[serde(rename = "Valor da Base de Cálculo do ISS", default)]
    pub v_bc_iss: Option<f64>,

    /// Valor monetário calculado de ISS se aplicável.
    #[serde(rename = "Valor de ISS", default)]
    pub v_iss: Option<f64>,

    /// Base de cálculo monetária apurada para o ICMS de transporte.
    #[serde(rename = "Valor da Base de Cálculo do ICMS", default)]
    pub v_bc_icms: Option<f64>,

    /// Alíquota correspondente ao ICMS de transporte.
    #[serde(rename = "Alíquota de ICMS", default)]
    pub aliq_icms: Option<f64>,

    /// Valor calculado correspondente ao ICMS debitado.
    #[serde(rename = "Valor de ICMS", default)]
    pub v_icms: Option<f64>,

    /// Valor calculado correspondente ao ICMS-ST.
    #[serde(rename = "Valor de ICMS ST", default)]
    pub v_icms_st: Option<f64>,
}

crate::impl_sanitize_text!(
    InfoCte,
    versao,
    emitente_cnpj,
    emitente_cpf,
    emitente_nome,
    emitente_fantasia,
    emitente_ender_municipio,
    emitente_ender_estado,
    remetente_cnpj,
    remetente_cpf,
    remetente_nome,
    remetente_fantasia,
    remetente_ender_municipio,
    remetente_ender_estado,
    remetente_nfes,
    destinatario_cnpj,
    destinatario_cpf,
    destinatario_nome,
    destinatario_fantasia,
    destinatario_ender_municipio,
    destinatario_ender_estado,
    expedidor_cnpj,
    expedidor_cpf,
    expedidor_nome,
    expedidor_fantasia,
    expedidor_ender_municipio,
    expedidor_ender_estado,
    recebedor_cnpj,
    recebedor_cpf,
    recebedor_nome,
    recebedor_fantasia,
    recebedor_ender_municipio,
    recebedor_ender_estado,
    tomador_cnpj,
    tomador_cpf,
    tomador_nome,
    tomador_fantasia,
    tomador_ender_municipio,
    tomador_ender_estado,
    cte,
    doc_tipo,
    cancelado,
    nat_operacao,
    uf_inicio,
    mun_inicio,
    uf_termino,
    mun_termino,
    local_entrega_nome,
    obs_gerais,
    cte_complementar,
    cte_anteriores,
    nfes_vinculados,
    nfes,
    ncm_descricao,
);

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

    /// Recupera a identificação base de um participante (primeiros 10 chars do CNPJ ou 11 do CPF).
    fn get_base_doc(cnpj: &Option<String>, cpf: &Option<String>) -> Option<String> {
        cnpj.get_first(10)
            .or_else(|| cpf.get_first(11))
            .get_not_empty()
    }

    /// Infere o código do Tomador apenas quando ausente no XML, preservando o código explícito
    /// (ex.: `4` = Outros/Terceiro mesmo quando o CNPJ base coincide com outro participante).
    pub fn corrigir_codigo_do_tomador(&mut self) {
        if self.tomador_codigo.is_some() {
            return;
        }

        let Some(base_toma) = Self::get_base_doc(&self.tomador_cnpj, &self.tomador_cpf) else {
            return;
        };

        for (codigo, opt_base_agente) in [
            (
                0,
                Self::get_base_doc(&self.remetente_cnpj, &self.remetente_cpf),
            ),
            (
                1,
                Self::get_base_doc(&self.expedidor_cnpj, &self.expedidor_cpf),
            ),
            (
                2,
                Self::get_base_doc(&self.recebedor_cnpj, &self.recebedor_cpf),
            ),
            (
                3,
                Self::get_base_doc(&self.destinatario_cnpj, &self.destinatario_cpf),
            ),
        ] {
            if opt_base_agente.as_deref() == Some(base_toma.as_str()) {
                self.tomador_codigo = Some(codigo);
                return;
            }
        }

        self.tomador_codigo = Some(4);
    }

    /// Retorna o CNPJ ou CPF base inferido correspondente ao tomador após correção de papéis.
    pub fn get_cnpj_cpf_base_do_tomador(&self) -> Option<String> {
        match self.tomador_codigo {
            Some(0) => Self::get_base_doc(&self.remetente_cnpj, &self.remetente_cpf),
            Some(1) => Self::get_base_doc(&self.expedidor_cnpj, &self.expedidor_cpf),
            Some(2) => Self::get_base_doc(&self.recebedor_cnpj, &self.recebedor_cpf),
            Some(3) => Self::get_base_doc(&self.destinatario_cnpj, &self.destinatario_cpf),
            Some(4) => Self::get_base_doc(&self.tomador_cnpj, &self.tomador_cpf),
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

    /// Preenche os dados cadastrais de qualquer agente (`emit`, `rem`, `dest`, `exped`, `receb`).
    fn fill_emitente(&mut self, emit: Option<&Agente>) {
        self.emitente_cnpj = emit.and_then(Agente::get_cnpj);
        self.emitente_cpf = emit.and_then(Agente::get_cpf);
        self.emitente_crt = emit.and_then(Agente::get_crt);
        self.emitente_nome = emit.and_then(Agente::get_nome);
        self.emitente_fantasia = emit.and_then(Agente::get_fantasia);
        self.emitente_ender_municipio = emit.and_then(Agente::get_endereco_municipio);
        self.emitente_ender_estado = emit.and_then(Agente::get_endereco_estado);
    }

    fn fill_remetente(&mut self, rem: Option<&Agente>) {
        self.remetente_cnpj = rem.and_then(Agente::get_cnpj);
        self.remetente_cpf = rem.and_then(Agente::get_cpf);
        self.remetente_crt = rem.and_then(Agente::get_crt);
        self.remetente_nome = rem.and_then(Agente::get_nome);
        self.remetente_fantasia = rem.and_then(Agente::get_fantasia);
        self.remetente_ender_municipio = rem.and_then(Agente::get_endereco_municipio);
        self.remetente_ender_estado = rem.and_then(Agente::get_endereco_estado);
        self.remetente_nfes = rem.map_or_else(Vec::new, Agente::get_nfes);
    }

    fn fill_destinatario(&mut self, dest: Option<&Agente>) {
        self.destinatario_cnpj = dest.and_then(Agente::get_cnpj);
        self.destinatario_cpf = dest.and_then(Agente::get_cpf);
        self.destinatario_crt = dest.and_then(Agente::get_crt);
        self.destinatario_nome = dest.and_then(Agente::get_nome);
        self.destinatario_fantasia = dest.and_then(Agente::get_fantasia);
        self.destinatario_ender_municipio = dest.and_then(Agente::get_endereco_municipio);
        self.destinatario_ender_estado = dest.and_then(Agente::get_endereco_estado);
    }

    fn fill_expedidor(&mut self, exped: Option<&Agente>) {
        self.expedidor_cnpj = exped.and_then(Agente::get_cnpj);
        self.expedidor_cpf = exped.and_then(Agente::get_cpf);
        self.expedidor_crt = exped.and_then(Agente::get_crt);
        self.expedidor_nome = exped.and_then(Agente::get_nome);
        self.expedidor_fantasia = exped.and_then(Agente::get_fantasia);
        self.expedidor_ender_municipio = exped.and_then(Agente::get_endereco_municipio);
        self.expedidor_ender_estado = exped.and_then(Agente::get_endereco_estado);
    }

    fn fill_recebedor(&mut self, receb: Option<&Agente>) {
        self.recebedor_cnpj = receb.and_then(Agente::get_cnpj);
        self.recebedor_cpf = receb.and_then(Agente::get_cpf);
        self.recebedor_crt = receb.and_then(Agente::get_crt);
        self.recebedor_nome = receb.and_then(Agente::get_nome);
        self.recebedor_fantasia = receb.and_then(Agente::get_fantasia);
        self.recebedor_ender_municipio = receb.and_then(Agente::get_endereco_municipio);
        self.recebedor_ender_estado = receb.and_then(Agente::get_endereco_estado);
    }

    /// Preenche dados do tomador a partir de `<ide>` com fallback para `<toma>`.
    fn fill_tomador(&mut self, ide: &Ide, toma_alt: Option<&Agente>) {
        self.tomador_cnpj = toma_alt
            .and_then(Agente::get_cnpj)
            .or_else(|| ide.get_toma_cnpj());
        self.tomador_cpf = toma_alt
            .and_then(Agente::get_cpf)
            .or_else(|| ide.get_toma_cpf());
        self.tomador_nome = toma_alt
            .and_then(Agente::get_nome)
            .or_else(|| ide.get_toma_nome());
        self.tomador_fantasia = toma_alt
            .and_then(Agente::get_fantasia)
            .or_else(|| ide.get_toma_fantasia());
        self.tomador_ender_municipio = toma_alt
            .and_then(Agente::get_endereco_municipio)
            .or_else(|| ide.get_toma_ender_municipio());
        self.tomador_ender_estado = toma_alt
            .and_then(Agente::get_endereco_estado)
            .or_else(|| ide.get_toma_ender_estado());
    }

    /// Preenche os campos tributários de PIS, COFINS, IPI, ISS e ICMS a partir de `<imp>`.
    fn fill_imposto(&mut self, imposto: Option<&Imposto>) {
        self.aliq_pis = imposto.and_then(Imposto::get_aliq_pis);
        self.aliq_cofins = imposto.and_then(Imposto::get_aliq_cofins);
        self.v_pis = imposto.and_then(Imposto::get_v_pis);
        self.v_cofins = imposto.and_then(Imposto::get_v_cofins);
        self.v_ipi = imposto.and_then(Imposto::get_v_ipi);
        self.v_bc_iss = imposto.and_then(Imposto::get_v_bc_iss);
        self.v_iss = imposto.and_then(Imposto::get_v_iss);
        self.v_bc_icms = imposto.and_then(Imposto::get_v_bc_icms);
        self.aliq_icms = imposto.and_then(Imposto::get_aliq_icms);
        self.v_icms = imposto.and_then(Imposto::get_v_icms);
        self.v_icms_st = imposto.and_then(Imposto::get_v_icms_st_cte);
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
        self.prot_cte
            .inf_prot
            .ch_cte
            .get_key()
            .or_else(|| self.cte.get_chave_from_id())
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
        let mut info = self.cte.get_info(self.get_cte());
        apply_cancel_from_prot(&self.prot_cte, &mut info);
        info
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
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

impl StructExtension for Cte {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cte xml_path: {xml_path:?}");
            println!("cte: {self:#?}\n");
        }
        Information::Cte(Box::new(self.get_info(self.get_chave_from_id())))
    }
}

impl Cte {
    /// Extrai a chave de acesso de 44 dígitos do atributo `@Id` de `<infCte>`.
    pub fn get_chave_from_id(&self) -> Option<String> {
        crate::extract_chave_from_id(self.inf_cte.as_ref()?.id.as_deref(), "CTe")
    }

    /// Monta a estrutura consolidada `InfoCte` a partir de `<CTe>` e de uma chave opcional.
    pub fn get_info(&self, ch_cte: Option<String>) -> InfoCte {
        let inf_cte = self.inf_cte.as_ref();
        let compl = inf_cte.and_then(|inf| inf.compl.as_ref());
        let (ide_nfes, ide_ctes) = inf_cte
            .map(|inf| inf.ide.get_correlated_refs())
            .unwrap_or_default();
        let cte_complementar: Vec<String> = inf_cte
            .iter()
            .flat_map(|inf| inf.get_cte_mult())
            .chain(inf_cte.iter().flat_map(|inf| inf.get_cte_comp()))
            .chain(
                inf_cte
                    .and_then(|inf| inf.inf_cte_anu.as_ref())
                    .and_then(|a| a.ch_cte.get_key()),
            )
            .chain(
                inf_cte
                    .and_then(|inf| inf.inf_cte_norm.as_ref())
                    .into_iter()
                    .flat_map(InfCteNorm::get_sub_ctes),
            )
            .chain(ide_ctes)
            .collect();
        let nfes_vinculados: Vec<String> = inf_cte
            .iter()
            .flat_map(|inf| inf.get_info_de_nfes())
            .chain(
                inf_cte
                    .and_then(|inf| inf.inf_cte_norm.as_ref())
                    .into_iter()
                    .flat_map(InfCteNorm::get_sub_nfes),
            )
            .chain(ide_nfes)
            .collect();

        let local_entrega_nome = inf_cte
            .and_then(|inf| inf.destinatario.as_ref())
            .and_then(|d| d.loc_ent.as_deref())
            .and_then(|le| le.x_nome.get_not_empty())
            .or_else(|| compl.and_then(Compl::get_local_entrega_nome));

        let mut info_cte = InfoCte {
            versao: inf_cte.and_then(|inf| inf.versao.clone()),
            tomador_codigo: inf_cte.and_then(|inf| inf.ide.get_cod_tomador()),
            cte: ch_cte,
            doc_tipo: "CTe".to_string(),
            numero_da_nota: inf_cte.and_then(|inf| inf.ide.get_num_cte()),
            nat_operacao: inf_cte.and_then(|inf| inf.ide.nat_operacao.get_not_empty()),
            uf_inicio: inf_cte.and_then(|inf| inf.ide.ufini.get_not_empty()),
            mun_inicio: inf_cte.and_then(|inf| inf.ide.x_mun_ini.get_not_empty()),
            uf_termino: inf_cte.and_then(|inf| inf.ide.uffim.get_not_empty()),
            mun_termino: inf_cte.and_then(|inf| inf.ide.x_mun_fim.get_not_empty()),
            local_entrega_nome,
            obs_gerais: compl.and_then(Compl::get_obs_gerais),
            cfop: inf_cte.and_then(|inf| inf.ide.get_cfop()),
            data_emissao: inf_cte.and_then(|inf| inf.ide.get_dt_emissao()),
            cte_complementar,
            cte_anteriores: inf_cte.map_or_else(Vec::new, InfCte::get_info_de_ctes),
            nfes_vinculados,
            valor_total: inf_cte.map(|inf| inf.v_prest.v_tprest),
            ..Default::default()
        };

        if let Some(inf) = inf_cte {
            info_cte.fill_emitente(inf.emitente.as_ref());
            info_cte.fill_remetente(inf.remetente.as_ref());
            info_cte.fill_destinatario(inf.destinatario.as_ref());
            info_cte.fill_expedidor(inf.expedidor.as_ref());
            info_cte.fill_recebedor(inf.recebedor.as_ref());
            info_cte.fill_tomador(&inf.ide, None);
            info_cte.fill_imposto(Some(&inf.imposto));
        }

        info_cte.corrigir_codigo_do_tomador();
        info_cte.get_unique_elements();
        info_cte
    }
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
    #[serde(rename = "pgtoVinc", default)]
    pub pgto_vinc: Option<PgtoVinc>,

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
    #[serde(rename = "vTPrestLiq", default)]
    pub v_tprest_liq: Option<f64>,
}

impl VPrest {
    /// Retorna o valor líquido da prestação de serviço (`vTPrestLiq`), se presente.
    #[inline]
    pub fn get_valor_liquido(&self) -> Option<f64> {
        self.v_tprest_liq
    }
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

// =========================================================================
// GTV-e (Guia de Transporte de Valores Eletrônica - Modelo 64, NT 2020.002)
// =========================================================================

/// Envelope XML de processamento da GTV-e (`<GTVeProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GtveProc {
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

    #[serde(rename = "GTVe")]
    pub gtve: Gtve,
    #[serde(rename = "protCTe")]
    pub prot_cte: ProtCte,
}

impl StructExtension for GtveProc {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("gtve xml_path: {xml_path:?}");
            println!("gtve_proc: {self:#?}\n");
        }
        let ch_cte = self
            .prot_cte
            .inf_prot
            .ch_cte
            .get_key()
            .or_else(|| self.gtve.get_chave_from_id());
        let mut info = self.gtve.get_info(ch_cte);
        apply_cancel_from_prot(&self.prot_cte, &mut info);
        Information::Cte(Box::new(info))
    }
}

#[inline]
fn apply_cancel_from_prot(prot_cte: &ProtCte, info: &mut InfoCte) {
    if prot_cte
        .inf_prot
        .c_stat
        .as_deref()
        .is_some_and(|s| matches!(s.trim(), "101" | "151" | "155"))
    {
        info.cancelado = Some("Sim".to_string());
    }
}

/// Tag `<GTVe>` (avulsa ou dentro de `<GTVeProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Gtve {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "infCte", default)]
    pub inf_cte: Option<GtveInfCte>,
    #[serde(rename = "infCTeSupl", default)]
    pub inf_cte_supl: Option<InfCteSupl>,
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

impl StructExtension for Gtve {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("gtve xml_path: {xml_path:?}");
            println!("gtve: {self:#?}\n");
        }
        Information::Cte(Box::new(self.get_info(self.get_chave_from_id())))
    }
}

impl Gtve {
    pub fn get_chave_from_id(&self) -> Option<String> {
        crate::extract_chave_from_id(self.inf_cte.as_ref()?.id.as_deref(), "CTe")
    }

    pub fn get_info(&self, ch_cte: Option<String>) -> InfoCte {
        let inf = self.inf_cte.as_ref();
        let compl = inf.and_then(|i| i.compl.as_ref());
        let (ide_nfes, ide_ctes) = inf.map(|i| i.ide.get_correlated_refs()).unwrap_or_default();
        let valor_total = inf
            .and_then(|i| i.det_gtv.as_ref())
            .and_then(DetGtv::get_valor_total);

        let mut info_cte = InfoCte {
            versao: inf.and_then(|i| i.versao.clone()),
            tomador_codigo: inf.and_then(|i| {
                match i.ide.get_cod_tomador() {
                    Some(1) if i.ide.tomador.is_none() => Some(3),
                    other => other,
                }
                .or_else(|| i.ide.tomador.as_ref().and(Some(4)))
            }),

            cte: ch_cte,
            doc_tipo: "GTVe".to_string(),
            cancelado: None,
            numero_da_nota: inf.and_then(|i| i.ide.get_num_cte()),
            nat_operacao: inf.and_then(|i| i.ide.nat_operacao.get_not_empty()),
            uf_inicio: inf
                .and_then(|i| i.ide.ufini.get_not_empty())
                .or_else(|| inf.and_then(|i| i.origem.get_endereco_ext_estado()))
                .or_else(|| inf.and_then(|i| i.remetente.get_ext_estado())),
            mun_inicio: inf
                .and_then(|i| i.ide.x_mun_ini.get_not_empty())
                .or_else(|| inf.and_then(|i| i.origem.get_endereco_ext_municipio()))
                .or_else(|| inf.and_then(|i| i.remetente.get_ext_municipio())),
            uf_termino: inf
                .and_then(|i| i.ide.uffim.get_not_empty())
                .or_else(|| inf.and_then(|i| i.destino.get_endereco_ext_estado()))
                .or_else(|| inf.and_then(|i| i.destinatario.get_ext_estado())),
            mun_termino: inf
                .and_then(|i| i.ide.x_mun_fim.get_not_empty())
                .or_else(|| inf.and_then(|i| i.destino.get_endereco_ext_municipio()))
                .or_else(|| inf.and_then(|i| i.destinatario.get_ext_municipio())),
            local_entrega_nome: compl.and_then(Compl::get_local_entrega_nome),
            obs_gerais: compl.and_then(Compl::get_obs_gerais),
            cfop: inf.and_then(|i| i.ide.get_cfop()),
            data_emissao: inf.and_then(|i| i.ide.get_dt_emissao()),
            cte_complementar: ide_ctes,
            nfes_vinculados: ide_nfes,
            valor_total,
            ..Default::default()
        };

        if let Some(i) = inf {
            info_cte.fill_emitente(i.emitente.as_ref());
            info_cte.fill_remetente(i.remetente.as_ref());
            info_cte.fill_destinatario(i.destinatario.as_ref());
            info_cte.fill_tomador(&i.ide, None);
        }

        info_cte.corrigir_codigo_do_tomador();
        info_cte.get_unique_elements();
        info_cte
    }
}

/// Conteúdo `<infCte>` específico da GTV-e.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GtveInfCte {
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    #[serde(rename = "emit", default)]
    pub emitente: Option<Agente>,
    #[serde(rename = "rem", default)]
    pub remetente: Option<Agente>,
    #[serde(rename = "dest", default)]
    pub destinatario: Option<Agente>,
    #[serde(rename = "origem", default)]
    pub origem: Option<Endereco>,
    #[serde(rename = "destino", default)]
    pub destino: Option<Endereco>,
    #[serde(rename = "compl", default)]
    pub compl: Option<Compl>,
    #[serde(rename = "detGTV", default)]
    pub det_gtv: Option<DetGtv>,
    #[serde(rename = "autXML", default)]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "infRespTec", default)]
    pub inf_resp_tec: Option<InfRespTec>,
    #[serde(rename = "infSolicNFF", default)]
    pub inf_solic_nff: Option<InfSolicNff>,
    #[serde(rename = "pgtoVinc", default)]
    pub pgto_vinc: Option<PgtoVinc>,

    #[serde(rename = "ide")]
    pub ide: Ide,
}

/// Detalhamento da GTV-e (`<detGTV>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetGtv {
    #[serde(rename = "infEspecie", default)]
    pub inf_especie: Option<Vec<InfEspecie>>,
    #[serde(rename = "infVeiculo", default)]
    pub inf_veiculo: Option<Vec<InfVeiculoGtve>>,
    #[serde(rename = "qCarga", default)]
    pub q_carga: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl DetGtv {
    /// Soma os valores declarados em `<infEspecie><vEspecie>`.
    pub fn get_valor_total(&self) -> Option<f64> {
        let especies = self.inf_especie.as_ref()?;
        let mut has_value = false;
        let sum = especies
            .iter()
            .filter_map(|e| e.v_especie.to_float64())
            .inspect(|_| has_value = true)
            .sum();
        has_value.then_some(sum)
    }
}

/// Espécie transportada na GTV-e (`<infEspecie>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfEspecie {
    #[serde(rename = "tpEspecie", default)]
    pub tp_especie: Option<String>,
    #[serde(rename = "vEspecie", default)]
    pub v_especie: Option<String>,
    #[serde(rename = "tpNumerario", default)]
    pub tp_numerario: Option<String>,
    #[serde(rename = "xMoedaEstr", default)]
    pub x_moeda_estr: Option<String>,
}

/// Veículo informado na GTV-e (`<infVeiculo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfVeiculoGtve {
    #[serde(rename = "placa", default)]
    pub placa: Option<String>,
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,
    #[serde(rename = "RNTRC", default)]
    pub rntrc: Option<String>,
}

// =========================================================================
// CT-e Simplificado (NT 2024.002 / NT 2026.004)
// =========================================================================

/// Envelope XML de processamento do CT-e Simplificado (`<cteSimpProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteSimpProc {
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

    #[serde(rename = "CTeSimp")]
    pub cte_simp: CteSimp,
    #[serde(rename = "protCTe")]
    pub prot_cte: ProtCte,
}

impl StructExtension for CteSimpProc {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cteSimp xml_path: {xml_path:?}");
            println!("cte_simp_proc: {self:#?}\n");
        }
        let ch_cte = self
            .prot_cte
            .inf_prot
            .ch_cte
            .get_key()
            .or_else(|| self.cte_simp.get_chave_from_id());
        let mut info = self.cte_simp.get_info(ch_cte);
        apply_cancel_from_prot(&self.prot_cte, &mut info);
        Information::Cte(Box::new(info))
    }
}

/// Tag `<CTeSimp>` (avulsa ou dentro de `<cteSimpProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteSimp {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "infCte", default)]
    pub inf_cte: Option<CteSimpInfCte>,
    #[serde(rename = "infCTeSupl", default)]
    pub inf_cte_supl: Option<InfCteSupl>,
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

impl StructExtension for CteSimp {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cteSimp xml_path: {xml_path:?}");
            println!("cte_simp: {self:#?}\n");
        }
        Information::Cte(Box::new(self.get_info(self.get_chave_from_id())))
    }
}

impl CteSimp {
    pub fn get_chave_from_id(&self) -> Option<String> {
        crate::extract_chave_from_id(self.inf_cte.as_ref()?.id.as_deref(), "CTe")
    }

    pub fn get_info(&self, ch_cte: Option<String>) -> InfoCte {
        let inf = self.inf_cte.as_ref();
        let imposto = inf.and_then(|i| i.imposto.as_ref());
        let compl = inf.and_then(|i| i.compl.as_ref());
        let dets = inf.and_then(|i| i.det.as_deref()).unwrap_or(&[]);
        let first_det = dets.first();

        let mut nfes_vinculados: Vec<String> = Vec::new();
        let mut cte_anteriores: Vec<String> = Vec::new();
        let mut sum_det_prest = 0.0f64;
        let mut has_det_prest = false;

        for d in dets {
            if let Some(vp) = d.v_prest {
                sum_det_prest += vp;
                has_det_prest = true;
            }
            if let Some(nfes) = &d.inf_nfe {
                nfes_vinculados.extend(nfes.iter().filter_map(|n| n.chave.get_key()));
            }
            if let Some(docs_ant) = &d.inf_doc_ant {
                for ant in docs_ant {
                    if let Some(ch) = ant.ch_cte.get_key() {
                        cte_anteriores.push(ch);
                    }
                    if let Some(parciais) = &ant.inf_nfe_transp_parcial {
                        nfes_vinculados.extend(parciais.iter().filter_map(|p| p.ch_nfe.get_key()));
                    }
                }
            }
        }

        let (ide_nfes, ide_ctes) = inf.map(|i| i.ide.get_correlated_refs()).unwrap_or_default();
        let cte_complementar: Vec<String> = inf
            .and_then(|i| i.inf_cte_sub.as_ref())
            .into_iter()
            .flat_map(InfCteSub::get_ctes)
            .chain(ide_ctes)
            .collect();
        if let Some(sub) = inf.and_then(|i| i.inf_cte_sub.as_ref()) {
            nfes_vinculados.extend(sub.get_nfes());
        }
        nfes_vinculados.extend(ide_nfes);

        let valor_total = inf
            .and_then(|i| i.total.as_ref())
            .and_then(|t| t.v_tprest)
            .or_else(|| has_det_prest.then_some(sum_det_prest));

        let mut info_cte = InfoCte {
            versao: inf.and_then(|i| i.versao.clone()),
            tomador_codigo: inf
                .and_then(|i| i.ide.get_cod_tomador())
                .or_else(|| inf.and_then(|i| i.toma.as_ref().and(Some(4)))),

            cte: ch_cte,
            doc_tipo: "CTeSimp".to_string(),
            cancelado: None,
            numero_da_nota: inf.and_then(|i| i.ide.get_num_cte()),
            nat_operacao: inf.and_then(|i| i.ide.nat_operacao.get_not_empty()),
            uf_inicio: inf.and_then(|i| i.ide.ufini.get_not_empty()),
            mun_inicio: inf
                .and_then(|i| i.ide.x_mun_ini.get_not_empty())
                .or_else(|| first_det.and_then(|d| d.x_mun_ini.get_not_empty())),
            uf_termino: inf.and_then(|i| i.ide.uffim.get_not_empty()),
            mun_termino: inf
                .and_then(|i| i.ide.x_mun_fim.get_not_empty())
                .or_else(|| first_det.and_then(|d| d.x_mun_fim.get_not_empty())),
            local_entrega_nome: compl.and_then(Compl::get_local_entrega_nome),
            obs_gerais: compl.and_then(Compl::get_obs_gerais),
            cfop: inf.and_then(|i| i.ide.get_cfop()),
            data_emissao: inf.and_then(|i| i.ide.get_dt_emissao()),
            cte_complementar,
            cte_anteriores,
            nfes_vinculados,
            valor_total,
            ..Default::default()
        };

        if let Some(i) = inf {
            info_cte.fill_emitente(i.emitente.as_ref());
            info_cte.fill_tomador(&i.ide, i.toma.as_ref());
            info_cte.fill_imposto(imposto);
        }

        info_cte.corrigir_codigo_do_tomador();
        info_cte.get_unique_elements();
        info_cte
    }
}

/// Conteúdo `<infCte>` específico do CT-e Simplificado.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteSimpInfCte {
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    #[serde(rename = "emit", default)]
    pub emitente: Option<Agente>,
    #[serde(rename = "toma", default)]
    pub toma: Option<Agente>,
    #[serde(rename = "compl", default)]
    pub compl: Option<Compl>,
    #[serde(rename = "infCarga", default)]
    pub inf_carga: Option<InfCarga>,
    #[serde(rename = "det", default)]
    pub det: Option<Vec<DetCteSimp>>,
    #[serde(rename = "infModal", default)]
    pub inf_modal: Option<InfModal>,
    #[serde(rename = "cobr", default)]
    pub cobr: Option<Cobranca>,
    #[serde(rename = "infCteSub", default)]
    pub inf_cte_sub: Option<InfCteSub>,
    #[serde(rename = "imp", default)]
    pub imposto: Option<Imposto>,
    #[serde(rename = "total", default)]
    pub total: Option<TotalCteSimp>,
    #[serde(rename = "autXML", default)]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "infRespTec", default)]
    pub inf_resp_tec: Option<InfRespTec>,
    #[serde(rename = "infSolicNFF", default)]
    pub inf_solic_nff: Option<InfSolicNff>,
    #[serde(rename = "infPAA", default)]
    pub inf_paa: Option<InfPaa>,
    #[serde(rename = "pgtoVinc", default)]
    pub pgto_vinc: Option<PgtoVinc>,

    #[serde(rename = "ide")]
    pub ide: Ide,
}

/// Detalhamento de cada entrega/prestação no CT-e Simplificado (`<det>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetCteSimp {
    #[serde(rename = "@nItem", default)]
    pub n_item: Option<String>,
    #[serde(rename = "cMunIni", default)]
    pub c_mun_ini: Option<String>,
    #[serde(rename = "xMunIni", default)]
    pub x_mun_ini: Option<String>,
    #[serde(rename = "cMunFim", default)]
    pub c_mun_fim: Option<String>,
    #[serde(rename = "xMunFim", default)]
    pub x_mun_fim: Option<String>,
    #[serde(rename = "vPrest", default)]
    pub v_prest: Option<f64>,
    #[serde(rename = "vRec", default)]
    pub v_rec: Option<f64>,
    #[serde(rename = "vPrestLiq", default)]
    pub v_prest_liq: Option<f64>,
    #[serde(rename = "Comp", default)]
    pub comp: Option<Vec<Comp>>,
    #[serde(rename = "infNFe", default)]
    pub inf_nfe: Option<Vec<InfNfe>>,
    #[serde(rename = "infDocAnt", default)]
    pub inf_doc_ant: Option<Vec<InfDocAntSimp>>,
    #[serde(rename = "IBSCBS", default)]
    pub ibscbs: Option<Box<Ibscbs>>,
}

/// Documentos anteriores referenciados no item do CT-e Simplificado (`<infDocAnt>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfDocAntSimp {
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,
    #[serde(rename = "tpPrest", default)]
    pub tp_prest: Option<String>,
    #[serde(rename = "infNFeTranspParcial", default)]
    pub inf_nfe_transp_parcial: Option<Vec<InfNfeTranspParcial>>,
}

/// NF-e de transporte parcial vinculada a documento anterior no CT-e Simplificado (`<infNFeTranspParcial>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfNfeTranspParcial {
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,
}

/// Totais consolidados do CT-e Simplificado (`<total>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TotalCteSimp {
    #[serde(rename = "vTPrest", default)]
    pub v_tprest: Option<f64>,
    #[serde(rename = "vTPrestLiq", default)]
    pub v_tprest_liq: Option<f64>,
    #[serde(rename = "vTRec", default)]
    pub v_trec: Option<f64>,
    #[serde(rename = "vTotDFe", default)]
    pub v_tot_dfe: Option<f64>,
    #[serde(rename = "IBSCBSTot", default)]
    pub ibscbstot: Option<Box<Ibscbstot>>,
}

// =========================================================================
// Vinculação de Pagamentos (NT 2026.001 - Split Payment)
// =========================================================================

/// Grupo de informações da vinculação com a transação de pagamento (`<pgtoVinc>` - NT 2026.001).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgtoVinc {
    #[serde(rename = "pgto", default)]
    pub pgto: Option<Vec<PgtoVincItem>>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados de cada pagamento previsto/vinculado (`<pgto>` - NT 2026.001).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgtoVincItem {
    #[serde(rename = "@nPag", default)]
    pub n_pag_attr: Option<String>,
    #[serde(rename = "nPag", default)]
    pub n_pag: Option<String>,
    #[serde(rename = "idTransacao", default)]
    pub id_transacao: Option<String>,
    #[serde(rename = "tpMeioPgto", default)]
    pub tp_meio_pgto: Option<String>,
    #[serde(rename = "CNPJReceb", default)]
    pub cnpj_receb: Option<String>,
    #[serde(rename = "CNPJBasePSP", default)]
    pub cnpj_base_psp: Option<String>,
    #[serde(rename = "gMultaJuros", default)]
    pub g_multa_juros: Option<GMultaJuros>,
}

/// Multas e juros vinculados ao pagamento (`<gMultaJuros>` - NT 2026.004 / NT 2026.006).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GMultaJuros {
    #[serde(rename = "chCTeRef", default)]
    pub ch_cte_ref: Option<String>,
    #[serde(rename = "vMulta", default)]
    pub v_multa: Option<String>,
    #[serde(rename = "vJuros", default)]
    pub v_juros: Option<String>,
}

// =========================================================================
// CT-e OS (Conhecimento de Transporte Eletrônico para Outros Serviços - Mod. 67)
// =========================================================================

/// Envelope XML de processamento do CT-e OS (`<cteOSProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteOsProc {
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

    #[serde(rename = "CTeOS")]
    pub cte_os: CteOs,
    #[serde(rename = "protCTe")]
    pub prot_cte: ProtCte,
}

impl StructExtension for CteOsProc {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cteOS xml_path: {xml_path:?}");
            println!("cte_os_proc: {self:#?}\n");
        }
        let ch_cte = self
            .prot_cte
            .inf_prot
            .ch_cte
            .get_key()
            .or_else(|| self.cte_os.get_chave_from_id());
        let mut info = self.cte_os.get_info(ch_cte);
        apply_cancel_from_prot(&self.prot_cte, &mut info);
        Information::Cte(Box::new(info))
    }
}

/// Tag `<CTeOS>` (avulsa ou dentro de `<cteOSProc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteOs {
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,
    #[serde(rename = "infCte", default)]
    pub inf_cte: Option<CteOsInfCte>,
    #[serde(rename = "infCTeSupl", default)]
    pub inf_cte_supl: Option<InfCteSupl>,
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

impl StructExtension for CteOs {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("cteOS xml_path: {xml_path:?}");
            println!("cte_os: {self:#?}\n");
        }
        Information::Cte(Box::new(self.get_info(self.get_chave_from_id())))
    }
}

impl CteOs {
    pub fn get_chave_from_id(&self) -> Option<String> {
        crate::extract_chave_from_id(self.inf_cte.as_ref()?.id.as_deref(), "CTe")
    }

    pub fn get_info(&self, ch_cte: Option<String>) -> InfoCte {
        let inf = self.inf_cte.as_ref();
        let compl = inf.and_then(|i| i.compl.as_ref());
        let norm = inf.and_then(|i| i.inf_cte_norm.as_ref());
        let (ide_nfes, ide_ctes) = inf.map(|i| i.ide.get_correlated_refs()).unwrap_or_default();

        let cte_complementar: Vec<String> = inf
            .and_then(|i| i.inf_cte_comp.as_ref())
            .into_iter()
            .flatten()
            .flat_map(InfCteComp::merge_keys)
            .chain(
                inf.and_then(|i| i.inf_cte_anu.as_ref())
                    .and_then(|a| a.ch_cte.get_key()),
            )
            .chain(
                norm.and_then(|n| n.inf_cte_sub.as_ref())
                    .into_iter()
                    .flat_map(InfCteSub::get_ctes),
            )
            .chain(norm.and_then(|n| n.ref_cte_canc.get_key()))
            .chain(
                norm.and_then(|n| n.inf_gtve.as_ref())
                    .into_iter()
                    .flatten()
                    .filter_map(|g| g.ch_cte.get_key()),
            )
            .chain(ide_ctes)
            .collect();

        let mut nfes_vinculados: Vec<String> = norm
            .and_then(|n| n.inf_cte_sub.as_ref())
            .map_or_else(Vec::new, InfCteSub::get_nfes);
        nfes_vinculados.extend(ide_nfes);

        let mut info_cte = InfoCte {
            versao: inf.and_then(|i| i.versao.clone()),
            cte: ch_cte,
            doc_tipo: "CTeOS".to_string(),
            cancelado: None,
            tomador_codigo: inf
                .and_then(|i| i.ide.get_cod_tomador())
                .or_else(|| inf.and_then(|i| i.toma.as_ref().and(Some(4)))),
            numero_da_nota: inf.and_then(|i| i.ide.get_num_cte()),
            nat_operacao: inf.and_then(|i| i.ide.nat_operacao.get_not_empty()),
            uf_inicio: inf.and_then(|i| i.ide.ufini.get_not_empty()),
            mun_inicio: inf.and_then(|i| i.ide.x_mun_ini.get_not_empty()),
            uf_termino: inf.and_then(|i| i.ide.uffim.get_not_empty()),
            mun_termino: inf.and_then(|i| i.ide.x_mun_fim.get_not_empty()),
            local_entrega_nome: compl.and_then(Compl::get_local_entrega_nome),
            obs_gerais: compl.and_then(Compl::get_obs_gerais),
            cfop: inf.and_then(|i| i.ide.get_cfop()),
            data_emissao: inf.and_then(|i| i.ide.get_dt_emissao()),
            cte_complementar,
            nfes_vinculados,
            valor_total: inf.and_then(|i| i.v_prest.as_ref()).map(|v| v.v_tprest),
            ..Default::default()
        };

        if let Some(i) = inf {
            info_cte.fill_emitente(i.emitente.as_ref());
            info_cte.fill_tomador(&i.ide, i.toma.as_ref());
            info_cte.fill_imposto(i.imposto.as_ref());
        }

        info_cte.corrigir_codigo_do_tomador();
        info_cte.get_unique_elements();
        info_cte
    }
}

/// Conteúdo `<infCte>` específico do CT-e OS.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CteOsInfCte {
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    #[serde(rename = "emit", default)]
    pub emitente: Option<Agente>,
    #[serde(rename = "toma", default)]
    pub toma: Option<Agente>,
    #[serde(rename = "compl", default)]
    pub compl: Option<Compl>,
    #[serde(rename = "vPrest", default)]
    pub v_prest: Option<VPrest>,
    #[serde(rename = "imp", default)]
    pub imposto: Option<Imposto>,
    #[serde(rename = "infCTeNorm", default)]
    pub inf_cte_norm: Option<InfCteOsNorm>,
    #[serde(rename = "infCteAnu", default)]
    pub inf_cte_anu: Option<InfCteAnu>,
    #[serde(rename = "infCteComp", default)]
    pub inf_cte_comp: Option<Vec<InfCteComp>>,
    #[serde(rename = "autXML", default)]
    pub aut_xml: Option<Vec<AutXML>>,
    #[serde(rename = "infRespTec", default)]
    pub inf_resp_tec: Option<InfRespTec>,
    #[serde(rename = "infSolicNFF", default)]
    pub inf_solic_nff: Option<InfSolicNff>,
    #[serde(rename = "infPAA", default)]
    pub inf_paa: Option<InfPaa>,
    #[serde(rename = "pgtoVinc", default)]
    pub pgto_vinc: Option<PgtoVinc>,

    #[serde(rename = "ide")]
    pub ide: Ide,
}

/// Grupo de informações normais do CT-e OS (`<infCTeNorm>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfCteOsNorm {
    #[serde(rename = "infServico", default)]
    pub inf_servico: Option<InfServicoOs>,
    #[serde(rename = "infDocRef", default)]
    pub inf_doc_ref: Option<Vec<InfDocRefOs>>,
    #[serde(rename = "seg", default)]
    pub seg: Option<Vec<SegOs>>,
    #[serde(rename = "infModal", default)]
    pub inf_modal: Option<InfModal>,
    #[serde(rename = "infCteSub", default)]
    pub inf_cte_sub: Option<InfCteSub>,
    #[serde(rename = "refCTeCanc", default)]
    pub ref_cte_canc: Option<String>,
    #[serde(rename = "cobr", default)]
    pub cobr: Option<Cobranca>,
    #[serde(rename = "infGTVe", default)]
    pub inf_gtve: Option<Vec<InfGtveOs>>,
}

/// Informações da prestação de serviço no CT-e OS (`<infServico>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfServicoOs {
    #[serde(rename = "xDescServ", default)]
    pub x_desc_serv: Option<String>,
    #[serde(rename = "infQ", default)]
    pub inf_q: Option<InfQ>,
}

/// Documentos referenciados no CT-e OS (`<infDocRef>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfDocRefOs {
    #[serde(rename = "nDoc", default)]
    pub n_doc: Option<String>,
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,
    #[serde(rename = "subserie", default)]
    pub subserie: Option<String>,
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,
    #[serde(rename = "vDoc", default)]
    pub v_doc: Option<String>,
}

/// GTV-e relacionadas ao CT-e OS (`<infGTVe>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfGtveOs {
    #[serde(rename = "chCTe", default)]
    pub ch_cte: Option<String>,
    #[serde(rename = "Comp", default)]
    pub comp: Option<Vec<Comp>>,
}

/// Seguro da carga no CT-e OS (`<seg>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SegOs {
    #[serde(rename = "respSeg", default)]
    pub resp_seg: Option<String>,
    #[serde(rename = "xSeg", default)]
    pub x_seg: Option<String>,
    #[serde(rename = "nApol", default)]
    pub n_apol: Option<String>,
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
#[cfg(test)]
mod tests_gtve_and_cte_simp {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn test_parse_gtve_e_cte_simp() {
        let gtve_xml = r#"<GTVeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte">
            <GTVe>
                <infCte Id="CTe35260111222333000144640010000001001000001001" versao="4.00">
                    <ide>
                        <CFOP>5353</CFOP>
                        <natOp>TRANSPORTE DE VALORES</natOp>
                        <mod>64</mod>
                        <nCT>100</nCT>
                        <dhEmi>2026-01-10T10:00:00-03:00</dhEmi>
                        <UFIni>SP</UFIni>
                        <xMunIni>SAO PAULO</xMunIni>
                        <UFFim>SP</UFFim>
                        <xMunFim>CAMPINAS</xMunFim>
                        <toma>0</toma>
                    </ide>
                    <emit><CNPJ>11222333000144</CNPJ><xNome>EMPRESA VALORES LTDA</xNome></emit>
                    <rem><CNPJ>47960950000121</CNPJ><xNome>MAGAZINE LUIZA S/A</xNome></rem>
                    <detGTV>
                        <infEspecie><tpEspecie>1</tpEspecie><vEspecie>1500.50</vEspecie></infEspecie>
                        <infEspecie><tpEspecie>3</tpEspecie><vEspecie>499.50</vEspecie></infEspecie>
                    </detGTV>
                </infCte>
            </GTVe>
            <protCTe versao="4.00"><infProt><chCTe>35260111222333000144640010000001001000001001</chCTe></infProt></protCTe>
        </GTVeProc>"#;

        let proc_gtve: GtveProc = from_str(gtve_xml).unwrap();
        let info_gtve = proc_gtve
            .gtve
            .get_info(proc_gtve.prot_cte.inf_prot.ch_cte.get_key());
        assert_eq!(info_gtve.doc_tipo, "GTVe");
        assert_eq!(info_gtve.numero_da_nota, Some(100));
        assert_eq!(info_gtve.valor_total, Some(2000.0));
        assert_eq!(info_gtve.mun_inicio.as_deref(), Some("SAO PAULO"));

        let simp_xml = r#"<cteSimpProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte">
            <CTeSimp>
                <infCte Id="CTe35250199888777000166570010000002001000002002" versao="4.00">
                    <ide>
                        <CFOP>5352</CFOP>
                        <natOp>PRESTACAO DE SERVICO SIMPLIFICADO</natOp>
                        <mod>57</mod>
                        <nCT>200</nCT>
                        <dhEmi>2025-05-15T14:00:00-03:00</dhEmi>
                        <UFIni>SP</UFIni>
                        <UFFim>SP</UFFim>
                    </ide>
                    <emit><CNPJ>99888777000166</CNPJ><xNome>TRANS SIMP LTDA</xNome></emit>
                    <toma><CNPJ>47964911000100</CNPJ><xNome>USINA JUSSARA SA</xNome></toma>
                    <det nItem="1">
                        <xMunIni>FRANCA</xMunIni>
                        <xMunFim>RIBEIRAO PRETO</xMunFim>
                        <vPrest>350.00</vPrest>
                        <infNFe><chave>35250547964911000100550010000000011000000011</chave></infNFe>
                    </det>
                    <total><vTPrest>350.00</vTPrest><vTRec>350.00</vTRec></total>
                </infCte>
            </CTeSimp>
            <protCTe versao="4.00"><infProt><chCTe>35250199888777000166570010000002001000002002</chCTe></infProt></protCTe>
        </cteSimpProc>"#;

        let proc_simp: CteSimpProc = from_str(simp_xml).unwrap();
        let info_simp = proc_simp
            .cte_simp
            .get_info(proc_simp.prot_cte.inf_prot.ch_cte.get_key());
        assert_eq!(info_simp.doc_tipo, "CTeSimp");
        assert_eq!(info_simp.numero_da_nota, Some(200));
        assert_eq!(info_simp.valor_total, Some(350.0));
        assert_eq!(info_simp.mun_inicio.as_deref(), Some("FRANCA"));
        assert_eq!(info_simp.mun_termino.as_deref(), Some("RIBEIRAO PRETO"));
        assert_eq!(info_simp.nfes_vinculados.len(), 1);
    }

    #[test]
    fn test_parse_cte_os() {
        let xml = r#"<cteOSProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte">
            <CTeOS>
                <infCte Id="CTe35260111222333000144670010000003001000003001" versao="4.00">
                    <ide>
                        <CFOP>5357</CFOP>
                        <natOp>PRESTACAO DE SERVICO DE TRANSPORTE DE PESSOAS</natOp>
                        <mod>67</mod>
                        <nCT>300</nCT>
                        <dhEmi>2026-01-15T08:00:00-03:00</dhEmi>
                        <UFIni>SP</UFIni>
                        <xMunIni>SAO PAULO</xMunIni>
                        <UFFim>RJ</UFFim>
                        <xMunFim>RIO DE JANEIRO</xMunFim>
                    </ide>
                    <emit><CNPJ>11222333000144</CNPJ><xNome>VIACAO MODELO LTDA</xNome><CRT>3</CRT></emit>
                    <toma><CNPJ>47960950000121</CNPJ><xNome>CLIENTE MODELO SA</xNome></toma>
                    <vPrest><vTPrest>1200.00</vTPrest><vRec>1200.00</vRec></vPrest>
                    <imp><ICMS><ICMS00><CST>00</CST><vBC>1200.00</vBC><pICMS>12.00</pICMS><vICMS>144.00</vICMS></ICMS00></ICMS></imp>
                    <infCTeNorm><infServico><xDescServ>TRANSPORTE DE PASSAGEIROS</xDescServ></infServico></infCTeNorm>
                    <pgtoVinc><pgto nPag="1"><idTransacao>TX-1</idTransacao><tpMeioPgto>17</tpMeioPgto><CNPJReceb>11222333000144</CNPJReceb><CNPJBasePSP>12345678</CNPJBasePSP></pgto></pgtoVinc>
                </infCte>
            </CTeOS>
            <protCTe versao="4.00"><infProt><chCTe>35260111222333000144670010000003001000003001</chCTe></infProt></protCTe>
        </cteOSProc>"#;

        let proc_os: CteOsProc = from_str(xml).unwrap();
        let info_os = proc_os
            .cte_os
            .get_info(proc_os.prot_cte.inf_prot.ch_cte.get_key());
        assert_eq!(info_os.doc_tipo, "CTeOS");
        assert_eq!(info_os.numero_da_nota, Some(300));
        assert_eq!(info_os.valor_total, Some(1200.0));
        assert_eq!(info_os.v_icms, Some(144.0));
        assert_eq!(info_os.tomador_codigo, Some(4));
    }

    /// CT-e Simplificado com `<infNFe>` sem `<chave>` (emissor não conforme):
    /// deve parsear sem erro; o alias `chNFe` preserva o vínculo existente.
    ///
    /// `cargo test -- --show-output test_cte_simp_inf_nfe_sem_chave`
    #[test]
    fn test_cte_simp_inf_nfe_sem_chave() {
        let xml = r#"<cteSimpProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte"><CTeSimp><infCte Id="CTe35250699888777000166570010000003001000003003" versao="4.00"><ide><CFOP>5352</CFOP><nCT>300</nCT><dhEmi>2025-06-10T09:00:00-03:00</dhEmi></ide><emit><CNPJ>99888777000166</CNPJ><xNome>TRANS SIMP LTDA</xNome></emit><det nItem="1"><xMunIni>FRANCA</xMunIni><xMunFim>RIBEIRAO PRETO</xMunFim><vPrest>100.00</vPrest><infNFe><dPrev>2025-06-12</dPrev></infNFe><infNFe><chNFe>35250547964911000100550010000000011000000011</chNFe></infNFe></det><total><vTPrest>100.00</vTPrest></total></infCte></CTeSimp><protCTe versao="4.00"><infProt><chCTe>35250699888777000166570010000003001000003003</chCTe></infProt></protCTe></cteSimpProc>"#;

        let proc: CteSimpProc = from_str(xml).expect("infNFe sem chave nao deve quebrar o parse");
        let info = proc
            .cte_simp
            .get_info(proc.prot_cte.inf_prot.ch_cte.get_key());

        assert_eq!(info.nfes_vinculados.len(), 1);
        assert_eq!(
            info.nfes_vinculados[0],
            "35250547964911000100550010000000011000000011"
        );
    }
}
