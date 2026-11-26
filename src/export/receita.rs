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
    collections::{BTreeSet, HashMap, HashSet},
    fmt,
};
use struct_iterable::Iterable;

use crate::{
    InfoExtension, SanitizeText, lookup_cfop_desc, lookup_cst_pis_cofins_desc,
    lookup_natureza_bc_desc, lookup_ncm_desc,
    xml_structs::{cte::InfoCte, nfe::InfoNfe},
};

pub const RECEITA_ND: &str = "<N/D>";
const ND: &str = RECEITA_ND;
const DIVERSOS: &str = "*DIVERSOS*";

/// Estilo de representação das colunas de ponto flutuante do layout Receita.
///
/// O padrão é o formato **nativo** dos arquivos XML (sem separador de milhar e com
/// `.` como separador decimal), idêntico ao já usado pelos leiautes nativos
/// (`InfoCte` / `InfoNfe`). O formato brasileiro é opcional (`-f` / `--format`) e
/// existe apenas para comparação com os extratos da Receita Federal.
///
/// Colunas inteiras ou de código (CRT, Nº da Nota, Nº do Item, CFOP, NCM, Nº da DI)
/// e chaves de acesso de 44 dígitos nunca são afetadas.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormat {
    /// Formato nativo do XML / `f64`: `123456.78`, `0.3456`, `3.0`, `0.0`.
    #[default]
    Native,
    /// Formato brasileiro: `123.456,78` (milhar `.`, decimal `,`).
    Brazilian,
}

impl From<bool> for NumberFormat {
    #[inline]
    fn from(brazilian: bool) -> Self {
        if brazilian {
            Self::Brazilian
        } else {
            Self::Native
        }
    }
}

/// Opções de geração do layout padrão da Receita Federal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReceitaOptions {
    pub number_format: NumberFormat,
    /// Quando `true` (`--diversos`), exibe `*DIVERSOS*` em CT-es com múltiplas NF-es.
    /// Por padrão (`false`), exibe todas as chaves correlacionadas.
    pub diversos: bool,
}

impl ReceitaOptions {
    #[inline]
    pub const fn new(number_format: NumberFormat, diversos: bool) -> Self {
        Self {
            number_format,
            diversos,
        }
    }
}

impl From<NumberFormat> for ReceitaOptions {
    #[inline]
    fn from(number_format: NumberFormat) -> Self {
        Self {
            number_format,
            diversos: false,
        }
    }
}

impl NumberFormat {
    /// Formata um valor com a quantidade de casas decimais informada.
    #[inline]
    pub fn format(self, val: f64, decimals: usize) -> String {
        match self {
            Self::Native => format_native_float(val, decimals),
            Self::Brazilian => format_br_float(val, decimals),
        }
    }

    /// Valor monetário (2 casas decimais). Aceita `f64` ou `Option<f64>`.
    #[inline]
    pub fn money(self, val: impl Into<Option<f64>>) -> String {
        self.format(val.into().unwrap_or(0.0), 2)
    }

    /// Alíquota (4 casas decimais em ambos os formatos). Aceita `f64` ou `Option<f64>`.
    ///
    /// No formato nativo a impressão é mínima (`7.6`, `0.18`, `0.3456`); no
    /// brasileiro são sempre 4 casas (`7,6000`, `0,1800`, `0,3456`).
    #[inline]
    pub fn rate(self, val: impl Into<Option<f64>>) -> String {
        self.format(val.into().unwrap_or(0.0), 4)
    }
}

/// Formata `f64` no formato nativo dos arquivos XML: sem separador de milhar, `.`
/// como separador decimal e a menor representação que preserva o valor
/// (`0.0`, `3.0`, `44556677.1`, `0.3456`).
///
/// O valor é arredondado para `decimals` casas antes da impressão, eliminando
/// resíduos binários de operações como o rateio proporcional
/// (`100.00000000000001` -> `100.0`).
pub fn format_native_float(val: f64, decimals: usize) -> String {
    if !val.is_finite() {
        return "0.0".to_string();
    }
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (val * factor).round() / factor;
    if rounded == 0.0 {
        return "0.0".to_string();
    }
    let s = format!("{rounded}");
    if s.contains('.') { s } else { format!("{s}.0") }
}

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
pub fn ctes_to_receita(ctes: &[InfoCte], options: impl Into<ReceitaOptions>) -> Vec<InfoReceita> {
    let opts = options.into();
    let mut rows: Vec<InfoReceita> = ctes.par_iter().map(|cte| convert_cte(cte, opts)).collect();
    canonicalize_cte_names(&mut rows);
    sort_receita_rows(&mut rows);
    rows
}

/// Converte uma fatia de [`InfoNfe`] (preferencialmente ordenada por chave e item)
/// para o formato unificado da Receita Federal, gerando a linha sintética `Item 0`
/// sempre que uma NF-e possuir mais de um item e ordenando no padrão da Receita Federal.
pub fn nfes_to_receita(nfes: &[InfoNfe], options: impl Into<ReceitaOptions>) -> Vec<InfoReceita> {
    let opts = options.into();
    let chunks: Vec<&[InfoNfe]> = nfes
        .chunk_by(|a, b| a.nfe.is_some() && a.nfe == b.nfe)
        .collect();

    let mut rows: Vec<InfoReceita> = chunks
        .par_iter()
        .flat_map_iter(|chunk| convert_nfe_chunk(chunk, opts))
        .collect();

    sort_receita_rows(&mut rows);
    rows
}

/// Extrai `(cnpj_contribuinte, nome_contribuinte, cpf_cnpj_participante, nome_participante)` de um CT-e.
pub fn cte_contrib_and_part(cte: &InfoCte) -> (String, String, String, String) {
    let contrib_doc =
        first_non_empty_opt(&[cte.emitente_cnpj.as_deref(), cte.emitente_cpf.as_deref()]);
    let contrib_name = opt_str_or_nd(cte.emitente_nome.as_deref());
    let (part_doc, part_name) = match cte.tomador_codigo {
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
    (contrib_doc, contrib_name, part_doc, part_name)
}

/// Contabiliza votos de nomes para canonicalização (usado pelo motor out-of-core).
pub fn vote_cte_names(
    cte: &InfoCte,
    clean_spaces: bool,
    contrib_votes: &mut HashMap<String, HashMap<String, usize>>,
    part_votes: &mut HashMap<String, HashMap<String, usize>>,
) {
    let (contrib_doc, mut contrib_name, part_doc, mut part_name) = cte_contrib_and_part(cte);
    if clean_spaces {
        contrib_name.clean_spaces();
        part_name.clean_spaces();
    }
    if contrib_doc != ND && contrib_name != ND {
        *contrib_votes
            .entry(contrib_doc)
            .or_default()
            .entry(contrib_name)
            .or_insert(0) += 1;
    }
    if part_doc != ND && part_name != ND {
        *part_votes
            .entry(part_doc)
            .or_default()
            .entry(part_name)
            .or_insert(0) += 1;
    }
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
pub fn resolve_canonical_map<K>(counts: HashMap<K, HashMap<K, usize>>) -> HashMap<String, String>
where
    K: AsRef<str> + Eq + std::hash::Hash,
{
    counts
        .into_iter()
        .filter_map(|(doc, name_map)| {
            let best = name_map
                .into_iter()
                .max_by(|(n1, c1), (n2, c2)| {
                    c1.cmp(c2)
                        .then_with(|| n1.as_ref().len().cmp(&n2.as_ref().len()))
                        .then_with(|| n2.as_ref().cmp(n1.as_ref()))
                })?
                .0;
            Some((doc.as_ref().to_string(), best.as_ref().to_string()))
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
        if r.nome_contribuinte == ND
            && let Some(canonical) = contrib_map.get(&r.cnpj_contribuinte)
        {
            r.nome_contribuinte = canonical.clone();
        }
        if r.nome_participante == ND
            && let Some(canonical) = part_map.get(&r.cpf_cnpj_participante)
        {
            r.nome_participante = canonical.clone();
        }
    }
}

/// Compara dois registros [`InfoReceita`] no padrão oficial da Receita Federal (`<N/D>` primeiro).
pub fn compare_receita_rows(a: &InfoReceita, b: &InfoReceita) -> std::cmp::Ordering {
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
}

/// Ordena registros [`InfoReceita`] em paralelo utilizando [`compare_receita_rows`].
pub fn sort_receita_rows(rows: &mut [InfoReceita]) {
    rows.par_sort_by(compare_receita_rows);
}

// =============================================================================
// LÓGICA DE TRANSFORMAÇÃO: CT-e
// =============================================================================

pub fn convert_cte_to_receita(cte: &InfoCte, options: impl Into<ReceitaOptions>) -> InfoReceita {
    convert_cte(cte, options.into())
}

fn convert_cte(cte: &InfoCte, options: ReceitaOptions) -> InfoReceita {
    let fmt = options.number_format;
    let papel = match cte.tomador_codigo {
        Some(0) => "Remetente",
        Some(1) => "Expedidor",
        Some(2) => "Recebedor",
        Some(3) => "Destinatário",
        Some(4) => "Outros",
        _ => ND,
    }
    .to_string();

    let (cnpj_contribuinte, nome_contribuinte, part_doc, part_nome) = cte_contrib_and_part(cte);

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

    let inf_nfe_chave = format_cte_linked_keys(cte, options.diversos);
    let cte_obs_gerais = opt_str_or_nd(cte.obs_gerais.as_deref());

    let valor_total_str = fmt.money(cte.valor_total);
    let aliq_icms = cte.aliq_icms.unwrap_or(0.0) / 100.0;

    InfoReceita {
        cnpj_contribuinte,
        nome_contribuinte,
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
            "CTeOS" => "Conhecimento de Transporte Eletrônico para Outros Serviços CT-e OS",
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
        cofins_aliq: fmt.rate(cte.aliq_cofins),
        pis_aliq: fmt.rate(cte.aliq_pis),
        cst_cofins_desc: ND.to_string(),
        cst_pis_desc: ND.to_string(),
        valor_total: valor_total_str.clone(),
        valor_nota_proporcional: valor_total_str,
        valor_descontos: fmt.money(0.0),
        valor_seguro: fmt.money(0.0),
        cofins_valor: fmt.money(cte.v_cofins),
        pis_valor: fmt.money(cte.v_pis),
        ipi_valor: fmt.money(cte.v_ipi),
        iss_bc: fmt.money(cte.v_bc_iss),
        iss_valor: fmt.money(cte.v_iss),
        icms_aliquota: fmt.rate(aliq_icms),
        icms_bc: fmt.money(cte.v_bc_icms),
        icms_valor: fmt.money(cte.v_icms),
        icms_st_valor: fmt.money(cte.v_icms_st),
    }
}

fn format_cte_linked_keys(cte: &InfoCte, diversos: bool) -> String {
    let mut all_keys: BTreeSet<String> = cte
        .get_correlated_nfes()
        .into_iter()
        .chain(cte.nfes.iter().cloned())
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();

    if diversos {
        return match all_keys.len() {
            0 => ND.to_string(),
            1 => all_keys
                .into_iter()
                .next()
                .unwrap_or_else(|| ND.to_string()),
            _ => DIVERSOS.to_string(),
        };
    }

    all_keys.extend(
        cte.get_correlated_ctes()
            .into_iter()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty()),
    );
    if let Some(own) = cte.cte.as_deref().map(str::trim) {
        all_keys.remove(own);
    }

    if all_keys.is_empty() {
        ND.to_string()
    } else {
        all_keys.into_iter().collect::<Vec<_>>().join(", ")
    }
}

fn format_nfe_linked_keys(nfe: &InfoNfe, diversos: bool) -> String {
    if diversos {
        return ND.to_string();
    }

    let mut all_keys: BTreeSet<String> = nfe
        .ctes
        .iter()
        .chain(nfe.nfes_relacionadas.iter())
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();

    if let Some(own) = nfe.nfe.as_deref().map(str::trim) {
        all_keys.remove(own);
    }

    if all_keys.is_empty() {
        ND.to_string()
    } else {
        all_keys.into_iter().collect::<Vec<_>>().join(", ")
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

pub fn convert_nfe_chunk_to_receita(
    chunk: &[InfoNfe],
    options: impl Into<ReceitaOptions>,
) -> Vec<InfoReceita> {
    convert_nfe_chunk(chunk, options.into())
}

fn convert_nfe_chunk(chunk: &[InfoNfe], options: ReceitaOptions) -> Vec<InfoReceita> {
    let fmt = options.number_format;
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
    let sum_v_items: f64 = chunk.iter().filter_map(|it| it.v_item_total).sum();
    let total_vb01: f64 = chunk
        .iter()
        .map(|it| {
            it.v_item_total.unwrap_or_else(|| {
                (it.v_prod.unwrap_or(0.0) - it.v_desc.unwrap_or(0.0)
                    + it.v_frete.unwrap_or(0.0)
                    + it.v_seg.unwrap_or(0.0)
                    + it.v_outro.unwrap_or(0.0)
                    + it.v_ipi.unwrap_or(0.0)
                    + it.v_icms_st.unwrap_or(0.0))
                .max(0.0)
            })
        })
        .sum();

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
            let valor_item_vb01 = (valor_item - item.v_desc.unwrap_or(0.0)
                + item.v_frete.unwrap_or(0.0)
                + item.v_seg.unwrap_or(0.0)
                + item.v_outro.unwrap_or(0.0)
                + item.v_ipi.unwrap_or(0.0)
                + item.v_icms_st.unwrap_or(0.0))
            .max(0.0);

            let valor_proporcional = match (item.valor_total_nfe, item.v_item_total) {
                (Some(v_nf), _) if !multi_item => v_nf,
                (Some(v_nf), Some(v_it)) if sum_v_items > 0.0 => {
                    if (sum_v_items - v_nf).abs() <= 0.02 {
                        v_it
                    } else {
                        v_it * (v_nf / sum_v_items)
                    }
                }
                (None, Some(v_it)) => v_it,
                (Some(v_nf), None) if total_vb01 > 0.0 => {
                    if (total_vb01 - v_nf).abs() <= 0.02 {
                        valor_item_vb01
                    } else {
                        valor_item_vb01 * (v_nf / total_vb01)
                    }
                }
                (Some(v_nf), _) if valor_total_itens > 0.0 => {
                    valor_item * (v_nf / valor_total_itens)
                }
                (Some(v_nf), _) => v_nf,
                (None, None) => valor_item_vb01,
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
                cofins_aliq: fmt.rate(item.aliq_cofins),
                pis_aliq: fmt.rate(item.aliq_pis),
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
        inf_nfe_chave: format_nfe_linked_keys(first_nfe, options.diversos),
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
            valor_total: fmt.money(total_nfe),
            valor_nota_proporcional: fmt.money(0.0),
            valor_descontos: fmt.money(0.0),
            valor_seguro: fmt.money(0.0),
            cofins_valor: fmt.money(0.0),
            pis_valor: fmt.money(0.0),
            ipi_valor: fmt.money(0.0),
            iss_bc: fmt.money(0.0),
            iss_valor: fmt.money(0.0),
            icms_aliquota: fmt.rate(0.0),
            icms_bc: fmt.money(0.0),
            icms_valor: fmt.money(0.0),
            icms_st_valor: fmt.money(0.0),
            ..base_row.clone()
        });
    }

    for item in parsed_items {
        let valor_total_row = if multi_item {
            0.0
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
            valor_total: fmt.money(valor_total_row),
            valor_nota_proporcional: fmt.money(item.valor_proporcional),
            valor_descontos: fmt.money(item.valor_desconto),
            valor_seguro: fmt.money(item.valor_seguro),
            cofins_valor: fmt.money(item.valor_cofins),
            pis_valor: fmt.money(item.valor_pis),
            ipi_valor: fmt.money(item.valor_ipi),
            iss_bc: fmt.money(item.bc_iss),
            iss_valor: fmt.money(item.valor_iss),
            icms_aliquota: fmt.rate(item.aliq_icms),
            icms_bc: fmt.money(item.bc_icms),
            icms_valor: fmt.money(item.valor_icms),
            icms_st_valor: fmt.money(item.valor_icms_st),
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
// TESTES UNITÁRIOS
// =============================================================================

#[cfg(test)]
mod tests_receita {
    use super::*;

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

        let mut sem_nfe = ctes_to_receita(&[cte.clone()], NumberFormat::Native);
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
        let com_nfe = ctes_to_receita(&[cte], NumberFormat::Native);
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

        let cte_sem_nome = InfoCte {
            emitente_cnpj: Some("00.124.733/0016-14".to_string()),
            remetente_cnpj: Some("50.221.019/0097-88".to_string()),
            tomador_codigo: Some(0),
            cte: Some("41260100124733001614570520000008001000008001".to_string()),
            numero_da_nota: Some(800),
            uf_inicio: Some("SP".to_string()),
            mun_inicio: Some("GUARULHOS".to_string()),
            uf_termino: Some("PR".to_string()),
            mun_termino: Some("PONTA GROSSA".to_string()),
            cfop: Some(6352),
            ..Default::default()
        };

        let opts_div = ReceitaOptions::new(NumberFormat::Native, true);
        let res = ctes_to_receita(
            &[
                cte_256.clone(),
                cte_757.clone(),
                cte_255.clone(),
                cte_sem_nome.clone(),
            ],
            opts_div,
        );
        // Preserva os nomes originais encontrados no XML do CT-e 757:
        // "H U ..." (800, 255, 256) ordenam antes de "HU - ..." (757).
        let notas: Vec<&str> = res.iter().map(|r| r.numero_nota.as_str()).collect();
        assert_eq!(notas, ["800", "255", "256", "757"]);

        let row_757 = &res[3];
        assert_eq!(row_757.nome_contribuinte, "HU - TRANSPORTE RODOVIARIO LTDA");
        assert_eq!(
            row_757.nome_participante,
            "HNK BR INDUSTRIA DE BEBIDAS LTDA"
        );
        assert_eq!(row_757.cte_rem_nome_nfe, "HNK BR INDUSTRIA DE BEBIDAS LTDA");
        assert_eq!(row_757.inf_nfe_chave, "*DIVERSOS*");

        // Preenche via votação canônica apenas quando ausente no XML (<N/D>)
        let row_800 = &res[0];
        assert_eq!(row_800.nome_contribuinte, "H U TRANSPORTE RODOVIARIO LTDA");
        assert_eq!(
            row_800.nome_participante,
            "HNK BR INDUSTRIA DE BEBIDAS LTDA."
        );

        let res_all = ctes_to_receita(&[cte_256, cte_757, cte_255], NumberFormat::Native);
        assert_eq!(
            res_all[2].inf_nfe_chave,
            format!("{}, {}", "1".repeat(44), "2".repeat(44))
        );
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
            valor_total_nfe: Some(412.34),
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

        let itens = [item1, item2];
        let res = nfes_to_receita(&itens, NumberFormat::Native);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].numero_item, "0");
        assert_eq!(res[0].codigo_cfop, "*DIVERSOS*");
        assert_eq!(res[0].valor_total, "412.34");
        assert_eq!(res[0].valor_nota_proporcional, "0.0");
        assert_eq!(res[0].icms_st_valor, "0.0");
        assert_eq!(res[1].numero_item, "1");
        assert_eq!(
            res[1].chave_nfe,
            "35250500206861001364550010001321681637262329"
        );
        assert_eq!(res[1].valor_nota_proporcional, "112.34");
        assert_eq!(res[1].icms_st_valor, "12.34");
        assert_eq!(res[2].numero_item, "2");
        assert_eq!(res[2].valor_nota_proporcional, "300.0");

        // O mesmo dataset em formato brasileiro (--format-br).
        let br = nfes_to_receita(&itens, NumberFormat::Brazilian);
        assert_eq!(br[0].valor_total, "412,34");
        assert_eq!(br[0].valor_nota_proporcional, "0,00");
        assert_eq!(br[1].valor_nota_proporcional, "112,34");
        assert_eq!(br[1].icms_st_valor, "12,34");
        assert_eq!(br[2].valor_nota_proporcional, "300,00");
    }

    #[test]
    fn test_number_format_money_e_rate() {
        let nat = NumberFormat::Native;
        let br = NumberFormat::Brazilian;
        assert_eq!(nat, NumberFormat::default());
        assert_eq!(nat, NumberFormat::from(false));
        assert_eq!(br, NumberFormat::from(true));

        assert_eq!(nat.money(1234567.89), "1234567.89");
        assert_eq!(br.money(1234567.89), "1.234.567,89");
        assert_eq!(nat.money(None::<f64>), "0.0");
        assert_eq!(br.money(None::<f64>), "0,00");

        // Alíquotas: 4 casas em ambos; nativo imprime o mínimo necessário.
        assert_eq!(nat.rate(Some(7.6)), "7.6");
        assert_eq!(br.rate(Some(7.6)), "7,6000");
        assert_eq!(nat.rate(0.18), "0.18");
        assert_eq!(br.rate(0.18), "0,1800");
        assert_eq!(nat.rate(0.3456), "0.3456");
        assert_eq!(br.rate(0.3456), "0,3456");
    }

    #[test]
    fn test_native_preserva_inteiros_e_chaves() {
        let nfe = InfoNfe {
            emitente_cnpj: Some("00.206.861/0013-64".to_string()),
            emitente_crt: Some(3),
            nfe: Some("35250500206861001364550010001321681637262329".to_string()),
            numero_da_nota: Some(132168),
            n_item: Some(1),
            cfop: Some(5102),
            ncm: Some("7326.90.90".to_string()),
            valor_total_nfe: Some(400.0),
            valor_total_itens: Some(400.0),
            v_prod: Some(400.0),
            aliq_icms: Some(18.0),
            v_icms: Some(72.0),
            ..Default::default()
        };
        let res = nfes_to_receita(&[nfe], NumberFormat::Native);
        assert_eq!(res.len(), 1);
        let r = &res[0];

        assert_eq!(r.crt, "3");
        assert_eq!(r.numero_nota, "132168");
        assert_eq!(r.numero_item, "1");
        assert_eq!(r.codigo_cfop, "5102");
        assert_eq!(r.codigo_ncm, "73269090");
        assert_eq!(r.chave_nfe, "35250500206861001364550010001321681637262329");

        assert_eq!(r.valor_total, "400.0");
        assert_eq!(r.valor_nota_proporcional, "400.0");
        assert_eq!(r.icms_aliquota, "0.18");
        assert_eq!(r.icms_valor, "72.0");
        assert_eq!(r.valor_descontos, "0.0");
    }

    #[test]
    fn test_item_zero_icms_aliquota_br_e_cteos_modelo() {
        let base = InfoNfe {
            nfe: Some("35260100206861001364550010001321681637262329".to_string()),
            numero_da_nota: Some(1),
            valor_total_nfe: Some(200.0),
            valor_total_itens: Some(200.0),
            ..Default::default()
        };
        let itens = [
            InfoNfe {
                n_item: Some(1),
                v_prod: Some(100.0),
                ..base.clone()
            },
            InfoNfe {
                n_item: Some(2),
                v_prod: Some(100.0),
                ..base
            },
        ];
        let br = nfes_to_receita(&itens, NumberFormat::Brazilian);
        assert_eq!(br[0].numero_item, "0");
        assert_eq!(br[0].icms_aliquota, "0,0000");

        let cte_os = InfoCte {
            cte: Some("35260111222333000144670010000000011000000011".to_string()),
            doc_tipo: "CTeOS".to_string(),
            numero_da_nota: Some(1),
            cfop: Some(5357),
            valor_total: Some(500.0),
            ..Default::default()
        };
        let res_os = ctes_to_receita(&[cte_os], NumberFormat::Native);
        assert_eq!(res_os.len(), 1);
        assert_eq!(res_os[0].registro_origem_item, "CTeOS");
        assert_eq!(
            res_os[0].modelo_descricao,
            "Conhecimento de Transporte Eletrônico para Outros Serviços CT-e OS"
        );
    }

    #[test]
    fn test_format_native_float() {
        assert_eq!(format_native_float(0.0, 2), "0.0");
        assert_eq!(format_native_float(-0.0001, 2), "0.0");
        assert_eq!(format_native_float(3.0, 2), "3.0");
        assert_eq!(format_native_float(123456.78, 2), "123456.78");
        assert_eq!(format_native_float(44556677.1, 2), "44556677.1");
        assert_eq!(format_native_float(0.3456, 4), "0.3456");
        assert_eq!(format_native_float(100.00000000000001, 2), "100.0");
        assert_eq!(format_native_float(-1234.5, 2), "-1234.5");
        assert_eq!(format_native_float(f64::NAN, 2), "0.0");
    }

    #[test]
    fn test_receita_correlacoes_padrao_vs_diversos_cte_e_nfe() {
        let k_cte1 = "35260100130057000105570010000198841285476050";
        let k_cte2 = "35260100130057000105570010000198851285476051";
        let k_nfe1 = "35260102948030000230550010018611891935989130";
        let k_nfe2 = "35260102948030000230550010018611901935989131";

        let cte = InfoCte {
            cte: Some(k_cte1.to_string()),
            cte_complementar: vec![k_cte2.to_string()],
            nfes_vinculados: vec![k_nfe1.to_string()],
            nfes: vec![k_nfe1.to_string(), k_nfe2.to_string()],
            ..Default::default()
        };

        // Padrão (-r sem --diversos): CT-e exibe todas as NF-es e CT-es correlacionados ordenados
        let cte_default = ctes_to_receita(std::slice::from_ref(&cte), NumberFormat::Native);
        assert_eq!(
            cte_default[0].inf_nfe_chave,
            format!("{k_cte2}, {k_nfe1}, {k_nfe2}")
        );

        // Com --diversos: CT-e exibe *DIVERSOS* conforme extrato Receita
        let cte_div = ctes_to_receita(&[cte], ReceitaOptions::new(NumberFormat::Native, true));
        assert_eq!(cte_div[0].inf_nfe_chave, "*DIVERSOS*");

        let nfe = InfoNfe {
            nfe: Some(k_nfe1.to_string()),
            n_item: Some(1),
            ctes: vec![k_cte1.to_string()],
            nfes_relacionadas: vec![k_nfe2.to_string()],
            v_prod: Some(100.0),
            ..Default::default()
        };

        // Padrão (-r sem --diversos): NF-e exibe todas as chaves correlacionadas (CT-es + NF-es)
        let nfe_default = nfes_to_receita(std::slice::from_ref(&nfe), NumberFormat::Native);
        assert_eq!(nfe_default[0].inf_nfe_chave, format!("{k_cte1}, {k_nfe2}"));

        // Com --diversos: NF-e mantém <N/D> idêntico ao 2026-NFe-Destinatario-mes01.csv
        let nfe_div = nfes_to_receita(&[nfe], ReceitaOptions::new(NumberFormat::Native, true));
        assert_eq!(nfe_div[0].inf_nfe_chave, "<N/D>");
    }

    #[test]
    fn test_nfe_vb01_10_com_vdesc_e_voutro_e_icms61() {
        let xml = r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe>
                <infNFe Id="NFe35250121262781000162550010000004331080076504" versao="4.00">
                    <ide><nNF>433</nNF><dhEmi>2025-01-28T13:30:00-03:00</dhEmi><tpNF>1</tpNF></ide>
                    <emit><CNPJ>21262781000162</CNPJ><xNome>EMPRESA MODELO</xNome><CRT>1</CRT></emit>
                    <dest><CNPJ>47964911000100</CNPJ><xNome>DESTINATARIO MODELO</xNome></dest>
                    <det nItem="1">
                        <prod><xProd>ITEM 1</xProd><NCM>16023210</NCM><CFOP>5102</CFOP><vProd>89.00</vProd><vDesc>15.10</vDesc><vOutro>24.10</vOutro></prod>
                        <imposto>
                            <ICMS><ICMS61><orig>0</orig><CST>61</CST><vICMSMonoRet>151.44</vICMSMonoRet></ICMS61></ICMS>
                        </imposto>
                    </det>
                    <det nItem="2">
                        <prod><xProd>ITEM 2</xProd><NCM>20041000</NCM><CFOP>5102</CFOP><vProd>38.00</vProd></prod>
                        <imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto>
                    </det>
                    <det nItem="3">
                        <prod><xProd>ITEM 3</xProd><NCM>22021000</NCM><CFOP>5403</CFOP><vProd>40.00</vProd></prod>
                        <imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto>
                        <vItem>40.00</vItem>
                    </det>
                    <total><ICMSTot><vProd>167.00</vProd><vDesc>15.10</vDesc><vOutro>24.10</vOutro><vNF>176.00</vNF></ICMSTot></total>
                    <transp><modFrete>9</modFrete></transp>
                </infNFe>
            </NFe>
            <protNFe versao="4.00"><infProt><chNFe>35250121262781000162550010000004331080076504</chNFe><cStat>100</cStat></infProt></protNFe>
        </nfeProc>"#;

        let proc: crate::xml_structs::nfe::NfeProc = quick_xml::de::from_str(xml).unwrap();
        let infos = proc.get_infos();
        assert_eq!(infos[0].v_outro, Some(24.10));
        assert_eq!(infos[0].v_item_total, Some(98.00));
        assert_eq!(infos[0].v_icms, None);
        assert_eq!(infos[1].v_item_total, Some(38.00));
        assert_eq!(infos[2].v_item_total, Some(40.00));

        let rec = nfes_to_receita(&infos, NumberFormat::Brazilian);
        assert_eq!(rec[0].numero_item, "0");
        assert_eq!(rec[1].valor_nota_proporcional, "98,00");
        assert_eq!(rec[1].icms_valor, "0,00");
        assert_eq!(rec[2].valor_nota_proporcional, "38,00");
        assert_eq!(rec[3].valor_nota_proporcional, "40,00");
    }

    #[test]
    fn test_legislacao_nov_2026_w16_10_vb01_10_casos_comuns_e_extremos() {
        // Caso 1: XML real 35250121262781000162550010000004331080076504 (5 itens, vDesc e vOutro no item 1)
        // Comprova remoção definitiva do bug do ContÁgil (que distorcia para 81,78 / 42,05 / 44,27)
        let xml_real = r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe><infNFe Id="NFe35250121262781000162550010000004331080076504" versao="4.00">
                <ide><nNF>433</nNF><dhEmi>2025-01-28T13:30:00-03:00</dhEmi><tpNF>1</tpNF></ide>
                <emit><CNPJ>21262781000162</CNPJ><xNome>ADEGA MODELO</xNome><CRT>1</CRT></emit>
                <dest><CNPJ>47964911000100</CNPJ><xNome>DESTINATARIO MODELO</xNome></dest>
                <det nItem="1"><prod><xProd>CONTRA FILE</xProd><NCM>16023210</NCM><CFOP>5102</CFOP><vProd>89.00</vProd><vDesc>15.10</vDesc><vOutro>24.10</vOutro></prod><imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto></det>
                <det nItem="2"><prod><xProd>BATATA PALITO</xProd><NCM>20041000</NCM><CFOP>5102</CFOP><vProd>38.00</vProd></prod><imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto></det>
                <det nItem="3"><prod><xProd>JARRA DE SUCO</xProd><NCM>22021000</NCM><CFOP>5403</CFOP><vProd>40.00</vProd></prod><imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto></det>
                <det nItem="4"><prod><xProd>BROWNIE</xProd><NCM>19059090</NCM><CFOP>5102</CFOP><vProd>38.00</vProd></prod><imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto></det>
                <det nItem="5"><prod><xProd>SUFLE</xProd><NCM>19059090</NCM><CFOP>5102</CFOP><vProd>36.00</vProd></prod><imposto><ICMS><ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102></ICMS></imposto></det>
                <total><ICMSTot><vProd>241.00</vProd><vDesc>15.10</vDesc><vOutro>24.10</vOutro><vNF>250.00</vNF></ICMSTot></total>
                <transp><modFrete>9</modFrete></transp>
            </infNFe></NFe>
            <protNFe versao="4.00"><infProt><chNFe>35250121262781000162550010000004331080076504</chNFe><cStat>100</cStat></infProt></protNFe>
        </nfeProc>"#;

        let proc_real: crate::xml_structs::nfe::NfeProc =
            quick_xml::de::from_str(xml_real).unwrap();
        let infos_real = proc_real.get_infos();
        assert_eq!(
            infos_real
                .iter()
                .map(|i| i.v_item_total.unwrap())
                .collect::<Vec<_>>(),
            vec![98.0, 38.0, 40.0, 38.0, 36.0]
        );
        let rec_real = nfes_to_receita(&infos_real, NumberFormat::Brazilian);
        assert_ne!(rec_real[1].valor_nota_proporcional, "81,78");
        assert_eq!(rec_real[1].valor_nota_proporcional, "98,00");
        assert_eq!(rec_real[2].valor_nota_proporcional, "38,00");
        assert_eq!(rec_real[3].valor_nota_proporcional, "40,00");
        assert_eq!(rec_real[4].valor_nota_proporcional, "38,00");
        assert_eq!(rec_real[5].valor_nota_proporcional, "36,00");

        // Caso 2: Todos os componentes VB01-10 / W16-10 + condicionais desligadas + ICMS15 + det/vItem + IBS/CBS 2026
        let xml_completo = r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe><infNFe Id="NFe35261111222333000144550010000010001000001001" versao="4.00">
                <ide><nNF>1000</nNF><dhEmi>2026-11-20T10:00:00-03:00</dhEmi><tpNF>1</tpNF></ide>
                <emit><CNPJ>11222333000144</CNPJ><xNome>EMPRESA TESTE</xNome><CRT>3</CRT></emit>
                <dest><CNPJ>47964911000100</CNPJ><xNome>DESTINATARIO</xNome></dest>
                <det nItem="1">
                    <prod><xProd>COMPLETO</xProd><NCM>84713019</NCM><CFOP>5102</CFOP><vProd>1000.00</vProd><vDesc>50.00</vDesc><vFrete>40.00</vFrete><vSeg>10.00</vSeg><vOutro>15.00</vOutro></prod>
                    <imposto>
                        <ICMS><ICMS70><orig>0</orig><CST>70</CST><vBC>950.00</vBC><pICMS>18.00</pICMS><vICMS>171.00</vICMS><vBCST>1200.00</vBCST><pICMSST>18.00</pICMSST><vICMSST>120.00</vICMSST><vFCPST>20.00</vFCPST><vICMSDeson>30.00</vICMSDeson><indDeduzDeson>1</indDeduzDeson></ICMS70></ICMS>
                        <II><vBC>1000.00</vBC><vDespAdu>0.00</vDespAdu><vII>80.00</vII><vIOF>0.00</vIOF></II>
                        <IPI><cEnq>999</cEnq><IPITrib><CST>50</CST><vBC>1000.00</vBC><pIPI>5.50</pIPI><vIPI>55.00</vIPI></IPITrib></IPI>
                        <PISST><vBC>1000.00</vBC><pPIS>1.80</pPIS><vPIS>18.00</vPIS><indSomaPISST>1</indSomaPISST></PISST>
                        <COFINSST><vBC>1000.00</vBC><pCOFINS>7.60</pCOFINS><vCOFINS>76.00</vCOFINS><indSomaCOFINSST>1</indSomaCOFINSST></COFINSST>
                    </imposto>
                    <impostoDevol><pDevol>100.00</pDevol><IPI><vIPIDevol>12.00</vIPIDevol></IPI></impostoDevol>
                </det>
                <det nItem="2">
                    <prod><xProd>CONDICIONAIS DESLIGADAS + IBS/CBS 2026</xProd><NCM>04012090</NCM><CFOP>5102</CFOP><vProd>500.00</vProd></prod>
                    <imposto>
                        <ICMS><ICMS40><orig>0</orig><CST>40</CST><vICMSDeson>25.00</vICMSDeson><indDeduzDeson>0</indDeduzDeson></ICMS40></ICMS>
                        <PISST><vBC>500.00</vBC><pPIS>1.80</pPIS><vPIS>9.00</vPIS><indSomaPISST>0</indSomaPISST></PISST>
                        <COFINSST><vBC>500.00</vBC><pCOFINS>7.60</pCOFINS><vCOFINS>40.00</vCOFINS><indSomaCOFINSST>0</indSomaCOFINSST></COFINSST>
                        <IBSCBS><CST>000</CST><gIBSCBS><vBC>500.00</vBC><vIBS>0.50</vIBS><gCBS><pCBS>0.90</pCBS><vCBS>4.50</vCBS></gCBS></gIBSCBS></IBSCBS>
                    </imposto>
                </det>
                <det nItem="3">
                    <prod><xProd>PRECEDENCIA VITEM</xProd><NCM>22021000</NCM><CFOP>5102</CFOP><vProd>200.00</vProd></prod>
                    <imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><vBC>200.00</vBC><pICMS>12.00</pICMS><vICMS>24.00</vICMS></ICMS00></ICMS></imposto>
                    <vItem>210.55</vItem>
                </det>
                <det nItem="4">
                    <prod><xProd>COMBUSTIVEL ICMS15</xProd><NCM>27101921</NCM><CFOP>5655</CFOP><vProd>400.00</vProd></prod>
                    <imposto><ICMS><ICMS15><orig>0</orig><CST>15</CST><vICMSMono>50.00</vICMSMono><vICMSMonoReten>35.00</vICMSMonoReten></ICMS15></ICMS></imposto>
                </det>
                <det nItem="5">
                    <prod><xProd>DESCONTO TOTAL EXTREMO</xProd><NCM>00</NCM><CFOP>5102</CFOP><vProd>50.00</vProd><vDesc>50.00</vDesc></prod>
                    <imposto><ICMS><ICMS40><orig>0</orig><CST>40</CST></ICMS40></ICMS></imposto>
                </det>
                <total><ICMSTot><vProd>2150.00</vProd><vDesc>100.00</vDesc><vICMSDeson>30.00</vICMSDeson><vST>120.00</vST><vFCPST>20.00</vFCPST><vICMSMonoReten>35.00</vICMSMonoReten><vFrete>40.00</vFrete><vSeg>10.00</vSeg><vOutro>15.00</vOutro><vII>80.00</vII><vIPI>55.00</vIPI><vIPIDevol>12.00</vIPIDevol><vNF>2511.55</vNF></ICMSTot></total>
                <transp><modFrete>9</modFrete></transp>
            </infNFe></NFe>
            <protNFe versao="4.00"><infProt><chNFe>35261111222333000144550010000010001000001001</chNFe><cStat>100</cStat></infProt></protNFe>
        </nfeProc>"#;

        let proc_comp: crate::xml_structs::nfe::NfeProc =
            quick_xml::de::from_str(xml_completo).unwrap();
        let infos_comp = proc_comp.get_infos();
        assert_eq!(infos_comp[0].v_item_total, Some(1366.00));
        assert_eq!(infos_comp[1].v_item_total, Some(500.00));
        assert_eq!(infos_comp[2].v_item_total, Some(210.55));
        assert_eq!(infos_comp[3].v_icms, Some(50.00));
        assert_eq!(infos_comp[3].v_icms_st, Some(35.00));
        assert_eq!(infos_comp[3].v_item_total, Some(435.00));
        assert_eq!(infos_comp[4].v_item_total, Some(0.00));

        let rec_comp = nfes_to_receita(&infos_comp, NumberFormat::Brazilian);
        assert_eq!(rec_comp[0].valor_total, "2.511,55");
        assert_eq!(rec_comp[1].valor_nota_proporcional, "1.366,00");
        assert_eq!(rec_comp[2].valor_nota_proporcional, "500,00");
        assert_eq!(rec_comp[3].valor_nota_proporcional, "210,55");
        assert_eq!(rec_comp[4].valor_nota_proporcional, "435,00");
        assert_eq!(rec_comp[5].valor_nota_proporcional, "0,00");

        // Caso 3: Natureza da Base de Cálculo estrita (não infere crédito em CST 70/01)
        assert_eq!(crate::lookup_natureza_bc_desc("1662", "70"), None);
        assert_eq!(crate::lookup_natureza_bc_desc("1662", "01"), None);
        assert_eq!(
            crate::lookup_natureza_bc_desc("1662", "50"),
            Some("Devolução de Vendas Sujeitas à Incidência Não-Cumulativa")
        );

        // Caso 4: CT-e com ICMS60 (vICMSSTRet) capturado em v_icms_st
        let cte_xml = r#"<cteProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte">
            <CTe><infCte Id="CTe13260118805360000207570010000065211912793160" versao="4.00">
                <ide><CFOP>5352</CFOP><nCT>6521</nCT><dhEmi>2026-01-10T10:00:00-03:00</dhEmi><toma3><toma>3</toma></toma3></ide>
                <emit><CNPJ>18805360000207</CNPJ><xNome>TRANS MODELO</xNome><CRT>3</CRT></emit>
                <vPrest><vTPrest>500.00</vTPrest><vRec>500.00</vRec></vPrest>
                <imp><ICMS><ICMS60><CST>60</CST><vBCSTRet>658.67</vBCSTRet><vICMSSTRet>79.04</vICMSSTRet><pICMSSTRet>12.00</pICMSSTRet></ICMS60></ICMS></imp>
            </infCte></CTe>
            <protCTe versao="4.00"><infProt><chCTe>13260118805360000207570010000065211912793160</chCTe><cStat>100</cStat></infProt></protCTe>
        </cteProc>"#;
        let cte_proc: crate::xml_structs::cte::CteProc = quick_xml::de::from_str(cte_xml).unwrap();
        let info_cte = cte_proc.get_info();
        assert_eq!(info_cte.v_icms_st, Some(79.04));
        let rec_cte = ctes_to_receita(std::slice::from_ref(&info_cte), NumberFormat::Brazilian);
        assert_eq!(rec_cte[0].icms_st_valor, "79,04");

        // Caso 5: Fallback quando XML omite soma nos itens (vNF difere da soma VB01-10)
        let base_ext = InfoNfe {
            nfe: Some("35261199999999000199550010000000011000000001".to_string()),
            numero_da_nota: Some(1),
            valor_total_nfe: Some(120.0),
            valor_total_itens: Some(100.0),
            ..Default::default()
        };
        let itens_ext = [
            InfoNfe {
                n_item: Some(1),
                v_prod: Some(60.0),
                v_item_total: Some(60.0),
                ..base_ext.clone()
            },
            InfoNfe {
                n_item: Some(2),
                v_prod: Some(40.0),
                v_item_total: Some(40.0),
                ..base_ext
            },
        ];
        let rec_ext = nfes_to_receita(&itens_ext, NumberFormat::Brazilian);
        assert_eq!(rec_ext[1].valor_nota_proporcional, "72,00");
        assert_eq!(rec_ext[2].valor_nota_proporcional, "48,00");
    }

    #[test]
    fn test_regressao_contagil_vs_xml_tetra_pak_icms_outra_uf_e_mandamentos() {
        // 1. Caso real Tetra Pak (35250161528030000160550010011719411291219973):
        //    vIPI exclusivo no item 2 + vDesc + vOutro -> bloqueia rateio ContÁgil
        let xml_tp = r#"<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe><infNFe Id="NFe35250161528030000160550010011719411291219973" versao="4.00">
                <ide><nNF>1171941</nNF><dhEmi>2025-01-10T10:00:00-03:00</dhEmi><tpNF>1</tpNF></ide>
                <emit><CNPJ>61528030000160</CNPJ><xNome>Tetra Pak Ltda</xNome><CRT>3</CRT></emit>
                <dest><CNPJ>47964911000100</CNPJ><xNome>USINA DE LATICINIOS JUSSARA SA</xNome></dest>
                <det nItem="1"><prod><xProd>ITEM 1</xProd><NCM>48115923</NCM><CFOP>5101</CFOP><vProd>79428.89</vProd><vDesc>13344.06</vDesc><vOutro>1119.85</vOutro></prod><imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><vBC>67204.68</vBC><pICMS>18.00</pICMS><vICMS>12096.84</vICMS></ICMS00></ICMS></imposto></det>
                <det nItem="2"><prod><xProd>ITEM 2</xProd><NCM>48115923</NCM><CFOP>5101</CFOP><vProd>79428.89</vProd><vDesc>13344.06</vDesc><vOutro>1119.84</vOutro></prod><imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><vBC>67204.67</vBC><pICMS>18.00</pICMS><vICMS>12096.84</vICMS></ICMS00></ICMS><IPI><cEnq>999</cEnq><IPITrib><CST>50</CST><vBC>67204.67</vBC><pIPI>3.25</pIPI><vIPI>2184.15</vIPI></IPITrib></IPI></imposto></det>
                <det nItem="3"><prod><xProd>ITEM 3</xProd><NCM>48115923</NCM><CFOP>5101</CFOP><vProd>39714.45</vProd><vDesc>6672.03</vDesc><vOutro>1119.84</vOutro></prod><imposto><ICMS><ICMS00><orig>0</orig><CST>00</CST><vBC>34162.26</vBC><pICMS>18.00</pICMS><vICMS>6149.21</vICMS></ICMS00></ICMS></imposto></det>
                <total><ICMSTot><vProd>198572.23</vProd><vDesc>33360.15</vDesc><vIPI>2184.15</vIPI><vOutro>3359.53</vOutro><vNF>170755.76</vNF></ICMSTot></total>
                <transp><modFrete>9</modFrete></transp>
            </infNFe></NFe>
            <protNFe versao="4.00"><infProt><chNFe>35250161528030000160550010011719411291219973</chNFe><cStat>100</cStat></infProt></protNFe>
        </nfeProc>"#;

        let proc_tp: crate::xml_structs::nfe::NfeProc = quick_xml::de::from_str(xml_tp).unwrap();
        let rec_tp = nfes_to_receita(&proc_tp.get_infos(), NumberFormat::Brazilian);
        assert_eq!(rec_tp[0].valor_total, "170.755,76");
        assert_eq!(rec_tp[1].valor_nota_proporcional, "67.204,68");
        assert_eq!(rec_tp[2].valor_nota_proporcional, "69.388,82");
        assert_eq!(rec_tp[3].valor_nota_proporcional, "34.162,26");
        // Bloqueia regressão ao bug do ContÁgil (que distorcia para 67.411,11 / 69.639,09 / 33.705,56)
        assert_ne!(rec_tp[1].valor_nota_proporcional, "67.411,11");
        assert_ne!(rec_tp[2].valor_nota_proporcional, "69.639,09");

        // 2. CT-e com ICMSOutraUF + toma4 (Mandamento 1 e 2: não zerar ICMSOutraUF como o ContÁgil)
        let cte_outra_uf = r#"<cteProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte">
            <CTe><infCte Id="CTe25260107189259001909570030003777801014957634" versao="4.00">
                <ide><CFOP>6352</CFOP><nCT>377780</nCT><dhEmi>2026-01-12T08:00:00-03:00</dhEmi><UFIni>RN</UFIni><UFFim>PB</UFFim><toma4><toma>4</toma><CNPJ>47960950075725</CNPJ><xNome>TOMADOR OUTROS</xNome></toma4></ide>
                <emit><CNPJ>07189259001909</CNPJ><xNome>L AUTO CARGO</xNome><CRT>3</CRT></emit>
                <rem><CNPJ>47960950000121</CNPJ><xNome>MAGAZINE LUIZA S/A</xNome></rem>
                <dest><CNPJ>47960950075725</CNPJ><xNome>MAGAZINE LUIZA S/A</xNome></dest>
                <vPrest><vTPrest>11.36</vTPrest><vRec>11.36</vRec></vPrest>
                <imp><ICMS><ICMSOutraUF><CST>90</CST><vBCOutraUF>11.36</vBCOutraUF><pICMSOutraUF>12.00</pICMSOutraUF><vICMSOutraUF>1.36</vICMSOutraUF></ICMSOutraUF></ICMS></imp>
            </infCte></CTe>
            <protCTe versao="4.00"><infProt><chCTe>25260107189259001909570030003777801014957634</chCTe><cStat>100</cStat></infProt></protCTe>
        </cteProc>"#;
        let proc_outra: crate::xml_structs::cte::CteProc =
            quick_xml::de::from_str(cte_outra_uf).unwrap();
        let rec_outra = ctes_to_receita(
            std::slice::from_ref(&proc_outra.get_info()),
            NumberFormat::Brazilian,
        );
        assert_eq!(rec_outra[0].icms_bc, "11,36");
        assert_eq!(rec_outra[0].icms_aliquota, "0,1200");
        assert_eq!(rec_outra[0].icms_valor, "1,36");
        assert_eq!(rec_outra[0].cte_papel_tomador_comp, "Outros");
        assert_eq!(rec_outra[0].cte_outro_tomador_comp, "47.960.950/0757-25");

        // 3. CT-e ICMS60 sem <infNFe>: preserva vICMSSTRet, Papel e xObs (não omite como ContÁgil)
        let cte_icms60 = r#"<cteProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/cte">
            <CTe><infCte Id="CTe15260100657565000308570000000258411510044076" versao="4.00">
                <ide><CFOP>6352</CFOP><nCT>25841</nCT><dhEmi>2026-01-15T10:00:00-03:00</dhEmi><UFIni>PA</UFIni><UFFim>SP</UFFim><toma3><toma>0</toma></toma3></ide>
                <compl><xObs>REGIME TRIBUTARIO DIFERENCIADO</xObs></compl>
                <emit><CNPJ>00657565000308</CNPJ><xNome>GAB TRANSPORTES LTDA</xNome><CRT>3</CRT></emit>
                <rem><CNPJ>00657565000227</CNPJ><xNome>GAB MANAUS</xNome></rem>
                <dest><CNPJ>47960950000121</CNPJ><xNome>MAGAZINE LUIZA SA</xNome></dest>
                <vPrest><vTPrest>17561.32</vTPrest><vRec>17561.32</vRec></vPrest>
                <imp><ICMS><ICMS60><CST>60</CST><vBCSTRet>17561.32</vBCSTRet><vICMSSTRet>2107.36</vICMSSTRet><pICMSSTRet>12.00</pICMSSTRet></ICMS60></ICMS></imp>
            </infCte></CTe>
            <protCTe versao="4.00"><infProt><chCTe>15260100657565000308570000000258411510044076</chCTe><cStat>100</cStat></infProt></protCTe>
        </cteProc>"#;
        let proc_60: crate::xml_structs::cte::CteProc =
            quick_xml::de::from_str(cte_icms60).unwrap();
        let rec_60 = ctes_to_receita(
            std::slice::from_ref(&proc_60.get_info()),
            ReceitaOptions::new(NumberFormat::Brazilian, true),
        );
        assert_eq!(rec_60[0].icms_st_valor, "2.107,36");
        assert_eq!(rec_60[0].cte_papel_tomador_comp, "Remetente");
        assert_eq!(rec_60[0].cte_obs_gerais, "REGIME TRIBUTARIO DIFERENCIADO");
        assert_eq!(rec_60[0].inf_nfe_chave, "<N/D>");

        // 4. Mandamento 3: nunca sobrescrever nomes históricos presentes no XML
        let cte_a = InfoCte {
            cte: Some("15260110889552000580570030002692251005954000".to_string()),
            emitente_cnpj: Some("10.889.552/0005-80".to_string()),
            emitente_nome: Some("NORTE BRASIL LOGISTICA LTDA".to_string()),
            ..Default::default()
        };
        let cte_b = InfoCte {
            cte: Some("15260110889552000580570030002692261005954001".to_string()),
            emitente_cnpj: Some("10.889.552/0005-80".to_string()),
            emitente_nome: Some("NORTE BRASIL LOGISTICA EIRELI".to_string()),
            ..Default::default()
        };
        let rec_nomes = ctes_to_receita(&[cte_a, cte_b], NumberFormat::Native);
        let nomes: Vec<&str> = rec_nomes
            .iter()
            .map(|r| r.nome_contribuinte.as_str())
            .collect();
        assert!(nomes.contains(&"NORTE BRASIL LOGISTICA LTDA"));
        assert!(nomes.contains(&"NORTE BRASIL LOGISTICA EIRELI"));
    }
}
