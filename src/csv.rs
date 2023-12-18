use serde::Serialize;
use std::{
    fs,
    path::PathBuf,
};

use crate::MyResult;

pub const DELIMITER_CHAR: char = ';';

/// Print CSV File
///
/// With DELIMITER_CHAR
pub fn print_csv_file<T>(lines: &[T], csv_filename: &str) -> MyResult<()>
where
    T: Serialize
{
    if lines.is_empty() {
        return Ok(());
    }

    let mut csv_file = PathBuf::from(csv_filename);
    csv_file.set_extension("csv");

    eprintln!("\nWrite CSV File: {:?}", csv_file.display());

    if let Err(why) = write_csv(lines, &csv_file) {
        eprintln!("fn print_csv_file()");
        eprintln!("CSV File: {:?}", csv_file.display());
        eprintln!("Error: {why}");
    }

    Ok(())
}

// https://docs.rs/csv/latest/csv/tutorial/index.html
// https://github.com/andrewleverette/rust_csv_examples/blob/master/src/bin/csv_write_serde.rs
fn write_csv<T>(lines: &[T], path: &PathBuf) -> MyResult<()>
where
    T: Serialize
{
    // Open a file in write-only mode, returns `io::Result<File>`
    let file = match fs::File::create(path) {
        Ok(file) => file,
        Err(error) => panic!("Couldn't create {:?}: {error}", path.display()),
    };

    let mut writer = csv::WriterBuilder::new()
        .delimiter(DELIMITER_CHAR as u8)
        .has_headers(true) // write the header
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(file);

    for line in lines {
        writer.serialize(line)?;
    }

    writer.flush()?;

    Ok(())
}