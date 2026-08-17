//! # Módulo de Geração de Planilhas Excel
//!
//! Este módulo gerencia a exportação de coleções de structs genéricas
//! para arquivos Excel de acordo com as estratégias de consumo de memória selecionadas.

use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use rayon::prelude::*;
use rust_xlsxwriter::{DocProperties, Table, TableStyle, Workbook, Worksheet};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use std::collections::HashMap;
use std::path::Path;
use struct_iterable::Iterable;

use crate::{ExcelFormatter, ExcelMemoryMode, XmlParserError, XmlParserResult};

const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

/// Atalho para correspondência de tipos genéricos em tempo de execução via downcast.
macro_rules! match_cast {
    ($any:ident { $( $bind:ident as $patt:ty => $body:block , )+ }) => {{
        let downcast = || {
            $(
            if let Some($bind) = $any.downcast_ref::<$patt>() {
                return $body;
            }
            )+
            None
        };
        downcast()
    }};
}

/// Extensão para obter os metadados de serialização de cabeçalhos das estruturas.
pub trait InfoExtension {
    fn get_headers<'de>() -> &'static [&'static str]
    where
        Self: Deserialize<'de>,
    {
        serde_introspect::<Self>()
    }
}

/// Escreve o arquivo XLSX com suporte para diferentes estratégias de consumo de memória.
///
/// Acumula e retorna mensagens de rastreamento em um vetor para evitar a poluição do console
/// em ambientes concorrentes com barras de progresso ativas.
pub fn write_xlsx<'de, T, P>(
    lines: &[T],
    sheet_name: &str,
    output_file: P,
    memory_mode: ExcelMemoryMode,
) -> XmlParserResult<Vec<String>>
// Retorna o vetor de logs em caso de sucesso
where
    P: AsRef<Path> + std::marker::Copy + std::fmt::Debug,
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable + Sync,
{
    let mut logs = Vec::new();

    if lines.is_empty() {
        return Ok(logs);
    }

    let output_path = output_file.as_ref();
    let file_name = output_path.to_string_lossy();

    logs.push(format!("Generating Excel file: {}", file_name));

    let mut workbook = Workbook::new();
    let properties = get_properties()?;
    workbook.set_properties(&properties);

    let number_of_rows = lines.len();
    let number_of_columns = T::get_headers().len();
    let number_of_sheets = number_of_rows.div_ceil(MAX_NUMBER_OF_ROWS);

    logs.push(format!(
        "Info: Dataset '{}' contains {} rows and {} columns. Partitioning into {} worksheet chunk(s)...",
        sheet_name, number_of_rows, number_of_columns, number_of_sheets
    ));

    if memory_mode.is_parallel() {
        logs.push(
            "Info: Starting concurrent worksheet generation across thread pool...".to_string(),
        );

        // Coleta os logs específicos das threads paralelas de forma segura
        let worksheets = lines
            .par_chunks(MAX_NUMBER_OF_ROWS)
            .enumerate()
            .map(
                |(index, chunk)| -> XmlParserResult<(Worksheet, Vec<String>)> {
                    let name = determine_sheet_name(sheet_name, index);
                    let mut thread_logs = Vec::new();
                    thread_logs.push(format!("Info: Thread working on worksheet '{}'...", name));

                    let mut worksheet = Worksheet::new();
                    worksheet.set_name(&name)?;
                    populate_worksheet(&mut worksheet, chunk)?;
                    Ok((worksheet, thread_logs))
                },
            )
            .collect::<Result<Vec<(Worksheet, Vec<String>)>, _>>()?;

        for (worksheet, thread_logs) in worksheets {
            logs.extend(thread_logs);
            workbook.push_worksheet(worksheet);
        }
    } else {
        logs.push(format!(
            "Info: Starting sequential worksheet generation (memory mode: {:?})...",
            memory_mode
        ));

        let chunks = lines.chunks(MAX_NUMBER_OF_ROWS);
        for (index, chunk) in chunks.enumerate() {
            let name = determine_sheet_name(sheet_name, index);
            logs.push(format!(
                "Info: Writing worksheet '{}' sequentially...",
                name
            ));

            let worksheet = memory_mode.add_worksheet_to_workbook(&mut workbook);
            worksheet.set_name(&name)?;
            populate_worksheet(worksheet, chunk)?;
        }
    }

    logs.push("Info: Writing workbook data to disk...".to_string());

    // Salva o resultado final no disco
    workbook
        .save(output_file)
        .map_err(|xlsx_error| XmlParserError::ExcelWrite {
            source: xlsx_error,
            path: output_path.to_path_buf(),
        })?;

    logs.push(format!(
        "Success: Excel document saved to '{}'\n",
        file_name
    ));

    Ok(logs)
}

/// Helper para nomear planilhas segmentadas com base no índice.
#[inline]
fn determine_sheet_name(base_name: &str, chunk_index: usize) -> String {
    if chunk_index == 0 {
        base_name.to_string()
    } else {
        format!("{} {}", base_name, chunk_index + 1)
    }
}

/// Monta as propriedades de metadados do documento gerado.
fn get_properties() -> XmlParserResult<DocProperties> {
    let properties = DocProperties::new()
        .set_title("Read XML")
        .set_subject("Read NFe/CTe XML files recursively and write XLSX files")
        .set_author("Claudio FSR (https://github.com/claudiofsr/read_xml)")
        .set_keywords("read, NFe, CTe, xml")
        .set_comment("Built with Rust")
        .set_hyperlink_base("https://github.com/claudiofsr/read_xml");

    Ok(properties)
}

/// Extensão para a estrutura `Table` da biblioteca `rust_xlsxwriter`.
///
/// Facilita a instanciação de tabelas pré-configuradas com estilos alternados
/// com base em um índice de controle.
pub trait TableExt {
    /// Cria uma nova tabela configurada com autofiltro ativo, sem linha de totais
    /// e aplicando a alternância de cores Azul (Medium2) e Cinza (Medium5).
    fn new_alternated(index: usize) -> Self;
}

impl TableExt for Table {
    fn new_alternated(index: usize) -> Self {
        // Define a alternância com base no índice do segmento
        let style = if index.is_multiple_of(2) {
            TableStyle::Medium5 // Cinza
        } else {
            TableStyle::Medium2 // Azul
        };

        Table::new()
            .set_autofilter(true)
            .set_total_row(false)
            .set_style(style)
    }
}

/// Extrai a última palavra de uma string delimitada por espaços.
///
/// Este método identifica transições semânticas de grupos de cabeçalhos no arquivo XML
/// comparando a palavra final de cada coluna contígua.
///
/// # Exemplos
///
/// Para que este teste de documentação funcione, certifique-se de que a função
/// esteja exportada no arquivo raiz do seu crate (`lib.rs` ou `main.rs`).
///
/// ```rust
/// use read_xml::last_word;
///
/// assert_eq!(last_word("Nome do Contribuinte"), Some("Contribuinte"));
/// assert_eq!(last_word("CNPJ do     Emitente"), Some("Emitente"));
/// assert_eq!(last_word("Valor Total"), Some("Total"));
/// assert_eq!(last_word("   "), None);
/// ```
pub fn last_word(name: &str) -> Option<&str> {
    name.split_whitespace().next_back()
}

/// Preenche os dados de formatação e serializa as linhas em uma planilha existente.
///
/// Esta função opera de forma totalmente compatível com o modo de streaming (`constant memory`),
/// o que impõe restrições estritas na ordenação sequencial das operações de gravação física.
///
/// # Racional da Ordenação de Operações
///
/// Para que o modo de memória constante grave as linhas com eficiência e baixo uso de RAM,
/// o ponteiro de gravação do buffer de células deve avançar estritamente de `0` a `N`.
/// Por este motivo, o ciclo de vida desta função executa as etapas na seguinte ordem:
///
/// 1. **Formatação de Linha (Metadados da Linha 0)**: Configura a altura e o estilo visual
///    da linha onde ficarão os cabeçalhos das tabelas.
/// 2. **Configuração de Cabeçalhos da Struct (`deserialize_headers`)**: Associa de forma lógica
///    os campos da struct genérica `T` às colunas físicas (etapa que não executa escritas físicas).
/// 3. **Configuração de Estilos de Coluna (`apply_column_formats`)**: Associa alinhamentos,
///    fontes e máscaras numéricas a cada coluna.
/// 4. **Segmentação Dinâmica e Escrita de Tabelas (`add_table`)**: Analisa e agrupa as fronteiras das colunas
///    por meio de agrupamento funcional contíguo sem alocação (`.chunk_by()`), escrevendo cada segmento de
///    tabela de forma direta sob demanda.
/// 5. **Serialização dos Dados (`serialize`)**: Descarrega recursivamente os dados em fluxo contínuo
///    para o disco, iniciando da linha `1` em diante.
/// 6. **Auto-ajuste de Largura (`auto_fit`)**: Analisa as larguras de células na memória e ajusta
///    as larguras de colunas físicas de forma retrospectiva.
///
/// # Erros
///
/// Retorna um [`XmlParserError`] caso ocorram falhas internas de formatação do Excel,
/// conflito de sobreposição de colunas de tabelas ou erros de gravação física no disco.
fn populate_worksheet<'de, T>(worksheet: &mut Worksheet, lines: &[T]) -> XmlParserResult<()>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable,
{
    let column_names: &[&str] = T::get_headers();
    let row_number: u32 = lines.len().try_into()?;

    let formatter = ExcelFormatter::new();
    let header_fmt = formatter.header_format().ok_or_else(|| {
        rust_xlsxwriter::XlsxError::ParameterError(
            "Header format registration is missing".to_string(),
        )
    })?;

    // 1. Configura as propriedades de metadados da linha de cabeçalho (linha 0)
    worksheet
        .set_row_height(0, 64)?
        .set_row_format(0, header_fmt)?
        .set_freeze_panes(1, 0)?;

    // 2. Registra o mapeamento de desserialização de cabeçalhos a partir do tipo genérico T
    worksheet.deserialize_headers::<T>(0, 0)?;

    // 3. Aplica alinhamentos e formatos das colunas de forma limpa via RegexSet
    formatter.apply_column_formats(worksheet, column_names)?;

    // 4. Detecção Dinâmica de Fronteiras de Colunas (Arquitetura Desacoplada e Fluida)
    if !column_names.is_empty() {
        let total_cols = column_names.len();

        // Divide as colunas de forma contígua em dois blocos a partir da posição da "Chave"
        let (grouped_columns, details_columns) = column_names.split_at(
            column_names
                .iter()
                .position(|name| name.contains("Chave do Documento Fiscal"))
                .unwrap_or(total_cols),
        );

        let mut col_offset = 0;
        let mut table_index = 0;

        // O método chunk_by() varre a fatia original agrupando elementos adjacentes
        // onde o predicado é verdadeiro (mesma palavra de sufixo no cabeçalho),
        // eliminando completamente o controle de estado complexo e alocações na Heap.
        for chunk in grouped_columns.chunk_by(|a, b| last_word(a) == last_word(b)) {
            let start = col_offset;
            // Estaticamente seguro: chunk_by garante que os blocos gerados nunca são vazios (len >= 1)
            let end = col_offset + chunk.len() - 1;
            col_offset += chunk.len(); // Incrementa o deslocamento acumulado de colunas

            // Aplica os estilos alternados baseando-se no índice lógico do bloco de dados
            let table = Table::new_alternated(table_index);
            worksheet.add_table(0, start as u16, row_number, end as u16, &table)?;
            table_index += 1;
        }

        // Se o bloco de detalhes (colunas à direita da Chave) existir,
        // adiciona-o como uma única tabela final contígua até a última coluna.
        if !details_columns.is_empty() {
            let start = col_offset;
            let end = total_cols - 1;

            let table = Table::new_alternated(table_index);
            worksheet.add_table(0, start as u16, row_number, end as u16, &table)?;
        }
    }

    // 5. Serializa os dados sequencialmente escrevendo-os em disco (avanço progressivo da linha 1 a N)
    worksheet.serialize(&lines)?;

    // 6. Dimensiona as larguras ideais de colunas baseando-se no conteúdo textual das células
    auto_fit(worksheet, lines, column_names)?;

    Ok(())
}

/// Dimensiona as larguras de coluna com base no conteúdo avaliado.
fn auto_fit<'de, T>(
    worksheet: &mut Worksheet,
    lines: &[T],
    column_names: &[&str],
) -> XmlParserResult<()>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable,
{
    let mut max_length: HashMap<usize, usize> = HashMap::new();

    let width_min = 10;
    let width_max = 150;
    let adjustment = 1.4;

    column_names
        .iter()
        .enumerate()
        .for_each(|(col_index, col_name)| {
            let col_len = col_name.chars_count().div_ceil(4);
            let col_width = width_min.max(col_len);
            max_length.insert(col_index, col_width);
        });

    for line in lines {
        get_length_of_column_values(line, &mut max_length);
    }

    for (index, len) in max_length {
        let width = width_max.min(len);
        worksheet.set_column_width(index as u16, (width as f64) * adjustment)?;
    }

    Ok(())
}

/// Extrai e mensura o comprimento em caracteres de cada coluna de forma **zero-allocation**.
pub fn get_length_of_column_values<'de, T>(line: &T, max_length: &mut HashMap<usize, usize>)
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable,
{
    line.iter()
        .enumerate()
        .for_each(|(index, (_field_name, field_value))| {
            let field_value_len: usize = match_cast!(field_value {
                // Tipos Inteiros (Cálculo via contagem de dígitos ilog10)
                val as Option<u8> => {
                    val.map(|v| count_u64_digits(v as u64))
                },
                val as Option<u16> => {
                    val.map(|v| count_u64_digits(v as u64))
                },
                val as Option<u32> => {
                    val.map(|v| count_u64_digits(v as u64))
                },
                val as Option<u64> => {
                    val.map(count_u64_digits)
                },
                val as Option<usize> => {
                    val.map(|v| count_u64_digits(v as u64))
                },

                // Ponto Flutuante (Cálculo da máscara monetária com separadores de milhar)
                val as Option<f64> => {
                    val.map(count_f64_formatted_len)
                },

                // Datas (Tamanho constante "DD/MM/YYYY" ou cálculo matemático)
                val as Option<NaiveDate> => {
                    val.map(count_date_len)
                },

                // Strings Primitivas e Opcionais (Medição de caracteres direta por referência)
                val as Option<String> => {
                    val.as_deref().map(|s| s.chars_count())
                },
                val as String => {
                    Some(val.chars_count())
                },

                // Coleção de Strings (Calcula "[item1, item2]" sem criar String via join)
                val as Vec<String> => {
                    Some(count_vec_str_len(val))
                },
                val as Option<Vec<String>> => {
                    val.as_deref().map(count_vec_str_len)
                },

                // Booleanos
                val as Option<bool> => {
                    val.map(|_b| 5) // "Sim" / "Não" / "True" / "False"
                },
            })
            // Fallback constante sem invocar format!
            .unwrap_or(10);

            // Atualiza a largura máxima usando a API de Entry do HashMap em O(1)
            let length = max_length.entry(index).or_insert(0);
            if field_value_len > *length {
                *length = field_value_len;
            }
        });
}

// =============================================================================
// FUNÇÕES AUXILIARES DE ALTA PERFORMANCE (ZERO-ALLOCATION)
// =============================================================================

/// Calcula a quantidade de dígitos de um número inteiro sem alocar memória.
#[inline]
pub fn count_u64_digits(val: u64) -> usize {
    if val == 0 {
        1
    } else {
        val.checked_ilog10().unwrap_or(0) as usize + 1
    }
}

/// Calcula a quantidade de caracteres de um `f64` formatado como moeda (`#,##0.00`)
/// considerando sinal negativo, separadores de milhar e duas casas decimais.
#[inline]
pub fn count_f64_formatted_len(val: f64) -> usize {
    if !val.is_finite() {
        return 0;
    }

    let sign_len = if val.is_sign_negative() { 1 } else { 0 };
    let abs_val = val.abs();
    let integer_part = abs_val.trunc() as u64;

    // Quantidade de dígitos da parte inteira
    let digits = count_u64_digits(integer_part);

    // Quantidade de separadores de milhar (um '.' a cada 3 dígitos inteiros além do primeiro)
    let thousand_separators = (digits.saturating_sub(1)) / 3;

    // Total = Sinal + Dígitos Inteiros + Separadores de Milhar + Vírgula (1) + Casas Decimais (2)
    sign_len + digits + thousand_separators + 3
}

/// Calcula a quantidade de caracteres de uma data no padrão `DD/MM/YYYY`.
#[inline]
pub fn count_date_len(_date: NaiveDate) -> usize {
    10 // "DD/MM/YYYY" sempre ocupa 10 caracteres
}

/// Calcula o comprimento exato da serialização `[item1, item2, ...]` sem alocar memória na Heap.
#[inline]
pub fn count_vec_str_len(vec: &[String]) -> usize {
    if vec.is_empty() {
        return 0;
    }

    let chars_total: usize = vec.iter().map(|s| s.chars_count()).sum();
    let commas_and_spaces = (vec.len().saturating_sub(1)) * 2; // ", " entre itens
    let brackets = 2; // "[" e "]"

    chars_total + commas_and_spaces + brackets
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output tests_excel
#[cfg(test)]
mod tests_excel {
    use super::*;

    #[test]
    fn test_count_u64_digits() {
        assert_eq!(count_u64_digits(0), 1);
        assert_eq!(count_u64_digits(9), 1);
        assert_eq!(count_u64_digits(10), 2);
        assert_eq!(count_u64_digits(999), 3);
        assert_eq!(count_u64_digits(1_000_000), 7);
    }

    #[test]
    fn test_count_f64_formatted_len_precision() {
        // Validação contra a formatação real de strings
        let casos = [
            (0.00, "0.00"),
            (7.50, "7.50"),
            (123.45, "123.45"),
            (1234.56, "1,234.56"),
            (1234567.89, "1,234,567.89"),
            (-1234.56, "-1,234.56"),
        ];

        for (valor, formatado_esperado) in casos {
            assert_eq!(
                count_f64_formatted_len(valor),
                formatado_esperado.len(),
                "Falha ao medir comprimento de: {valor}"
            );
        }
    }

    #[test]
    fn test_count_vec_str_len() {
        let vec_vazio: Vec<String> = Vec::new();
        assert_eq!(count_vec_str_len(&vec_vazio), 0);

        let vec_com_itens = vec![
            "NFe_001".to_string(),
            "NFe_002".to_string(),
            "NFe_003".to_string(),
        ];
        let formatado_real = format!("[{}]", vec_com_itens.join(", "));

        assert_eq!(count_vec_str_len(&vec_com_itens), formatado_real.len());
    }
}
