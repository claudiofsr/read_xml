use serde::Serialize;
use std::{
    fs,
    fmt::Debug,
    marker::Copy,
    path::Path,
};

use crate::MyResult;

/// Write CSV File with DELIMITER_CHAR
///
/// <https://docs.rs/csv/latest/csv/tutorial/index.html>
///
/// <https://github.com/andrewleverette/rust_csv_examples/blob/master/src/bin/csv_write_serde.rs>
pub fn write_csv<T, P>(lines: &[T], output_file: P, delimiter: char) -> MyResult<()>
where
    P: AsRef<Path> + Copy + Debug,
    T: Serialize
{
    if lines.is_empty() {
        return Ok(());
    }

    eprintln!("Write CSV File: {:?}", output_file);

    // Open a file in write-only mode, returns `io::Result<File>`
    let file = match fs::File::create(output_file) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("fn write_csv()");
            eprintln!("Couldn't create {:?}", output_file);
            panic!("Error: {error}");
        },
    };

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(true) // write the header
        .quote_style(csv::QuoteStyle::Necessary) // NonNumeric
        .from_writer(file);

    for line in lines {
        writer.serialize(line)?;
    }

    writer.flush()?;

    Ok(())
}