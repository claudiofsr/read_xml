//! # Tabela Oficial 4.3.7 — Natureza da Base de Cálculo dos Créditos (EFD-Contribuições)
//!
//! Fontes oficiais vigentes (novembro de 2026):
//! - Portal SPED / Receita Federal: Tabela 4.3.7 (Código de Base de Cálculo do Crédito) v1.0.2.
//! - Portal SPED / Receita Federal: Tabela CFOP - Operações Geradoras de Créditos v1.10 (`rust-sped/tabelas.rs`).

const NAT_01: &str = "Aquisição de Bens para Revenda";
const NAT_02: &str = "Aquisição de Bens Utilizados como Insumo";
const NAT_03: &str = "Aquisição de Serviços Utilizados como Insumo";
const NAT_04: &str = "Energia Elétrica e Térmica, Inclusive sob a Forma de Vapor";
const NAT_05: &str = "Aluguéis de Prédios";
const NAT_06: &str = "Aluguéis de Máquinas e Equipamentos";
const NAT_07: &str = "Armazenagem de Mercadoria e Frete na Operação de Venda";
const NAT_08: &str = "Contraprestações de Arrendamento Mercantil";
const NAT_09: &str =
    "Máquinas, Equipamentos e Outros Bens Incorporados ao Ativo Imobilizado (Depreciação)";
const NAT_10: &str =
    "Máquinas, Equipamentos e Outros Bens Incorporados ao Ativo Imobilizado (Valor de Aquisição)";
const NAT_11: &str = "Amortização e Depreciação de Edificações e Benfeitorias em Imóveis";
const NAT_12: &str = "Devolução de Vendas Sujeitas à Incidência Não-Cumulativa";
const NAT_13: &str = "Outras Operações com Direito a Crédito";
const NAT_14: &str = "Atividade de Transporte de Cargas - Subcontratação";
const NAT_15: &str = "Atividade Imobiliária - Custo Incorrido de Unidade Imobiliária";
const NAT_16: &str = "Atividade Imobiliária - Custo Orçado de Unidade Não Concluída";
const NAT_17: &str = "Atividade de Prestação de Serviços de Limpeza, Conservação e Manutenção";
const NAT_18: &str = "Estoque de Abertura de Bens";

/// 18 códigos oficiais da Tabela 4.3.7 da EFD-Contribuições ordenados para busca binária.
static NATUREZA_BC_TABLE: &[(u8, &str)] = &[
    (1, NAT_01),
    (2, NAT_02),
    (3, NAT_03),
    (4, NAT_04),
    (5, NAT_05),
    (6, NAT_06),
    (7, NAT_07),
    (8, NAT_08),
    (9, NAT_09),
    (10, NAT_10),
    (11, NAT_11),
    (12, NAT_12),
    (13, NAT_13),
    (14, NAT_14),
    (15, NAT_15),
    (16, NAT_16),
    (17, NAT_17),
    (18, NAT_18),
];

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

/// Busca binária O(log N) pela descrição da Natureza da Base de Cálculo pelo código (1..=18).
pub fn lookup_natureza_bc_por_codigo(codigo: u8) -> Option<&'static str> {
    match NATUREZA_BC_TABLE.binary_search_by_key(&codigo, |&(k, _)| k) {
        Ok(idx) => Some(NATUREZA_BC_TABLE[idx].1),
        Err(_) => None,
    }
}

/// 89 códigos CFOP geradores de crédito ordenados para busca binária (`rust-sped/tabelas.rs`).
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

#[cfg(test)]
mod tests_natureza {
    use super::*;

    #[test]
    fn test_tables_are_strictly_sorted_for_binary_search() {
        assert!(
            NATUREZA_BC_TABLE.windows(2).all(|w| w[0].0 < w[1].0),
            "NATUREZA_BC_TABLE deve estar estritamente ordenada"
        );
        assert!(
            CFOP_NATUREZA_BC_TABLE.windows(2).all(|w| w[0].0 < w[1].0),
            "CFOP_NATUREZA_BC_TABLE deve estar estritamente ordenada"
        );
    }

    #[test]
    fn test_lookup_natureza_bc() {
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
        assert_eq!(
            lookup_natureza_bc_por_codigo(14),
            Some("Atividade de Transporte de Cargas - Subcontratação")
        );
    }
}
