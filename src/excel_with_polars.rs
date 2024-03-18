// Not Used !!!!

use std::{
    env,
    path::PathBuf,
};

use rust_xlsxwriter::{
    Table,
    Format,
    FormatAlign,
    Workbook,
    Worksheet,
};

use polars::prelude::*;
use regex::Regex;

use crate::{
    MyResult,
    InfoEFinanceira,
    PolarsXlsxWriter,
    OUTPUT_FILENAME
};

const FONT_SIZE: f64 = 10.0;

/// Polar arguments with ENV vars
pub fn configure_the_environment() {
    // https://stackoverflow.com/questions/70830241/rust-polars-how-to-show-all-columns/75675569#75675569
    // https://docs.rs/polars/latest/polars/#config-with-env-vars
    env::set_var("POLARS_FMT_TABLE_ROUNDED_CORNERS", "1"); // apply rounded corners to UTF8-styled tables.
    env::set_var("POLARS_FMT_TABLE_INLINE_COLUMN_DATA_TYPE", "0"); // put column data type on the same line as the column name.
    //env::set_var("POLARS_FMT_MAX_COLS", "20"); // maximum number of columns shown when formatting DataFrames.
    env::set_var("POLARS_FMT_MAX_ROWS", "10");   // maximum number of rows shown when formatting DataFrames.
    env::set_var("POLARS_FMT_STR_LEN", "52");    // maximum number of characters printed per string value.
}

/// Write Dataframe to xlsx Excel file
///
/// <https://crates.io/crates/polars_excel_writer>
///
/// <https://github.com/jmcnamara/polars_excel_writer/issues/4>
pub fn write_xlsx(lines: &[InfoEFinanceira], sheet_name: &str) -> MyResult<()> {

    let mut xlsx_file = PathBuf::from(OUTPUT_FILENAME);
    xlsx_file.set_extension("xlsx");

    eprintln!("\nWrite xlsx file: {:?}", xlsx_file.display());

    let mut dataframe: DataFrame = get_dataframe_from_vec_struct(lines)?;
    rename_dataframe_header(&mut dataframe)?;

    // Workbook with worksheets
    let mut workbook = Workbook::new();
    let mut worksheet = Worksheet::new();

    format_worksheet(&dataframe, &mut worksheet, sheet_name)?;

    // Date format must be applied to PolarsXlsxWriter.
    let fmt_date = Format::new()
        .set_align(FormatAlign::Center)
        .set_num_format("dd/mm/yyyy");

    // Write the dataframe to the worksheet using `PolarsXlsxWriter`.
    PolarsXlsxWriter::new()
        .set_date_format(fmt_date)
        .set_float_format("#,##0.00")
        .write_dataframe_to_worksheet(&dataframe, &mut worksheet, 0, 0)?;

    // worksheet.autofit();
    // auto_fit(&dataframe, &mut worksheet)?;

    workbook.push_worksheet(worksheet);

    // Save the workbook to disk.
    workbook.save(xlsx_file)?;

    Ok(())
}

/// Creating Polars Dataframe from Vec<Struct>
///
/// <https://stackoverflow.com/questions/73167416/creating-polars-dataframe-from-vecstruct>
///
/// <https://doc.rust-lang.org/reference/macros-by-example.html>
///
/// <https://danielkeep.github.io/tlborm/book/README.html>
///
/// <https://blog.turbo.fish/proc-macro-simple-derive>
///
/// <https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples>
///
/// <https://crates.io/crates/struct_iterable>
macro_rules! struct_to_dataframe {
    ($input:expr, [$($field_name:ident),+]) => {
        {
            // ident for field name
            // Extract the field values into separate vectors
            $(let mut $field_name = Vec::new();)*

            for e in $input.into_iter() {
                $($field_name.push(e.$field_name.clone());)*
            }

            df! {
                $(stringify!($field_name) => $field_name,)*
            }
        }
    };
}

pub fn get_dataframe_from_vec_struct(lines: &[InfoEFinanceira]) -> MyResult<DataFrame> {

    let dataframe: DataFrame = struct_to_dataframe!(
        lines,
        [
            id, cnpj_do_declarante,
            ni_do_declarado, nome_declarado,
            ano_mes_caixa,
            num_conta,
            pais_reportado,
            tot_creditos, tot_debitos
        ]
    )?;

    Ok(dataframe)
}

pub fn rename_dataframe_header(dataframe: &mut DataFrame) -> MyResult<()> {

    let column_names_new: Vec<&str> = InfoEFinanceira::get_headers();
    //println!("column_names_new: {column_names_new:?}");

    let column_names_old: Vec<String> = dataframe
        .get_column_names()
        .iter()
        .map(|col_name| col_name.to_string())
        .collect();

    if column_names_old.len() != column_names_new.len() {
        eprintln!("fn get_dataframe_from_vec_struct()");
        eprintln!("column_names_old.len(): {}", column_names_old.len());
        eprintln!("column_names_new.len(): {}", column_names_new.len());
        panic!("Error: The old and new column are different lengths!");
    }

    for (old, new) in  column_names_old.iter().zip(column_names_new.iter()) {
        dataframe.rename(old, new)?;
    }

    /*
    let new_df = dataframe
        .clone()
        .lazy()
        .rename(column_names_old, column_names_new)
        .collect()?;
    */

    println!("dataframe: {dataframe}");

    Ok(())
}

/// Format worksheet
fn format_worksheet(dataframe: &DataFrame, worksheet: &mut Worksheet, sheet_name: &str) -> MyResult<()> {

    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::Center) // horizontally
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(FONT_SIZE);

    let fmt_center = Format::new()
        .set_align(FormatAlign::Center);

    let fmt_values = Format::new()
        .set_num_format("#,##0.00"); // two digits after the decimal point

    worksheet
        .set_name(sheet_name)?
        .set_row_format(0, &fmt_header)?
        .set_row_height(0, 64)?
        .set_freeze_panes(1, 0)?;

    let regex_center = Regex::new(r"(?ix)
        ^(:?CNPJ|CPF|Identifica|ni)
    ").unwrap();

    let regex_valor = Regex::new(r"(?ix)
        Total|Valor
    ").unwrap();

    let col_center = [
        // "CNPJ", "CPF",
        "Código",
        "Registro",
        "Chave do Documento",
    ];

    let col_values = [
        "Valor",
        "Total",
        "tot_creditos",
        "tot_debitos",
    ];

    for (column_number, col_name) in dataframe.get_column_names().iter().enumerate() {

        if regex_center.is_match(col_name) {
            worksheet.set_column_format(column_number as u16, &fmt_center)?;
            continue;
        }

        if regex_valor.is_match(col_name) {
            worksheet.set_column_format(column_number as u16, &fmt_values)?;
            continue;
        }

        for pattern in col_center {
            if col_name.contains(pattern) {
                worksheet.set_column_format(column_number as u16, &fmt_center)?;
                break;
            }
        }

        for value in col_values {
            if col_name.contains(value) {
                worksheet.set_column_format(column_number as u16, &fmt_values)?;
                break;
            }
        }
    }

    Ok(())
}

/// Iterate over all DataFrame and find the max data width for each column.
///
/// See:
///
/// <https://crates.io/crates/unicode-width>
///
/// <https://tomdebruijn.com/posts/rust-string-length-width-calculations>
fn auto_fit(df: &DataFrame, worksheet: &mut Worksheet) -> PolarsResult<()> {
    let width_min = 8;
    let width_max = 140;
    let adjustment = 1.10;

    for (col_num, series) in df.iter().enumerate() {
        let col_name  = series.name();
        let col_width = col_name.chars().count().div_ceil(4);
        let mut width = width_min.max(col_width);

        // analyze all column fields
        for data in series.iter() {
            let text = match data.dtype() {
                DataType::Float64 => {
                    let num: f64 = data.try_extract::<f64>()?;
                    //num.to_string()
                    format!("{num:0.2}") // two digits after the decimal point
                },
                DataType::Float32 => {
                    let num: f32 = data.try_extract::<f32>()?;
                    //num.to_string()
                    format!("{num:0.2}") // two digits after the decimal point
                },
                _ => data.to_string(),
            };

            let text_width = text.chars().count();

            if text_width > width {
                width = text_width;
            }

            if width > width_max {
                width = width_max;
                break;
            }
        }
        // println!("col_num: {col_num}, col_name: {col_name}, width: {width}");
        worksheet.set_column_width(col_num as u16, (width as f64) * adjustment)?;
    }

    Ok(())
}
