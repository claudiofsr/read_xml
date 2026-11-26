//! # Mapeamento do Grupo de Cobrança da NF-e e CT-e
//!
//! Este módulo gerencia o parse, a desserialização e a análise de dados financeiros
//! referentes ao grupo de cobranças (`<cobr>`), detalhando faturas (`<fat>`) e
//! suas respectivas parcelas representadas por duplicatas (`<dup>`).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::OptExt;

/// Informações de Cobrança da Nota Fiscal Eletrônica (`<cobr>`).
///
/// Agrupa os dados de faturamento global do documento fiscal e as parcelas
/// programadas para vencimento futuro.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cobranca {
    /// Conteúdo textual bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista contendo as parcelas/duplicatas de pagamento programadas.
    #[serde(rename = "dup", default)]
    pub dup: Option<Vec<Dup>>,

    /// Dados consolidados da fatura (valor original, desconto, líquido, etc.).
    #[serde(rename = "fat", default)]
    pub fat: Option<Fat>,
}

impl Cobranca {
    /// Retorna uma fatia (slice) contendo todas as duplicatas de forma idiomática e segura.
    ///
    /// Evita alocações adicionais na Heap, retornando uma lista vazia caso
    /// o elemento `<dup>` esteja ausente no XML.
    #[inline]
    pub fn get_duplicatas(&self) -> &[Dup] {
        self.dup.as_deref().unwrap_or(&[])
    }

    /// Retorna uma referência segura para os dados da fatura consolidada se estiver presente.
    #[inline]
    pub fn get_fatura(&self) -> Option<&Fat> {
        self.fat.as_ref()
    }
}

/// Dados da Duplicata de Cobrança (`<dup>`).
///
/// Representa uma parcela de pagamento individualizada contendo o identificador,
/// a data programada para o vencimento e o valor monetário acordado.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dup {
    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Data de vencimento da parcela (formato original string: `AAAA-MM-DD`).
    #[serde(rename = "dVenc", default)]
    pub d_venc: Option<String>,

    /// Número identificador sequencial da duplicata.
    #[serde(rename = "nDup", default)]
    pub n_dup: Option<String>,

    /// Valor bruto da duplicata (formato original string).
    #[serde(rename = "vDup", default)]
    pub v_dup: Option<String>,
}

impl Dup {
    /// Tenta realizar o parse da data de vencimento original para `chrono::NaiveDate`.
    ///
    /// Retorna `None` se a tag correspondente estiver vazia ou for inválida.
    pub fn get_data_vencimento(&self) -> Option<NaiveDate> {
        crate::get_naive_date_from_yyyy_mm_dd(&self.d_venc)
    }

    /// Tenta converter o valor original em string da duplicata para ponto flutuante `f64`.
    pub fn get_valor_duplicata(&self) -> Option<f64> {
        self.v_dup.to_float64()
    }
}

/// Dados Consolidados da Fatura Comercial (`<fat>`).
///
/// Detalha os totais financeiros de fechamento do faturamento, incluindo valores
/// brutos originais, descontos concedidos e o montante líquido final.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fat {
    /// Conteúdo descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Número identificador sequencial correspondente à fatura.
    #[serde(rename = "nFat", default)]
    pub n_fat: Option<String>,

    /// Desconto global concedido sobre o montante da fatura (formato original string).
    #[serde(rename = "vDesc", default)]
    pub v_desc: Option<String>,

    /// Valor líquido consolidado faturado (formato original string).
    #[serde(rename = "vLiq", default)]
    pub v_liq: Option<String>,

    /// Valor original bruto consolidado faturado (formato original string).
    #[serde(rename = "vOrig", default)]
    pub v_orig: Option<String>,
}

impl Fat {
    /// Tenta converter o valor do desconto original em string para `f64`.
    pub fn get_valor_desconto(&self) -> Option<f64> {
        self.v_desc.to_float64()
    }

    /// Tenta converter o valor líquido consolidado em string para `f64`.
    pub fn get_valor_liquido(&self) -> Option<f64> {
        self.v_liq.to_float64()
    }

    /// Tenta converter o valor bruto original em string para `f64`.
    pub fn get_valor_original(&self) -> Option<f64> {
        self.v_orig.to_float64()
    }
}
