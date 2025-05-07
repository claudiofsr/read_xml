# read_xml
Read [NFe](https://www.nfe.fazenda.gov.br) and [CTe](https://www.cte.fazenda.gov.br) xml files recursively and write XLSX/CSV files.

In a directory containing XML files, in the command line terminal run:

```
read_xml
```
The output will be saved to an XLSX (excel) file.

## Building

To build and install from source, run the following command:
```
git clone https://github.com/claudiofsr/read_xml.git
cd read_xml
cargo b -r && cargo install --path=.
```

## Examples

To show all fields and values, use the verbose option:

```
read_xml -v
```

To redirect the output to another file with all fields and values:

```
read_xml -tv > /tmp/xml_files.txt
```

To parse CTe or NFe xml file and print Rust struct:

```
read_xml -s cte.xml
```

And to print nodes from xml files:

```
read_xml -n cte.xml
```
