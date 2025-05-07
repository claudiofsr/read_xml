use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::MyResult;

// Write CSV File with DELIMITER_CHAR
// <https://docs.rs/csv/latest/csv/tutorial/index.html>
// <https://github.com/andrewleverette/rust_csv_examples/blob/master/src/bin/csv_write_serde.rs>

/**
Utilizar boas práticas de programação, como o Solid:
1. **Single Responsibility Principle (SRP)**: Criei uma estrutura `CsvWriter` que tem a responsabilidade única de escrever arquivos CSV.
2. **Dependency Injection**: O construtor `new` recebe os parâmetros `output_file` e `delimiter`, que são armazenados como campos da estrutura.
3. **Separation of Concerns (SoC)**: Dividi a lógica em métodos menores, cada um com uma responsabilidade específica (e.g., `open_file`, `create_writer`, `write`).

A estrutura `CsvWriter` pode ser usada para escrever arquivos CSV:
```ignore
use read_xml::CsvWriter;
let writer = CsvWriter::new("output.csv", ',');
writer.write(&my_data).unwrap();
```
Note que você precisa ajustar os tipos e parâmetros da função `write_csv` para se adequar às suas necessidades específicas.
*/
pub struct CsvWriter {
    output_file: PathBuf,
    delimiter: char,
}

impl CsvWriter {
    pub fn new(output_file: PathBuf, delimiter: char) -> Self {
        CsvWriter {
            output_file,
            delimiter,
        }
    }

    pub fn write<T>(&self, lines: &[T]) -> MyResult<()>
    where
        T: Serialize,
    {
        if lines.is_empty() {
            return Ok(());
        }

        let file = self.open_file()?;
        let mut writer = self.create_writer(file);

        // Write each line to the CSV file
        for line in lines {
            writer.serialize(line)?;
        }

        writer.flush()?;

        Ok(())
    }

    // Open a file in write-only mode, returns `io::Result<File>`
    fn open_file(&self) -> MyResult<fs::File> {
        fs::File::create(Path::new(&self.output_file)).map_err(|error| {
            eprintln!("Couldn't create {:?}", self.output_file);
            panic!("Error: {error}");
        })
    }

    // Create a writer for the CSV file
    fn create_writer(&self, file: fs::File) -> csv::Writer<fs::File> {
        csv::WriterBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(true) // write the header
            .quote_style(csv::QuoteStyle::Necessary) // NonNumeric
            .from_writer(file)
    }
}
