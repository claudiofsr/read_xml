//! # Transformação para o Layout Padrão da Receita Federal (56 Colunas)
//!
//! Converte documentos [`InfoCte`] e [`InfoNfe`] para o esquema tabular unificado
//! utilizado nos extratos da Receita Federal (`2026-CTe-*.csv`, `2026-NFe-*.csv`),
//! com acesso direto tipado aos campos (sem reflexão Serde), formatação numérica
//! brasileira, normalização de chaves/NCM, geração da linha sintética `Item 0` para
//! NF-es multi-item, cálculo de valor proporcional e tabelas fiscais completas
//! (CONFAZ CFOP, CST PIS/COFINS e Natureza da Base de Cálculo).

use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use struct_iterable::Iterable;

use crate::{
    InfoExtension, lookup_cfop_desc, lookup_ncm_desc,
    xml_structs::{cte::InfoCte, nfe::InfoNfe},
};

const ND: &str = "<N/D>";
const DIVERSOS: &str = "*DIVERSOS*";

/// Registro unificado no formato padrão da Receita Federal (56 colunas).
///
/// Compatível simultaneamente com exportação CSV (`CsvWriter`) e Excel (`write_xlsx`).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Iterable)]
pub struct InfoReceita {
    #[serde(rename = "CNPJ do Contribuinte : NF Item (Todos)")]
    pub cnpj_contribuinte: String,

    #[serde(rename = "Nome do Contribuinte : NF Item (Todos)")]
    pub nome_contribuinte: String,

    #[serde(rename = "Entrada/Saída : NF (Todos)")]
    pub entrada_saida: String,

    #[serde(rename = "CPF/CNPJ do Participante : NF (Todos)")]
    pub cpf_cnpj_participante: String,

    #[serde(rename = "Nome do Participante : NF (Todos)")]
    pub nome_participante: String,

    #[serde(rename = "CRT : NF (Todos)")]
    pub crt: String,

    #[serde(rename = "Observações : NF (Todos)")]
    pub observacoes: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_rem_cnpj_comp: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub cte_rem_cnpj_nfe: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: Nome de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub cte_rem_nome_nfe: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: Município de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub cte_rem_mun_nfe: String,

    #[serde(
        rename = "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_papel_tomador_comp: String,

    #[serde(
        rename = "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub cte_papel_tomador_nfe: String,

    #[serde(
        rename = "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_outro_tomador_comp: String,

    #[serde(
        rename = "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub cte_outro_tomador_nfe: String,

    #[serde(
        rename = "CTe - UF do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_uf_inicio: String,

    #[serde(
        rename = "CTe - Nome do Município do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_mun_inicio: String,

    #[serde(
        rename = "CTe - UF do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_uf_termino: String,

    #[serde(
        rename = "CTe - Nome do Município do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_mun_termino: String,

    #[serde(
        rename = "CTe - Informações do Destinatário do CT-e: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_dest_cnpj: String,

    #[serde(
        rename = "CTe - Informações do Destinatário do CT-e: Nome de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_dest_nome: String,

    #[serde(
        rename = "CTe - Local de Entrega constante na Nota Fiscal: Nome de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub cte_local_entrega_nome: String,

    #[serde(rename = "Descrição da Natureza da Operação : NF Item (Todos)")]
    pub desc_natureza_operacao: String,

    #[serde(rename = "Cancelada : NF (Todos)")]
    pub cancelada: String,

    #[serde(rename = "Registro de Origem do Item : NF Item (Todos)")]
    pub registro_origem_item: String,

    #[serde(rename = "Natureza da Base de Cálculo do Crédito Descrição : NF Item (Todos)")]
    pub nat_bc_credito_desc: String,

    #[serde(rename = "Modelo - Descrição : NF Item (Todos)")]
    pub modelo_descricao: String,

    #[serde(rename = "Número da Nota : NF Item (Todos)")]
    pub numero_nota: String,

    #[serde(rename = "Chave da Nota Fiscal Eletrônica : NF Item (Todos)")]
    pub chave_nfe: String,

    #[serde(rename = "Inf. NFe - Chave de acesso da NF-e : ConhecimentoInformacaoNFe")]
    pub inf_nfe_chave: String,

    #[serde(rename = "CTe - Observações Gerais de Conhecimento : ConhecimentoInformacaoNFe")]
    pub cte_obs_gerais: String,

    #[serde(rename = "Dia da Emissão : NF Item (Todos)")]
    pub dia_emissao: String,

    #[serde(rename = "Número da DI : NF Item (Todos)")]
    pub numero_di: String,

    #[serde(rename = "Número do Item : NF Item (Todos)")]
    pub numero_item: String,

    #[serde(rename = "Código CFOP : NF Item (Todos)")]
    pub codigo_cfop: String,

    #[serde(rename = "Descrição CFOP : NF Item (Todos)")]
    pub descricao_cfop: String,

    #[serde(rename = "Descrição da Mercadoria/Serviço : NF Item (Todos)")]
    pub descricao_mercadoria: String,

    #[serde(rename = "Código NCM : NF Item (Todos)")]
    pub codigo_ncm: String,

    #[serde(rename = "Descrição NCM : NF Item (Todos)")]
    pub descricao_ncm: String,

    #[serde(rename = "COFINS: Alíquota ad valorem - Atributo : NF Item (Todos)")]
    pub cofins_aliq: String,

    #[serde(rename = "PIS: Alíquota ad valorem - Atributo : NF Item (Todos)")]
    pub pis_aliq: String,

    #[serde(rename = "CST COFINS Descrição : NF Item (Todos)")]
    pub cst_cofins_desc: String,

    #[serde(rename = "CST PIS Descrição : NF Item (Todos)")]
    pub cst_pis_desc: String,

    #[serde(rename = "Valor Total : NF (Todos) SOMA")]
    pub valor_total: String,

    #[serde(rename = "Valor da Nota Proporcional : NF Item (Todos) SOMA")]
    pub valor_nota_proporcional: String,

    #[serde(rename = "Valor dos Descontos : NF Item (Todos) SOMA")]
    pub valor_descontos: String,

    #[serde(rename = "Valor Seguro : NF (Todos) SOMA")]
    pub valor_seguro: String,

    #[serde(rename = "COFINS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub cofins_valor: String,

    #[serde(rename = "PIS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub pis_valor: String,

    #[serde(rename = "IPI: Valor do Tributo : NF Item (Todos) SOMA")]
    pub ipi_valor: String,

    #[serde(rename = "ISS: Base de Cálculo : NF Item (Todos) SOMA")]
    pub iss_bc: String,

    #[serde(rename = "ISS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub iss_valor: String,

    #[serde(rename = "ICMS: Alíquota : NF Item (Todos) NOISE OR")]
    pub icms_aliquota: String,

    #[serde(rename = "ICMS: Base de Cálculo : NF Item (Todos) SOMA")]
    pub icms_bc: String,

    #[serde(rename = "ICMS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub icms_valor: String,

    #[serde(rename = "ICMS por Substituição: Valor do Tributo : NF Item (Todos) SOMA")]
    pub icms_st_valor: String,
}

impl InfoExtension for InfoReceita {}

crate::impl_sanitize_text!(
    InfoReceita,
    cnpj_contribuinte,
    nome_contribuinte,
    entrada_saida,
    cpf_cnpj_participante,
    nome_participante,
    crt,
    observacoes,
    cte_rem_cnpj_comp,
    cte_rem_cnpj_nfe,
    cte_rem_nome_nfe,
    cte_rem_mun_nfe,
    cte_papel_tomador_comp,
    cte_papel_tomador_nfe,
    cte_outro_tomador_comp,
    cte_outro_tomador_nfe,
    cte_uf_inicio,
    cte_mun_inicio,
    cte_uf_termino,
    cte_mun_termino,
    cte_dest_cnpj,
    cte_dest_nome,
    cte_local_entrega_nome,
    desc_natureza_operacao,
    cancelada,
    registro_origem_item,
    nat_bc_credito_desc,
    modelo_descricao,
    numero_nota,
    chave_nfe,
    inf_nfe_chave,
    cte_obs_gerais,
    dia_emissao,
    numero_di,
    numero_item,
    codigo_cfop,
    descricao_cfop,
    descricao_mercadoria,
    codigo_ncm,
    descricao_ncm,
    cofins_aliq,
    pis_aliq,
    cst_cofins_desc,
    cst_pis_desc,
    valor_total,
    valor_nota_proporcional,
    valor_descontos,
    valor_seguro,
    cofins_valor,
    pis_valor,
    ipi_valor,
    iss_bc,
    iss_valor,
    icms_aliquota,
    icms_bc,
    icms_valor,
    icms_st_valor,
);

// =============================================================================
// CONVERSÃO PÚBLICA EM PARALELO, CANONIZAÇÃO E ORDENAÇÃO
// =============================================================================

/// Converte uma fatia de [`InfoCte`] para o formato unificado da Receita Federal,
/// canonizando os nomes de contribuintes e participantes por CNPJ/CPF e ordenando
/// de forma determinística idêntica ao layout da Receita Federal (`2026-CTe-*.csv`).
pub fn ctes_to_receita(ctes: &[InfoCte]) -> Vec<InfoReceita> {
    let mut rows: Vec<InfoReceita> = ctes.par_iter().map(convert_cte).collect();
    canonicalize_cte_names(&mut rows);
    sort_receita_rows(&mut rows);
    rows
}

/// Converte uma fatia de [`InfoNfe`] (preferencialmente ordenada por chave e item)
/// para o formato unificado da Receita Federal, gerando a linha sintética `Item 0`
/// sempre que uma NF-e possuir mais de um item e ordenando no padrão da Receita Federal.
pub fn nfes_to_receita(nfes: &[InfoNfe]) -> Vec<InfoReceita> {
    let chunks: Vec<&[InfoNfe]> = nfes
        .chunk_by(|a, b| a.nfe.is_some() && a.nfe == b.nfe)
        .collect();

    let mut rows: Vec<InfoReceita> = chunks
        .par_iter()
        .flat_map_iter(|chunk| convert_nfe_chunk(chunk))
        .collect();

    sort_receita_rows(&mut rows);
    rows
}

/// Registra um voto único por documento (`chave`) para associação `(documento -> nome)`.
fn record_name_vote<'a>(
    counts: &mut HashMap<&'a str, HashMap<&'a str, usize>>,
    seen: &mut HashSet<(&'a str, &'a str)>,
    chave: &'a str,
    doc: &'a str,
    name: &'a str,
) {
    if doc != ND && name != ND && seen.insert((chave, doc)) {
        *counts.entry(doc).or_default().entry(name).or_insert(0) += 1;
    }
}

/// Resolve o nome canônico mais frequente por CNPJ/CPF (desempate por maior comprimento, depois lexicográfico).
fn resolve_canonical_map(counts: HashMap<&str, HashMap<&str, usize>>) -> HashMap<String, String> {
    counts
        .into_iter()
        .filter_map(|(doc, name_map)| {
            let best = name_map
                .into_iter()
                .max_by(|(n1, c1), (n2, c2)| {
                    c1.cmp(c2)
                        .then_with(|| n1.len().cmp(&n2.len()))
                        .then_with(|| n2.cmp(n1))
                })?
                .0;
            Some((doc.to_string(), best.to_string()))
        })
        .collect()
}

/// Unifica `nome_contribuinte` e `nome_participante` em CT-es por CNPJ/CPF,
/// replicando o cadastro de contribuintes/participantes da Receita Federal enquanto
/// preserva os nomes originais do XML nas colunas de conhecimento (`ConhecimentoInformacaoNFe` / `Destinatário`).
fn canonicalize_cte_names(rows: &mut [InfoReceita]) {
    let mut contrib_counts: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    let mut part_counts: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    let mut seen_contribs = HashSet::new();
    let mut seen_parts = HashSet::new();

    for r in rows.iter() {
        record_name_vote(
            &mut contrib_counts,
            &mut seen_contribs,
            &r.chave_nfe,
            &r.cnpj_contribuinte,
            &r.nome_contribuinte,
        );
        record_name_vote(
            &mut part_counts,
            &mut seen_parts,
            &r.chave_nfe,
            &r.cpf_cnpj_participante,
            &r.nome_participante,
        );
    }

    let contrib_map = resolve_canonical_map(contrib_counts);
    let part_map = resolve_canonical_map(part_counts);

    for r in rows.iter_mut() {
        if let Some(canonical) = contrib_map.get(&r.cnpj_contribuinte) {
            r.nome_contribuinte = canonical.clone();
        }
        if let Some(canonical) = part_map.get(&r.cpf_cnpj_participante) {
            r.nome_participante = canonical.clone();
        }
    }
}

/// Ordena registros [`InfoReceita`] da esquerda para a direita com `<N/D>` tratado como
/// nulo (`NULLS FIRST`) e ordenação numérica de nota/item, mantendo `Item 0..N` contíguos.
fn sort_receita_rows(rows: &mut [InfoReceita]) {
    #[inline]
    fn norm(s: &str) -> &str {
        if s == ND { "" } else { s }
    }
    #[inline]
    fn nota_num(s: &str) -> u64 {
        s.parse::<u64>().unwrap_or(0)
    }
    #[inline]
    fn item_num(s: &str) -> u32 {
        s.parse::<u32>().unwrap_or(0)
    }

    rows.par_sort_by(|a, b| {
        norm(&a.cnpj_contribuinte)
            .cmp(norm(&b.cnpj_contribuinte))
            .then_with(|| norm(&a.nome_contribuinte).cmp(norm(&b.nome_contribuinte)))
            .then_with(|| norm(&a.entrada_saida).cmp(norm(&b.entrada_saida)))
            .then_with(|| norm(&a.cpf_cnpj_participante).cmp(norm(&b.cpf_cnpj_participante)))
            .then_with(|| norm(&a.nome_participante).cmp(norm(&b.nome_participante)))
            .then_with(|| norm(&a.crt).cmp(norm(&b.crt)))
            .then_with(|| norm(&a.observacoes).cmp(norm(&b.observacoes)))
            .then_with(|| norm(&a.cte_rem_cnpj_comp).cmp(norm(&b.cte_rem_cnpj_comp)))
            .then_with(|| norm(&a.cte_rem_cnpj_nfe).cmp(norm(&b.cte_rem_cnpj_nfe)))
            .then_with(|| norm(&a.cte_rem_nome_nfe).cmp(norm(&b.cte_rem_nome_nfe)))
            .then_with(|| norm(&a.cte_rem_mun_nfe).cmp(norm(&b.cte_rem_mun_nfe)))
            .then_with(|| norm(&a.cte_papel_tomador_comp).cmp(norm(&b.cte_papel_tomador_comp)))
            .then_with(|| norm(&a.cte_papel_tomador_nfe).cmp(norm(&b.cte_papel_tomador_nfe)))
            .then_with(|| norm(&a.cte_outro_tomador_comp).cmp(norm(&b.cte_outro_tomador_comp)))
            .then_with(|| norm(&a.cte_outro_tomador_nfe).cmp(norm(&b.cte_outro_tomador_nfe)))
            .then_with(|| norm(&a.cte_uf_inicio).cmp(norm(&b.cte_uf_inicio)))
            .then_with(|| norm(&a.cte_mun_inicio).cmp(norm(&b.cte_mun_inicio)))
            .then_with(|| norm(&a.cte_uf_termino).cmp(norm(&b.cte_uf_termino)))
            .then_with(|| norm(&a.cte_mun_termino).cmp(norm(&b.cte_mun_termino)))
            .then_with(|| norm(&a.cte_dest_cnpj).cmp(norm(&b.cte_dest_cnpj)))
            .then_with(|| norm(&a.cte_dest_nome).cmp(norm(&b.cte_dest_nome)))
            .then_with(|| norm(&a.cte_local_entrega_nome).cmp(norm(&b.cte_local_entrega_nome)))
            .then_with(|| norm(&a.desc_natureza_operacao).cmp(norm(&b.desc_natureza_operacao)))
            .then_with(|| norm(&a.cancelada).cmp(norm(&b.cancelada)))
            .then_with(|| norm(&a.registro_origem_item).cmp(norm(&b.registro_origem_item)))
            .then_with(|| norm(&a.modelo_descricao).cmp(norm(&b.modelo_descricao)))
            .then_with(|| nota_num(&a.numero_nota).cmp(&nota_num(&b.numero_nota)))
            .then_with(|| norm(&a.chave_nfe).cmp(norm(&b.chave_nfe)))
            .then_with(|| item_num(&a.numero_item).cmp(&item_num(&b.numero_item)))
    });
}

// =============================================================================
// LÓGICA DE TRANSFORMAÇÃO: CT-e
// =============================================================================

fn convert_cte(cte: &InfoCte) -> InfoReceita {
    let papel = match cte.tomador_codigo {
        Some(0) => "Remetente",
        Some(1) => "Expedidor",
        Some(2) => "Recebedor",
        Some(3) => "Destinatário",
        Some(4) => "Outros",
        _ => ND,
    }
    .to_string();

    let (part_doc, part_nome) = match cte.tomador_codigo {
        Some(0) => doc_and_name(&cte.remetente_cnpj, &cte.remetente_cpf, &cte.remetente_nome),
        Some(1) => doc_and_name(&cte.expedidor_cnpj, &cte.expedidor_cpf, &cte.expedidor_nome),
        Some(2) => doc_and_name(&cte.recebedor_cnpj, &cte.recebedor_cpf, &cte.recebedor_nome),
        Some(3) => doc_and_name(
            &cte.destinatario_cnpj,
            &cte.destinatario_cpf,
            &cte.destinatario_nome,
        ),
        _ => doc_and_name(&cte.tomador_cnpj, &cte.tomador_cpf, &cte.tomador_nome),
    };

    let outro_tomador = if cte.tomador_codigo == Some(4) {
        first_non_empty_opt(&[cte.tomador_cnpj.as_deref(), cte.tomador_cpf.as_deref()])
    } else {
        ND.to_string()
    };

    let (rem_doc, rem_nome) =
        doc_and_name(&cte.remetente_cnpj, &cte.remetente_cpf, &cte.remetente_nome);
    let rem_mun = opt_str_or_nd(cte.remetente_ender_municipio.as_deref());

    let (dest_doc, dest_nome) = doc_and_name(
        &cte.destinatario_cnpj,
        &cte.destinatario_cpf,
        &cte.destinatario_nome,
    );

    let uf_inicio = first_non_empty_opt(&[
        cte.uf_inicio.as_deref(),
        cte.expedidor_ender_estado.as_deref(),
        cte.remetente_ender_estado.as_deref(),
    ]);
    let mun_inicio = first_non_empty_opt(&[
        cte.mun_inicio.as_deref(),
        cte.expedidor_ender_municipio.as_deref(),
        cte.remetente_ender_municipio.as_deref(),
    ]);
    let uf_termino = first_non_empty_opt(&[
        cte.uf_termino.as_deref(),
        cte.recebedor_ender_estado.as_deref(),
        cte.destinatario_ender_estado.as_deref(),
    ]);
    let mun_termino = first_non_empty_opt(&[
        cte.mun_termino.as_deref(),
        cte.recebedor_ender_municipio.as_deref(),
        cte.destinatario_ender_municipio.as_deref(),
    ]);

    let cfop = opt_display_or_nd(cte.cfop);
    let desc_cfop = lookup_cfop_desc(&cfop).to_string();
    let is_entrada = cfop_is_entrada(&cfop);

    let inf_nfe_chave = format_cte_linked_nfes(cte);
    let cte_obs_gerais = opt_str_or_nd(cte.obs_gerais.as_deref());

    let valor_total_str = format_opt_br(cte.valor_total);
    let aliq_icms = cte.aliq_icms.unwrap_or(0.0) / 100.0;

    InfoReceita {
        cnpj_contribuinte: first_non_empty_opt(&[
            cte.emitente_cnpj.as_deref(),
            cte.emitente_cpf.as_deref(),
        ]),
        nome_contribuinte: opt_str_or_nd(cte.emitente_nome.as_deref()),
        entrada_saida: if is_entrada { "Entrada" } else { "Saída" }.to_string(),
        cpf_cnpj_participante: part_doc,
        nome_participante: part_nome,
        crt: opt_display_or_nd(cte.emitente_crt),
        observacoes: ND.to_string(),
        cte_rem_cnpj_comp: rem_doc.clone(),
        cte_rem_cnpj_nfe: rem_doc,
        cte_rem_nome_nfe: rem_nome,
        cte_rem_mun_nfe: rem_mun,
        cte_papel_tomador_comp: papel.clone(),
        cte_papel_tomador_nfe: papel,
        cte_outro_tomador_comp: outro_tomador.clone(),
        cte_outro_tomador_nfe: outro_tomador,
        cte_uf_inicio: uf_inicio,
        cte_mun_inicio: mun_inicio,
        cte_uf_termino: uf_termino,
        cte_mun_termino: mun_termino,
        cte_dest_cnpj: dest_doc,
        cte_dest_nome: dest_nome,
        cte_local_entrega_nome: opt_str_or_nd(cte.local_entrega_nome.as_deref()),
        desc_natureza_operacao: opt_str_or_nd(cte.nat_operacao.as_deref()),
        cancelada: format_cancelada(cte.cancelado.as_deref()),
        registro_origem_item: cte.doc_tipo.clone(),
        nat_bc_credito_desc: ND.to_string(),
        modelo_descricao: match cte.doc_tipo.as_str() {
            "GTVe" => "Guia de Transporte de Valores Eletrônica GTV-e",
            "CTeSimp" => "Conhecimento de Transporte Eletrônico Simplificado CT-e",
            _ => "Conhecimento de Transporte Eletrônico CT-e",
        }
        .to_string(),
        numero_nota: opt_display_or_nd(cte.numero_da_nota),
        chave_nfe: opt_str_or_nd(cte.cte.as_deref()),
        inf_nfe_chave,
        cte_obs_gerais,
        dia_emissao: format_date_br(cte.data_emissao),
        numero_di: ND.to_string(),
        numero_item: "1".to_string(),
        codigo_cfop: cfop,
        descricao_cfop: desc_cfop,
        descricao_mercadoria: ND.to_string(),
        codigo_ncm: "00000000".to_string(),
        descricao_ncm: ND.to_string(),
        cofins_aliq: format_opt_br(cte.aliq_cofins),
        pis_aliq: format_opt_br(cte.aliq_pis),
        cst_cofins_desc: ND.to_string(),
        cst_pis_desc: ND.to_string(),
        valor_total: valor_total_str.clone(),
        valor_nota_proporcional: valor_total_str,
        valor_descontos: "0,00".to_string(),
        valor_seguro: "0,00".to_string(),
        cofins_valor: format_opt_br(cte.v_cofins),
        pis_valor: format_opt_br(cte.v_pis),
        ipi_valor: format_opt_br(cte.v_ipi),
        iss_bc: format_opt_br(cte.v_bc_iss),
        iss_valor: format_opt_br(cte.v_iss),
        icms_aliquota: format_br_float(aliq_icms, 2),
        icms_bc: format_opt_br(cte.v_bc_icms),
        icms_valor: format_opt_br(cte.v_icms),
        icms_st_valor: format_opt_br(cte.v_icms_st),
    }
}

fn format_cte_linked_nfes(cte: &InfoCte) -> String {
    let keys = cte.get_correlated_nfes();
    match keys.len() {
        0 => ND.to_string(),
        1 => keys[0].trim().to_string(),
        _ => DIVERSOS.to_string(),
    }
}

// =============================================================================
// LÓGICA DE TRANSFORMAÇÃO: NF-e (Item 0 + Itens 1..N)
// =============================================================================

struct ParsedNfeItem {
    numero_item: String,
    numero_di: String,
    cfop: String,
    desc_cfop: String,
    nat_bc_credito_desc: String,
    descricao: String,
    ncm: String,
    desc_ncm: String,
    cofins_aliq: String,
    pis_aliq: String,
    cst_cofins_desc: String,
    cst_pis_desc: String,
    valor_total_nfe: f64,
    valor_proporcional: f64,
    valor_desconto: f64,
    valor_seguro: f64,
    valor_cofins: f64,
    valor_pis: f64,
    valor_ipi: f64,
    bc_iss: f64,
    valor_iss: f64,
    aliq_icms: f64,
    bc_icms: f64,
    valor_icms: f64,
    valor_icms_st: f64,
}

fn convert_nfe_chunk(chunk: &[InfoNfe]) -> Vec<InfoReceita> {
    let Some(first_nfe) = chunk.first() else {
        return Vec::new();
    };

    let cnpj_contribuinte = first_non_empty_opt(&[
        first_nfe.emitente_cnpj.as_deref(),
        first_nfe.emitente_cpf.as_deref(),
    ]);
    let nome_contribuinte = opt_str_or_nd(first_nfe.emitente_nome.as_deref());
    let cpf_cnpj_participante = first_non_empty_opt(&[
        first_nfe.destinatario_cnpj.as_deref(),
        first_nfe.destinatario_cpf.as_deref(),
    ]);
    let nome_participante = opt_str_or_nd(first_nfe.destinatario_nome.as_deref());
    let crt = opt_display_or_nd(first_nfe.emitente_crt);
    let observacoes = build_nfe_observacoes(
        first_nfe.info_adic_fisco.as_deref(),
        first_nfe.info_adic_contribuinte.as_deref(),
    );

    let first_cfop = opt_display_or_nd(first_nfe.cfop);
    let is_entrada = match first_nfe.tp_nf.as_deref().map(str::trim) {
        Some("0") => true,
        Some("1") => false,
        _ => cfop_is_entrada(&first_cfop),
    };
    let entrada_saida = if is_entrada { "Entrada" } else { "Saída" }.to_string();

    let desc_natureza_operacao = opt_str_or_nd(first_nfe.nat_operacao.as_deref());
    let cancelada = format_cancelada(first_nfe.cancelado.as_deref());
    let modelo_descricao = match first_nfe.modelo.as_deref().map(str::trim) {
        Some("65") => "Nota Fiscal de Consumidor Eletrônica",
        _ => "Nota Fiscal Eletrônica",
    }
    .to_string();
    let numero_nota = opt_display_or_nd(first_nfe.numero_da_nota);
    let chave_nfe = opt_str_or_nd(first_nfe.nfe.as_deref());
    let dia_emissao = format_date_br(first_nfe.data_emissao);

    let multi_item = chunk.len() > 1;

    let parsed_items: Vec<ParsedNfeItem> = chunk
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let cfop = opt_display_or_nd(item.cfop);
            let desc_cfop = lookup_cfop_desc(&cfop).to_string();

            let ncm_digits = item
                .ncm
                .as_deref()
                .map(|s| s.remove_non_digits())
                .unwrap_or_default();
            let ncm = if ncm_digits.is_empty() {
                "00000000".to_string()
            } else {
                ncm_digits
            };

            let cst_cofins = opt_display_or_nd(item.cst_cofins);
            let cst_pis = opt_display_or_nd(item.cst_pis);
            let nat_bc_credito_desc = lookup_natureza_bc_desc(&cfop, &cst_pis)
                .or_else(|| lookup_natureza_bc_desc(&cfop, &cst_cofins))
                .unwrap_or(ND)
                .to_string();

            let valor_total_nfe = item.valor_total_nfe.unwrap_or(0.0);
            let valor_total_itens = item.valor_total_itens.unwrap_or(0.0);
            let valor_item = item.v_prod.unwrap_or(0.0);

            let valor_proporcional = if !multi_item && valor_total_nfe > 0.0 {
                valor_total_nfe
            } else if valor_total_itens > 0.0 && valor_total_nfe > 0.0 {
                valor_item * (valor_total_nfe / valor_total_itens)
            } else if valor_item > 0.0 {
                valor_item
            } else {
                valor_total_nfe
            };

            let numero_item = item
                .n_item
                .map_or_else(|| (idx + 1).to_string(), |n| n.to_string());

            ParsedNfeItem {
                numero_item,
                numero_di: opt_display_or_nd(item.n_di),
                cfop,
                desc_cfop,
                nat_bc_credito_desc,
                descricao: opt_str_or_nd(item.descricao.as_deref()),
                desc_ncm: lookup_ncm_desc(&ncm).to_string(),
                ncm,
                cofins_aliq: format_opt_br(item.aliq_cofins),
                pis_aliq: format_opt_br(item.aliq_pis),
                cst_cofins_desc: lookup_cst_pis_cofins_desc(&cst_cofins, is_entrada).to_string(),
                cst_pis_desc: lookup_cst_pis_cofins_desc(&cst_pis, is_entrada).to_string(),
                valor_total_nfe,
                valor_proporcional,
                valor_desconto: item.v_desc.unwrap_or(0.0),
                valor_seguro: item.v_seg.unwrap_or(0.0),
                valor_cofins: item.v_cofins.unwrap_or(0.0),
                valor_pis: item.v_pis.unwrap_or(0.0),
                valor_ipi: item.v_ipi.unwrap_or(0.0),
                bc_iss: item.v_bc_iss.unwrap_or(0.0),
                valor_iss: item.v_iss.unwrap_or(0.0),
                aliq_icms: item.aliq_icms.unwrap_or(0.0) / 100.0,
                bc_icms: item.v_bc_icms.unwrap_or(0.0),
                valor_icms: item.v_icms.unwrap_or(0.0),
                valor_icms_st: item.v_icms_st.unwrap_or(0.0),
            }
        })
        .collect();

    let base_row = InfoReceita {
        cnpj_contribuinte,
        nome_contribuinte,
        entrada_saida,
        cpf_cnpj_participante,
        nome_participante,
        crt,
        observacoes,
        cte_rem_cnpj_comp: ND.to_string(),
        cte_rem_cnpj_nfe: ND.to_string(),
        cte_rem_nome_nfe: ND.to_string(),
        cte_rem_mun_nfe: ND.to_string(),
        cte_papel_tomador_comp: ND.to_string(),
        cte_papel_tomador_nfe: ND.to_string(),
        cte_outro_tomador_comp: ND.to_string(),
        cte_outro_tomador_nfe: ND.to_string(),
        cte_uf_inicio: ND.to_string(),
        cte_mun_inicio: ND.to_string(),
        cte_uf_termino: ND.to_string(),
        cte_mun_termino: ND.to_string(),
        cte_dest_cnpj: ND.to_string(),
        cte_dest_nome: ND.to_string(),
        cte_local_entrega_nome: ND.to_string(),
        desc_natureza_operacao,
        cancelada,
        registro_origem_item: "NFe".to_string(),
        nat_bc_credito_desc: ND.to_string(),
        modelo_descricao,
        numero_nota,
        chave_nfe,
        inf_nfe_chave: ND.to_string(),
        cte_obs_gerais: ND.to_string(),
        dia_emissao,
        numero_di: ND.to_string(),
        ..Default::default()
    };

    let mut out = Vec::with_capacity(parsed_items.len() + usize::from(multi_item));

    if multi_item {
        let total_nfe = parsed_items
            .iter()
            .map(|p| p.valor_total_nfe)
            .find(|&v| v > 0.0)
            .unwrap_or_else(|| parsed_items.iter().map(|p| p.valor_proporcional).sum());

        let ncm_col = collapse_or_diversos(parsed_items.iter().map(|p| p.ncm.as_str()));
        let desc_ncm_col = if ncm_col == DIVERSOS {
            DIVERSOS.to_string()
        } else {
            collapse_or_diversos(parsed_items.iter().map(|p| p.desc_ncm.as_str()))
        };

        out.push(InfoReceita {
            nat_bc_credito_desc: collapse_or_diversos(
                parsed_items.iter().map(|p| p.nat_bc_credito_desc.as_str()),
            ),
            numero_di: collapse_or_diversos(parsed_items.iter().map(|p| p.numero_di.as_str())),
            numero_item: "0".to_string(),
            codigo_cfop: collapse_or_diversos(parsed_items.iter().map(|p| p.cfop.as_str())),
            descricao_cfop: collapse_or_diversos(parsed_items.iter().map(|p| p.desc_cfop.as_str())),
            descricao_mercadoria: collapse_or_diversos(
                parsed_items.iter().map(|p| p.descricao.as_str()),
            ),
            codigo_ncm: ncm_col,
            descricao_ncm: desc_ncm_col,
            cofins_aliq: collapse_or_diversos(parsed_items.iter().map(|p| p.cofins_aliq.as_str())),
            pis_aliq: collapse_or_diversos(parsed_items.iter().map(|p| p.pis_aliq.as_str())),
            cst_cofins_desc: collapse_or_diversos(
                parsed_items.iter().map(|p| p.cst_cofins_desc.as_str()),
            ),
            cst_pis_desc: collapse_or_diversos(
                parsed_items.iter().map(|p| p.cst_pis_desc.as_str()),
            ),
            valor_total: format_br_float(total_nfe, 2),
            valor_nota_proporcional: "0,00".to_string(),
            valor_descontos: "0,00".to_string(),
            valor_seguro: "0,00".to_string(),
            cofins_valor: "0,00".to_string(),
            pis_valor: "0,00".to_string(),
            ipi_valor: "0,00".to_string(),
            iss_bc: "0,00".to_string(),
            iss_valor: "0,00".to_string(),
            icms_aliquota: "0,00".to_string(),
            icms_bc: "0,00".to_string(),
            icms_valor: "0,00".to_string(),
            icms_st_valor: "0,00".to_string(),
            ..base_row.clone()
        });
    }

    for item in parsed_items {
        let valor_total_row = if multi_item {
            0.0
        } else if item.valor_total_nfe > 0.0 {
            item.valor_total_nfe
        } else {
            item.valor_proporcional
        };

        out.push(InfoReceita {
            nat_bc_credito_desc: item.nat_bc_credito_desc,
            numero_di: item.numero_di,
            numero_item: item.numero_item,
            codigo_cfop: item.cfop,
            descricao_cfop: item.desc_cfop,
            descricao_mercadoria: item.descricao,
            codigo_ncm: item.ncm,
            descricao_ncm: item.desc_ncm,
            cofins_aliq: item.cofins_aliq,
            pis_aliq: item.pis_aliq,
            cst_cofins_desc: item.cst_cofins_desc,
            cst_pis_desc: item.cst_pis_desc,
            valor_total: format_br_float(valor_total_row, 2),
            valor_nota_proporcional: format_br_float(item.valor_proporcional, 2),
            valor_descontos: format_br_float(item.valor_desconto, 2),
            valor_seguro: format_br_float(item.valor_seguro, 2),
            cofins_valor: format_br_float(item.valor_cofins, 2),
            pis_valor: format_br_float(item.valor_pis, 2),
            ipi_valor: format_br_float(item.valor_ipi, 2),
            iss_bc: format_br_float(item.bc_iss, 2),
            iss_valor: format_br_float(item.valor_iss, 2),
            icms_aliquota: format_br_float(item.aliq_icms, 2),
            icms_bc: format_br_float(item.bc_icms, 2),
            icms_valor: format_br_float(item.valor_icms, 2),
            icms_st_valor: format_br_float(item.valor_icms_st, 2),
            ..base_row.clone()
        });
    }

    out
}

fn build_nfe_observacoes(fisco: Option<&str>, contrib: Option<&str>) -> String {
    let fisco = fisco
        .map(crate::strip_line_breaks)
        .filter(|s| !s.is_empty());
    let contrib = contrib
        .map(crate::strip_line_breaks)
        .filter(|s| !s.is_empty());

    match (fisco, contrib) {
        (Some(f), Some(c)) => format!("Informações do fisco: {f} Informações do contribuinte: {c}"),
        (Some(f), None) => format!("Informações do fisco: {f}"),
        (None, Some(c)) => format!("Informações do contribuinte: {c}"),
        (None, None) => ND.to_string(),
    }
}

// =============================================================================
// HELPERS DE FORMATAÇÃO E AGREGAÇÃO (DRY)
// =============================================================================

#[inline]
fn opt_str_or_nd(opt: Option<&str>) -> String {
    opt.map(crate::strip_line_breaks)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ND.to_string())
}

#[inline]
fn first_non_empty_opt(opts: &[Option<&str>]) -> String {
    opts.iter()
        .flatten()
        .map(|s| crate::strip_line_breaks(s))
        .find(|s| !s.is_empty())
        .unwrap_or_else(|| ND.to_string())
}

#[inline]
fn opt_display_or_nd<T: fmt::Display>(opt: Option<T>) -> String {
    opt.map_or_else(|| ND.to_string(), |v| v.to_string())
}

#[inline]
fn doc_and_name(
    cnpj: &Option<String>,
    cpf: &Option<String>,
    name: &Option<String>,
) -> (String, String) {
    (
        first_non_empty_opt(&[cnpj.as_deref(), cpf.as_deref()]),
        opt_str_or_nd(name.as_deref()),
    )
}

#[inline]
fn format_opt_br(val: Option<f64>) -> String {
    format_br_float(val.unwrap_or(0.0), 2)
}

/// Agrupa uma sequência de strings retornando o valor comum caso todas coincidam,
/// ou `*DIVERSOS*` caso existam divergências.
pub fn collapse_or_diversos<'a, I>(mut iter: I) -> String
where
    I: Iterator<Item = &'a str>,
{
    let Some(first) = iter.next() else {
        return ND.to_string();
    };
    if iter.all(|item| item == first) {
        first.to_string()
    } else {
        DIVERSOS.to_string()
    }
}

/// Formata `f64` no padrão numérico brasileiro (`1.234.567,89`).
pub fn format_br_float(val: f64, decimals: usize) -> String {
    if !val.is_finite() {
        return format!("0,{}", "0".repeat(decimals));
    }

    let threshold = 0.5 * 10_f64.powi(-(decimals as i32));
    let clean = if val.abs() < threshold { 0.0 } else { val };

    let is_neg = clean.is_sign_negative() && clean != 0.0;
    let formatted = format!("{:.*}", decimals, clean.abs());

    let (int_part, frac_part) = formatted.split_once('.').unwrap_or((&formatted, ""));

    let mut rev_int = String::with_capacity(int_part.len() + int_part.len() / 3);
    for (idx, ch) in int_part.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            rev_int.push('.');
        }
        rev_int.push(ch);
    }

    let int_with_dots: String = rev_int.chars().rev().collect();
    let sign = if is_neg { "-" } else { "" };

    if decimals == 0 {
        format!("{sign}{int_with_dots}")
    } else {
        format!("{sign}{int_with_dots},{frac_part}")
    }
}

#[inline]
fn format_date_br(date: Option<NaiveDate>) -> String {
    date.map_or_else(|| ND.to_string(), |dt| dt.format("%d/%m/%Y").to_string())
}

#[inline]
fn format_cancelada(cancelado: Option<&str>) -> String {
    if cancelado.is_some_and(|s| s.trim().eq_ignore_ascii_case("Sim")) {
        "Sim".to_string()
    } else {
        "Não".to_string()
    }
}

#[inline]
fn cfop_is_entrada(cfop: &str) -> bool {
    cfop.trim()
        .chars()
        .next()
        .is_some_and(|c| matches!(c, '1' | '2' | '3'))
}

// =============================================================================
// TABELAS DE LOOKUP (CONFAZ CFOP, CST PIS/COFINS, NATUREZA DA BASE DE CÁLCULO)
// =============================================================================

/// Obtém a Natureza da Base de Cálculo dos Créditos com base no CFOP e no CST
/// (conforme regra oficial do `rust-sped`: CST em `50..=56 | 60..=66` e CFOP gerador de crédito).
pub fn lookup_natureza_bc_desc(cfop: &str, cst: &str) -> Option<&'static str> {
    let cst_code = cst.trim().parse::<u16>().ok()?;
    if !matches!(cst_code, 50..=56 | 60..=66) {
        return None;
    }

    let cfop_code = cfop.trim().parse::<u16>().ok()?;
    match CFOP_NATUREZA_BC_TABLE.binary_search_by_key(&cfop_code, |&(k, _)| k) {
        Ok(idx) => Some(CFOP_NATUREZA_BC_TABLE[idx].1),
        Err(_) => None,
    }
}

/// Retorna a descrição padrão da Receita Federal para o CST de PIS/COFINS.
pub fn lookup_cst_pis_cofins_desc(cst: &str, is_entrada: bool) -> &'static str {
    let trimmed = cst.trim();
    if trimmed.is_empty() {
        return ND;
    }

    let Ok(code) = trimmed.parse::<u8>() else {
        return ND;
    };

    if is_entrada {
        match code {
            1 | 50..=56 => "CRÉDITO/TRIB (MI)",
            2 => "TRIBUTADO ALIQ. DIFERENCIADA",
            3 => "TRIBUTADO POR UNIDADE DE MEDIDA",
            4 => "TRIBUTADO MONOFASICO ALIQ. ZERO",
            5 => "TRIBUTADO SUBST. TRIBUTARIA",
            6 | 73 => "AQUISIÇÃO/ALIQ ZERO",
            7 | 71 => "AQUISIÇÃO ISENTA",
            8 | 74 => "AQUISIÇÃO SEM INCIDÊNCIA",
            9 | 72 => "AQUISIÇÃO COM SUSPENSÃO",
            60..=67 => "CRÉDITO PRESUMIDO",
            70 => "AQUISIÇÃO SEM DIREITO A CRÉDITO",
            75 => "AQUISIÇÃO POR SUBST. TRIB.",
            49 | 98 | 99 => "OUTRAS OPERAÇÕES",
            _ => ND,
        }
    } else {
        match code {
            1 => "TRIBUTADO AD VALOREM",
            2 => "TRIBUTADO ALIQ. DIFERENCIADA",
            3 => "TRIBUTADO POR UNIDADE DE MEDIDA",
            4 => "TRIBUTADO MONOFASICO ALIQ. ZERO",
            5 => "TRIBUTADO SUBST. TRIBUTARIA",
            6 => "TRIBUTADO ALIQ. ZERO",
            7 => "ISENTO",
            8 => "SEM INCIDENCIA",
            9 => "COM SUSPENSAO",
            49 => "OUTRAS SAIDAS",
            99 => "OUTRAS OPERAÇÕES",
            _ => ND,
        }
    }
}

const NAT_01: &str = "Aquisição de Bens para Revenda";
const NAT_02: &str = "Aquisição de Bens Utilizados como Insumo";
const NAT_03: &str = "Aquisição de Serviços Utilizados como Insumo";
const NAT_12: &str = "Devolução de Vendas Sujeitas à Incidência Não-Cumulativa";
const NAT_13: &str = "Outras Operações com Direito a Crédito";

/// Tabela ordenada de CFOPs geradores de crédito -> Natureza da Base de Cálculo (`rust-sped/tabelas.rs`).
static CFOP_NATUREZA_BC_TABLE: &[(u16, &str)] = &[
    (1101, NAT_02),
    (1102, NAT_01),
    (1111, NAT_02),
    (1113, NAT_01),
    (1116, NAT_02),
    (1117, NAT_01),
    (1118, NAT_01),
    (1120, NAT_02),
    (1121, NAT_01),
    (1122, NAT_02),
    (1124, NAT_03),
    (1125, NAT_03),
    (1126, NAT_02),
    (1128, NAT_02),
    (1132, NAT_02),
    (1135, NAT_02),
    (1159, NAT_01),
    (1201, NAT_12),
    (1202, NAT_12),
    (1203, NAT_12),
    (1204, NAT_12),
    (1206, NAT_12),
    (1207, NAT_12),
    (1215, NAT_12),
    (1216, NAT_12),
    (1251, NAT_01),
    (1401, NAT_02),
    (1403, NAT_01),
    (1407, NAT_02),
    (1410, NAT_12),
    (1411, NAT_12),
    (1456, NAT_02),
    (1556, NAT_02),
    (1651, NAT_02),
    (1652, NAT_01),
    (1653, NAT_02),
    (1660, NAT_12),
    (1661, NAT_12),
    (1662, NAT_12),
    (1922, NAT_13),
    (1933, NAT_03),
    (2101, NAT_02),
    (2102, NAT_01),
    (2111, NAT_02),
    (2113, NAT_01),
    (2116, NAT_02),
    (2117, NAT_01),
    (2118, NAT_01),
    (2120, NAT_02),
    (2121, NAT_01),
    (2122, NAT_02),
    (2124, NAT_03),
    (2125, NAT_03),
    (2126, NAT_02),
    (2128, NAT_02),
    (2132, NAT_02),
    (2135, NAT_02),
    (2159, NAT_01),
    (2201, NAT_12),
    (2202, NAT_12),
    (2206, NAT_12),
    (2207, NAT_12),
    (2215, NAT_12),
    (2216, NAT_12),
    (2251, NAT_01),
    (2401, NAT_02),
    (2403, NAT_01),
    (2407, NAT_02),
    (2410, NAT_12),
    (2411, NAT_12),
    (2456, NAT_02),
    (2556, NAT_02),
    (2651, NAT_02),
    (2652, NAT_01),
    (2653, NAT_02),
    (2660, NAT_12),
    (2661, NAT_12),
    (2662, NAT_12),
    (2922, NAT_13),
    (2933, NAT_03),
    (3101, NAT_02),
    (3102, NAT_01),
    (3126, NAT_02),
    (3128, NAT_02),
    (3251, NAT_01),
    (3556, NAT_02),
    (3651, NAT_02),
    (3652, NAT_01),
    (3653, NAT_02),
];
// =============================================================================
// TESTES UNITÁRIOS
// =============================================================================

#[cfg(test)]
mod tests_receita {
    use super::*;

    #[test]
    fn test_tables_are_strictly_sorted_for_binary_search() {
        assert!(
            CFOP_NATUREZA_BC_TABLE.windows(2).all(|w| w[0].0 < w[1].0),
            "CFOP_NATUREZA_BC_TABLE deve estar estritamente ordenada"
        );
    }

    #[test]
    fn test_format_br_float() {
        assert_eq!(format_br_float(0.0, 2), "0,00");
        assert_eq!(format_br_float(-0.0001, 2), "0,00");
        assert_eq!(format_br_float(10763.8, 2), "10.763,80");
        assert_eq!(format_br_float(1234567.89, 2), "1.234.567,89");
        assert_eq!(format_br_float(-1234.5, 2), "-1.234,50");
        assert_eq!(format_br_float(0.12, 2), "0,12");
    }

    #[test]
    fn test_collapse_or_diversos() {
        assert_eq!(collapse_or_diversos(["5405", "5405"].into_iter()), "5405");
        assert_eq!(
            collapse_or_diversos(["5102", "5405"].into_iter()),
            "*DIVERSOS*"
        );
        assert_eq!(collapse_or_diversos(std::iter::empty()), "<N/D>");
    }

    #[test]
    fn test_lookup_cfop_cst_and_natureza_bc() {
        assert_eq!(
            lookup_cfop_desc("5352"),
            "Prestação de serviço de transporte a estabelecimento industrial"
        );
        assert_eq!(
            lookup_cfop_desc("7949"),
            "Outra saída de mercadoria ou prestação de serviço não especificado"
        );
        assert_eq!(lookup_cfop_desc("9999"), "<N/D>");

        assert_eq!(
            lookup_cst_pis_cofins_desc("1", false),
            "TRIBUTADO AD VALOREM"
        );
        assert_eq!(
            lookup_cst_pis_cofins_desc("06", false),
            "TRIBUTADO ALIQ. ZERO"
        );
        assert_eq!(lookup_cst_pis_cofins_desc("49", false), "OUTRAS SAIDAS");
        assert_eq!(lookup_cst_pis_cofins_desc("50", true), "CRÉDITO/TRIB (MI)");
        assert_eq!(
            lookup_cst_pis_cofins_desc("73", true),
            "AQUISIÇÃO/ALIQ ZERO"
        );

        assert_eq!(
            lookup_natureza_bc_desc("1410", "50"),
            Some("Devolução de Vendas Sujeitas à Incidência Não-Cumulativa")
        );
        assert_eq!(
            lookup_natureza_bc_desc("1101", "50"),
            Some("Aquisição de Bens Utilizados como Insumo")
        );
        assert_eq!(lookup_natureza_bc_desc("1410", "73"), None);
        assert_eq!(lookup_natureza_bc_desc("5102", "01"), None);
    }

    #[test]
    fn test_info_receita_headers_count_and_boundaries() {
        let headers = InfoReceita::get_headers();
        assert_eq!(headers.len(), 56);
        assert_eq!(headers[0], "CNPJ do Contribuinte : NF Item (Todos)");
        assert_eq!(
            headers[28],
            "Chave da Nota Fiscal Eletrônica : NF Item (Todos)"
        );
        assert_eq!(
            headers[55],
            "ICMS por Substituição: Valor do Tributo : NF Item (Todos) SOMA"
        );
    }

    #[test]
    fn test_cte_sem_inf_nfe_preenche_colunas_coerentes_e_clean_spaces() {
        let mut cte = InfoCte {
            emitente_cnpj: Some("00.130.057/0001-05".to_string()),
            emitente_nome: Some("FN TRANSPORTE LTDA".to_string()),
            emitente_crt: Some(3),
            remetente_cnpj: Some("02.948.030/0002-30".to_string()),
            remetente_nome: Some("ESMALTEC S/A".to_string()),
            remetente_ender_municipio: Some("MARACANAÚ".to_string()),
            destinatario_cnpj: Some("47.960.950/1100-67".to_string()),
            destinatario_nome: Some("MAGAZINE LUIZA SA".to_string()),
            tomador_codigo: Some(0),
            cte: Some("23260100130057000105570010000198841285476050".to_string()),
            numero_da_nota: Some(19884),
            nat_operacao: Some(
                "Prest. Serviço de Transp.\nEstabelecimento   industrial".to_string(),
            ),
            uf_inicio: Some("CE".to_string()),
            mun_inicio: Some("Maracanaú".to_string()),
            uf_termino: Some("BA".to_string()),
            mun_termino: Some("Candeias".to_string()),
            obs_gerais: Some("Linha 1\r\nLinha  2\tFim".to_string()),
            cfop: Some(6352),
            data_emissao: NaiveDate::from_ymd_opt(2026, 1, 12),
            valor_total: Some(810.23),
            v_bc_icms: Some(810.23),
            aliq_icms: Some(12.0),
            v_icms: Some(97.23),
            ..Default::default()
        };

        let mut sem_nfe = ctes_to_receita(&[cte.clone()]);
        assert_eq!(sem_nfe.len(), 1);
        let row = &sem_nfe[0];
        assert_eq!(
            row.chave_nfe,
            "23260100130057000105570010000198841285476050"
        );
        assert_eq!(row.cte_rem_cnpj_comp, "02.948.030/0002-30");
        assert_eq!(row.cte_rem_cnpj_nfe, "02.948.030/0002-30");
        assert_eq!(row.cte_rem_nome_nfe, "ESMALTEC S/A");
        assert_eq!(row.cte_rem_mun_nfe, "MARACANAÚ");
        assert_eq!(row.cte_papel_tomador_comp, "Remetente");
        assert_eq!(row.cte_papel_tomador_nfe, "Remetente");
        assert_eq!(row.inf_nfe_chave, "<N/D>");
        // Sem --clean-spaces: quebras removidas, múltiplos espaços internos preservados
        assert_eq!(row.cte_obs_gerais, "Linha 1 Linha  2 Fim");
        assert_eq!(
            row.desc_natureza_operacao,
            "Prest. Serviço de Transp. Estabelecimento   industrial"
        );

        // Com --clean-spaces: múltiplos espaços também colapsam para um único espaço
        crate::SanitizeText::clean_spaces(&mut sem_nfe[0]);
        assert_eq!(sem_nfe[0].cte_obs_gerais, "Linha 1 Linha 2 Fim");
        assert_eq!(
            sem_nfe[0].desc_natureza_operacao,
            "Prest. Serviço de Transp. Estabelecimento industrial"
        );

        cte.nfes_vinculados = vec!["23260102948030000230550010018611891935989130".to_string()];
        let com_nfe = ctes_to_receita(&[cte]);
        assert_eq!(
            com_nfe[0].inf_nfe_chave,
            "23260102948030000230550010018611891935989130"
        );
    }

    #[test]
    fn test_canonicalizacao_e_ordenacao_ctes() {
        let base = InfoCte {
            emitente_cnpj: Some("00.124.733/0016-14".to_string()),
            emitente_nome: Some("H U TRANSPORTE RODOVIARIO LTDA".to_string()),
            remetente_cnpj: Some("50.221.019/0097-88".to_string()),
            remetente_nome: Some("HNK BR INDUSTRIA DE BEBIDAS LTDA.".to_string()),
            tomador_codigo: Some(0),
            uf_inicio: Some("SP".to_string()),
            mun_inicio: Some("GUARULHOS".to_string()),
            uf_termino: Some("PR".to_string()),
            mun_termino: Some("PONTA GROSSA".to_string()),
            cfop: Some(6932),
            ..Default::default()
        };

        let cte_255 = InfoCte {
            cte: Some("41260100124733001614570020000002551014397424".to_string()),
            numero_da_nota: Some(255),
            ..base.clone()
        };
        let cte_256 = InfoCte {
            cte: Some("41260100124733001614570020000002561014397472".to_string()),
            numero_da_nota: Some(256),
            ..base
        };
        let cte_757 = InfoCte {
            emitente_cnpj: Some("00.124.733/0016-14".to_string()),
            emitente_nome: Some("HU - TRANSPORTE RODOVIARIO LTDA".to_string()),
            remetente_cnpj: Some("50.221.019/0097-88".to_string()),
            remetente_nome: Some("HNK BR INDUSTRIA DE BEBIDAS LTDA".to_string()),
            tomador_codigo: Some(0),
            cte: Some("41260100124733001614570520000007571000075798".to_string()),
            numero_da_nota: Some(757),
            uf_inicio: Some("PR".to_string()),
            mun_inicio: Some("PONTA GROSSA".to_string()),
            uf_termino: Some("SP".to_string()),
            mun_termino: Some("LOUVEIRA".to_string()),
            nfes_vinculados: vec!["1".repeat(44), "2".repeat(44)],
            cfop: Some(6352),
            ..Default::default()
        };

        let res = ctes_to_receita(&[cte_256, cte_757, cte_255]);
        assert_eq!(res[0].numero_nota, "757");
        assert_eq!(res[0].nome_contribuinte, "H U TRANSPORTE RODOVIARIO LTDA");
        assert_eq!(
            res[0].nome_participante,
            "HNK BR INDUSTRIA DE BEBIDAS LTDA."
        );
        assert_eq!(res[0].cte_rem_nome_nfe, "HNK BR INDUSTRIA DE BEBIDAS LTDA");
        assert_eq!(res[0].inf_nfe_chave, "*DIVERSOS*");
        assert_eq!(res[1].numero_nota, "255");
        assert_eq!(res[2].numero_nota, "256");
    }

    #[test]
    fn test_nfe_multi_item_gera_item_zero_e_preserva_ordem() {
        let base = InfoNfe {
            emitente_cnpj: Some("00.206.861/0013-64".to_string()),
            emitente_nome: Some("AZ COMERCIO DE TINTAS LTDA".to_string()),
            emitente_crt: Some(3),
            destinatario_cnpj: Some("47.964.911/0001-00".to_string()),
            destinatario_nome: Some("USINA DE LATICINIOS JUSSARA SA".to_string()),
            nfe: Some("35250500206861001364550010001321681637262329".to_string()),
            tp_nf: Some("1".to_string()),
            nat_operacao: Some("VENDA MERCAD. ADQUIRIDA TERCEIROS".to_string()),
            numero_da_nota: Some(132168),
            data_emissao: NaiveDate::from_ymd_opt(2025, 5, 30),
            valor_total_nfe: Some(400.0),
            valor_total_itens: Some(400.0),
            ..Default::default()
        };

        let item1 = InfoNfe {
            n_item: Some(1),
            cfop: Some(5102),
            descricao: Some("PRODUTO A".to_string()),
            ncm: Some("7326.90.90".to_string()),
            v_prod: Some(100.0),
            v_icms_st: Some(12.34),
            ..base.clone()
        };
        let item2 = InfoNfe {
            n_item: Some(2),
            cfop: Some(5405),
            descricao: Some("PRODUTO B".to_string()),
            ncm: Some("3208.20.19".to_string()),
            v_prod: Some(300.0),
            ..base
        };

        let res = nfes_to_receita(&[item1, item2]);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].numero_item, "0");
        assert_eq!(res[0].codigo_cfop, "*DIVERSOS*");
        assert_eq!(res[0].valor_total, "400,00");
        assert_eq!(res[0].valor_nota_proporcional, "0,00");
        assert_eq!(res[0].icms_st_valor, "0,00");
        assert_eq!(res[1].numero_item, "1");
        assert_eq!(
            res[1].chave_nfe,
            "35250500206861001364550010001321681637262329"
        );
        assert_eq!(res[1].valor_nota_proporcional, "100,00");
        assert_eq!(res[1].icms_st_valor, "12,34");
        assert_eq!(res[2].numero_item, "2");
        assert_eq!(res[2].valor_nota_proporcional, "300,00");
    }
}
