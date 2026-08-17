//! # Mapeamento de Informações de Pagamento da NF-e
//!
//! Este módulo gerencia a desserialização e manipulação dos dados de pagamento
//! de Notas Fiscais Eletrônicas (`<pag>`).
//!
//! Ele centraliza as formas de pagamento utilizadas, valores de parcelas ou
//! pagamentos à vista, troco devolvido e detalhes de transações com cartões
//! de crédito ou débito (se aplicável).

use serde::{Deserialize, Serialize};

/// Informações de Pagamento da Nota Fiscal Eletrônica (`<pag>`).
///
/// Contém o troco devolvido ao cliente e a listagem com as formas de pagamento
/// empregadas na respectiva transação comercial.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pagamento {
    /// Valor monetário correspondente ao troco devolvido (vTroco).
    #[serde(rename = "vTroco", default)]
    pub v_troco: Option<String>,

    /// Lista contendo os detalhes das formas de pagamento empregadas.
    #[serde(rename = "detPag", default)]
    pub det_pag: Vec<DetalhesPagamento>,

    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl Pagamento {
    /// Tenta converter e extrair de forma segura o valor do troco como `f64`.
    ///
    /// Trata espaços em branco e substitui separadores decimais do padrão brasileiro.
    pub fn troco(&self) -> Option<f64> {
        self.v_troco
            .as_ref()
            .and_then(|v| v.trim().replace(',', ".").parse::<f64>().ok())
    }

    /// Consolida e calcula o somatório de todos os pagamentos realizados listados no documento.
    ///
    /// Utiliza iteradores para garantir segurança e eficiência na acumulação de floats.
    pub fn valor_total_pago(&self) -> f64 {
        self.det_pag.iter().filter_map(|det| det.valor()).sum()
    }
}

/// Detalhes de uma forma de pagamento empregada (`<detPag>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetalhesPagamento {
    /// Indicador da forma de pagamento: 0 - Pagamento à Vista; 1 - Pagamento a Prazo.
    #[serde(rename = "indPag", default)]
    pub ind_pag: Option<String>,

    /// Meio de pagamento empregado (tPag) de acordo com a tabela da SEFAZ (p. ex., "01" - Dinheiro).
    #[serde(rename = "tPag", default)]
    pub t_pag: Option<String>,

    /// Descrição literal do meio de pagamento caso seja selecionada a opção "99" (Outros).
    #[serde(rename = "xPag", default)]
    pub x_pag: Option<String>,

    /// Valor monetário correspondente ao pagamento realizado por este meio específico (vPag).
    #[serde(rename = "vPag", default)]
    pub v_pag: Option<String>,

    /// Data em que o pagamento foi realizado (dPag).
    #[serde(rename = "dPag", default)]
    pub d_pag: Option<String>,

    /// CNPJ da instituição de pagamento vinculada (CNPJPag).
    #[serde(rename = "CNPJPag", default)]
    pub cnpj_pag: Option<String>,

    /// Sigla do Estado (UF) da instituição de pagamento vinculada (UFPag).
    #[serde(rename = "UFPag", default)]
    pub uf_pag: Option<String>,

    /// Dados da transação de cartão se o meio de pagamento for integrado à rede.
    #[serde(rename = "card", default)]
    pub card: Option<Card>,

    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl DetalhesPagamento {
    /// Tenta converter e extrair o valor deste pagamento específico como um `f64` válido.
    pub fn valor(&self) -> Option<f64> {
        self.v_pag
            .as_ref()
            .and_then(|v| v.trim().replace(',', ".").parse::<f64>().ok())
    }

    /// Tenta extrair o indicador de pagamento como um valor numérico `u8`.
    pub fn indicador_pagamento(&self) -> Option<u8> {
        self.ind_pag
            .as_ref()
            .and_then(|v| v.trim().parse::<u8>().ok())
    }
}

/// Dados do cartão de crédito / débito envolvido na transação (`<card>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    /// Tipo de integração para o processo de pagamento:
    ///
    /// * `1` - Integração eletrônica (TEF);
    /// * `2` - Pagamento não integrado (POS).
    #[serde(rename = "tpIntegra", default)]
    pub tp_integra: Option<String>,

    /// CNPJ da instituição credenciadora da transação de cartão.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Código identificador correspondente à bandeira operadora do cartão.
    #[serde(rename = "tBand", default)]
    pub t_band: Option<String>,

    /// Código de autorização gerado pela rede emissora do cartão.
    #[serde(rename = "cAut", default)]
    pub c_aut: Option<String>,

    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl Card {
    /// Retorna uma referência limpa do CNPJ da credenciadora do cartão, caso exista.
    pub fn get_cnpj(&self) -> Option<&str> {
        self.cnpj.as_ref().map(|c| c.trim())
    }

    /// Tenta extrair o tipo de integração do processo como um valor numérico `u8`.
    pub fn tipo_integracao(&self) -> Option<u8> {
        self.tp_integra
            .as_ref()
            .and_then(|v| v.trim().parse::<u8>().ok())
    }
}
