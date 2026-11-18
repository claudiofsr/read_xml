#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fatura {
    pub cliente: String,
    pub descricao: String,
    pub moeda: String,
}

#[cfg(test)]
mod test_example_windows_1252 {
    use crate::{XmlBufReadExt, XmlParserResult, XmlPathExt};
    use quick_xml::de::{DeError, from_reader};
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Fatura {
        cliente: String,
        descricao: String,
        moeda: String,
    }

    struct TempXmlFile(PathBuf);

    impl TempXmlFile {
        fn new(name: &str, bytes: &[u8]) -> std::io::Result<Self> {
            let path = std::env::temp_dir().join(format!("read_xml_{}_{name}", std::process::id()));
            fs::write(&path, bytes)?;
            Ok(Self(path))
        }
    }

    impl Drop for TempXmlFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    // Bytes reais em Windows-1252:
    // \xFA = ú | \xE9 = é | \xE7 = ç | \xE3 = ã
    // Exclusivos Windows-1252 (0x80..=0x9F):
    // \x80 = € | \x85 = … | \x93 = “ | \x94 = ” | \x96 = – | \x99 = ™
    const XML_WINDOWS_1252: &[u8] = b"<?xml version=\"1.0\" encoding=\"Windows-1252\"?>
<Fatura>
  <Cliente>Ind\xFAstria &amp; Com\xE9rcio \x93Modelo\x94</Cliente>
  <Descricao>Servi\xE7o t\xE9cnico \x96 plano padr\xE3o\x85</Descricao>
  <Moeda>\x80 1.250,00 \x99</Moeda>
</Fatura>";

    // Bytes Windows-1252 sem declaração XML (nem <?xml ...?>)
    const XML_WINDOWS_1252_SEM_ENCODING: &[u8] = b"<Fatura>
  <Cliente>Ind\xFAstria &amp; Com\xE9rcio \x93Modelo\x94</Cliente>
  <Descricao>Servi\xE7o t\xE9cnico \x96 plano padr\xE3o\x85</Descricao>
  <Moeda>\x80 1.250,00 \x99</Moeda>
</Fatura>";

    // Bytes Windows-1252 declarando encoding="UTF-8" incorretamente
    const XML_WINDOWS_1252_ENCODING_INCORRETO: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<Fatura>
  <Cliente>Ind\xFAstria &amp; Com\xE9rcio \x93Modelo\x94</Cliente>
  <Descricao>Servi\xE7o t\xE9cnico \x96 plano padr\xE3o\x85</Descricao>
  <Moeda>\x80 1.250,00 \x99</Moeda>
</Fatura>";

    #[test]
    /// `cargo test -- --show-output deserialize_xml_windows_1252`
    fn deserialize_xml_windows_1252() -> XmlParserResult<()> {
        let decoder = XML_WINDOWS_1252.into_decoding_reader()?;
        let fatura: Fatura = from_reader(decoder)?;

        assert_eq!(fatura.cliente, "Indústria & Comércio “Modelo”");
        assert_eq!(fatura.descricao, "Serviço técnico – plano padrão…");
        assert_eq!(fatura.moeda, "€ 1.250,00 ™");

        println!("Windows-1252 -> {fatura:#?}");
        Ok(())
    }

    #[test]
    /// Garante que sem `DecodingReader` o `quick-xml` 0.42 rejeita os bytes não-UTF-8.
    fn fail_without_decoding_reader_windows_1252() {
        let result: Result<Fatura, _> = from_reader(XML_WINDOWS_1252);
        assert!(matches!(
            result,
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_)))
        ));
    }

    #[test]
    /// Modo estrito (`into_decoding_reader`): XML sem declaração de `encoding` com bytes Windows-1252 deve falhar.
    fn reject_windows_1252_without_encoding() -> XmlParserResult<()> {
        let decoder = XML_WINDOWS_1252_SEM_ENCODING.into_decoding_reader()?;
        let result: Result<Fatura, _> = from_reader(decoder);

        assert!(matches!(
            result,
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_)))
        ));
        Ok(())
    }

    #[test]
    /// Modo estrito (`into_decoding_reader`): XML declarando `UTF-8` mas com bytes Windows-1252 deve falhar.
    fn reject_windows_1252_with_wrong_encoding() -> XmlParserResult<()> {
        let decoder = XML_WINDOWS_1252_ENCODING_INCORRETO.into_decoding_reader()?;
        let result: Result<Fatura, _> = from_reader(decoder);

        assert!(matches!(
            result,
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_)))
        ));
        Ok(())
    }

    #[test]
    /// Modo resiliente (`XmlPathExt::parse_xml`): recupera XML Windows-1252 mesmo sem declaração de `encoding`.
    fn fallback_parse_xml_windows_1252_without_encoding() -> XmlParserResult<()> {
        let tmp = TempXmlFile::new("win1252_sem_enc.xml", XML_WINDOWS_1252_SEM_ENCODING)?;
        let fatura: Fatura = tmp.0.parse_xml()?;

        assert_eq!(fatura.cliente, "Indústria & Comércio “Modelo”");
        assert_eq!(fatura.descricao, "Serviço técnico – plano padrão…");
        assert_eq!(fatura.moeda, "€ 1.250,00 ™");
        Ok(())
    }

    #[test]
    /// Modo resiliente (`XmlPathExt::parse_xml`): recupera XML Windows-1252 mesmo declarando `UTF-8` incorretamente.
    fn fallback_parse_xml_windows_1252_with_wrong_encoding() -> XmlParserResult<()> {
        let tmp = TempXmlFile::new(
            "win1252_enc_errado.xml",
            XML_WINDOWS_1252_ENCODING_INCORRETO,
        )?;
        let fatura: Fatura = tmp.0.parse_xml()?;

        assert_eq!(fatura.cliente, "Indústria & Comércio “Modelo”");
        assert_eq!(fatura.descricao, "Serviço técnico – plano padrão…");
        assert_eq!(fatura.moeda, "€ 1.250,00 ™");
        Ok(())
    }
}
