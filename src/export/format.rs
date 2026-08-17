//! # Módulo de Formatação e Estilos de Célula do Excel
//!
//! Este módulo gerencia os formatos visuais, fontes, estilos de alinhamento
//! e estratégias de consumo de memória para as planilhas criadas no sistema.
//! Ele centraliza a validação de cabeçalhos por meio de expressões regulares.

use rust_xlsxwriter::{Format, FormatAlign, Worksheet};
use std::collections::HashMap;

use crate::{FORMAT_RULES, REGEX_COLUMN_SET, XmlParserResult};

/// Tamanho padrão da fonte para as células de dados.
pub const FONT_SIZE: f64 = 14.0;
/// Tamanho padrão da fonte para as células da linha de cabeçalho.
pub const HEADER_FONT_SIZE: f64 = 12.0;

/// Gerenciador unificado de estilização e aplicação de formatos do Excel.
///
/// Encapsula e expõe formatos pré-configurados e métodos para estilizar colunas
/// dinamicamente de acordo com as expressões regulares definidas no projeto.
#[derive(Debug, Clone)]
pub struct ExcelFormatter {
    formats: HashMap<&'static str, Format>,
}

impl Default for ExcelFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExcelFormatter {
    /// Inicializa a coleção de formatos padrão utilizados nas planilhas.
    pub fn new() -> Self {
        let fmt_header = Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_text_wrap()
            .set_font_size(HEADER_FONT_SIZE);

        let fmt_center = Format::new()
            .set_align(FormatAlign::Center)
            .set_font_size(FONT_SIZE);

        let fmt_value = Format::new()
            .set_num_format("#,##0.00")
            .set_font_size(FONT_SIZE);

        let fmt_aliq = Format::new()
            .set_num_format("#,##0.0000")
            .set_font_size(FONT_SIZE);

        let fmt_date = Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_num_format("dd/mm/yyyy")
            .set_font_size(FONT_SIZE);

        let fmt_default = Format::new().set_font_size(FONT_SIZE);

        let formats = HashMap::from([
            ("header", fmt_header),
            ("center", fmt_center),
            ("value", fmt_value),
            ("aliq", fmt_aliq),
            ("date", fmt_date),
            ("default", fmt_default),
        ]);

        Self { formats }
    }

    /// Retorna uma referência segura ao formato associado à linha de cabeçalho.
    pub fn header_format(&self) -> Option<&Format> {
        self.formats.get("header")
    }

    /// Varre a listagem de colunas de uma planilha aplicando as regras usando o RegexSet unificado.
    pub fn apply_column_formats(
        &self,
        worksheet: &mut Worksheet,
        column_names: &[&str],
    ) -> XmlParserResult<()> {
        let default_fmt = self.formats.get("default").ok_or_else(|| {
            rust_xlsxwriter::XlsxError::ParameterError(
                "Default format registration is missing".to_string(),
            )
        })?;

        for (index, col_name) in column_names.iter().enumerate() {
            let column_number: u16 = index.try_into()?;

            // Determina o formato associado consultando a regra na mesma posição do índice retornado
            let selected_format = REGEX_COLUMN_SET
                .matches(col_name)
                .iter()
                .next()
                .and_then(|matched_index| {
                    // Obtém a regra correspondente de forma segura (O(1))
                    let rule = FORMAT_RULES.get(matched_index)?;
                    // Recupera o formato usando a chave de string registrada na regra
                    self.formats.get(rule.key)
                })
                // Se nenhuma regra regex coincidir com o nome da coluna,
                // aplica explicitamente o tamanho 14 padrão nesta coluna.
                .unwrap_or(default_fmt);

            worksheet.set_column_format(column_number, selected_format)?;
        }

        Ok(())
    }
}
