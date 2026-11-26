//! # Tabela Oficial 4.3.3 / 4.3.4 — CST de PIS/PASEP e COFINS (EFD-Contribuições)
//!
//! Fontes oficiais vigentes (novembro de 2026):
//! - Portal SPED / Receita Federal: Tabela 4.3.3 (CST-PIS) e Tabela 4.3.4 (CST-COFINS) v1.0.0.
//! - Mapeamento sintético utilizado nos extratos da Receita Federal (`2026-*-*.csv`).

use crate::RECEITA_ND;

const ND: &str = RECEITA_ND;

/// Retorna a descrição padrão do extrato da Receita Federal para o CST de PIS/COFINS.
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
            98 | 99 => "OUTRAS OPERAÇÕES",
            _ => ND,
        }
    }
}

/// Busca binária O(log N) pela descrição oficial completa da Tabela 4.3.3 / 4.3.4 da RFB.
pub fn lookup_cst_oficial_desc(cst: &str) -> &'static str {
    let Ok(code) = cst.trim().parse::<u8>() else {
        return ND;
    };

    match CST_OFICIAL_TABLE.binary_search_by_key(&code, |&(k, _)| k) {
        Ok(idx) => CST_OFICIAL_TABLE[idx].1,
        Err(_) => ND,
    }
}

/// 33 códigos oficiais da Tabela 4.3.3 / 4.3.4 (CST PIS/COFINS) ordenados para busca binária.
static CST_OFICIAL_TABLE: &[(u8, &str)] = &[
    (1, "Operação Tributável com Alíquota Básica"),
    (2, "Operação Tributável com Alíquota Diferenciada"),
    (
        3,
        "Operação Tributável com Alíquota por Unidade de Medida de Produto",
    ),
    (
        4,
        "Operação Tributável Monofásica - Revenda a Alíquota Zero",
    ),
    (5, "Operação Tributável por Substituição Tributária"),
    (6, "Operação Tributável a Alíquota Zero"),
    (7, "Operação Isenta da Contribuição"),
    (8, "Operação sem Incidência da Contribuição"),
    (9, "Operação com Suspensão da Contribuição"),
    (49, "Outras Operações de Saída"),
    (
        50,
        "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Tributada no Mercado Interno",
    ),
    (
        51,
        "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno",
    ),
    (
        52,
        "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita de Exportação",
    ),
    (
        53,
        "Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno",
    ),
    (
        54,
        "Operação com Direito a Crédito - Vinculada a Receitas Tributadas no Mercado Interno e de Exportação",
    ),
    (
        55,
        "Operação com Direito a Crédito - Vinculada a Receitas Não-Tributadas no Mercado Interno e de Exportação",
    ),
    (
        56,
        "Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação",
    ),
    (
        60,
        "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Tributada no Mercado Interno",
    ),
    (
        61,
        "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno",
    ),
    (
        62,
        "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita de Exportação",
    ),
    (
        63,
        "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno",
    ),
    (
        64,
        "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas no Mercado Interno e de Exportação",
    ),
    (
        65,
        "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Não-Tributadas no Mercado Interno e de Exportação",
    ),
    (
        66,
        "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação",
    ),
    (67, "Crédito Presumido - Outras Operações"),
    (70, "Operação de Aquisição sem Direito a Crédito"),
    (71, "Operação de Aquisição com Isenção"),
    (72, "Operação de Aquisição com Suspensão"),
    (73, "Operação de Aquisição a Alíquota Zero"),
    (74, "Operação de Aquisição sem Incidência da Contribuição"),
    (75, "Operação de Aquisição por Substituição Tributária"),
    (98, "Outras Operações de Entrada"),
    (99, "Outras Operações"),
];

#[cfg(test)]
mod tests_cst {
    use super::*;

    #[test]
    fn test_cst_oficial_table_is_strictly_sorted() {
        assert!(
            CST_OFICIAL_TABLE.windows(2).all(|w| w[0].0 < w[1].0),
            "CST_OFICIAL_TABLE deve estar estritamente ordenada"
        );
    }

    #[test]
    fn test_lookup_cst_pis_cofins_desc() {
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
            lookup_cst_oficial_desc("50"),
            "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Tributada no Mercado Interno"
        );
        assert_eq!(lookup_cst_oficial_desc("00"), "<N/D>");
    }
}
