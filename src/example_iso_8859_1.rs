#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NotaFiscal {
    pub emitente: String,
    pub endereco: String,
    pub observacao: String,
}

#[cfg(test)]
mod test_example_iso_8859_1 {
    use crate::{XmlBufReadExt, XmlParserResult, XmlPathExt};
    use quick_xml::de::{DeError, from_reader};
    use quick_xml::events::Event;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct NotaFiscal {
        emitente: String,
        endereco: String,
        observacao: String,
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

    // Bytes reais em ISO-8859-1:
    // \xE9 = é | \xE7 = ç | \xE3 = ã | \xBA = º | \xAA = ª
    const XML_ISO_8859_1: &[u8] = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>
<NotaFiscal>
  <Emitente>Com\xE9rcio de Pe\xE7as S\xE3o Jo\xE3o Ltda</Emitente>
  <Endereco>Pra\xE7a da S\xE9, n\xBA 100 - 1\xAA Loja</Endereco>
  <Observacao>Opera\xE7\xE3o isenta de tributa\xE7\xE3o</Observacao>
</NotaFiscal>";

    // Bytes ISO-8859-1 sem atributo encoding na declaração XML
    const XML_ISO_8859_1_SEM_ENCODING: &[u8] = b"<?xml version=\"1.0\"?>
<NotaFiscal>
  <Emitente>Com\xE9rcio de Pe\xE7as S\xE3o Jo\xE3o Ltda</Emitente>
  <Endereco>Pra\xE7a da S\xE9, n\xBA 100 - 1\xAA Loja</Endereco>
  <Observacao>Opera\xE7\xE3o isenta de tributa\xE7\xE3o</Observacao>
</NotaFiscal>";

    // Bytes ISO-8859-1 declarando encoding="UTF-8" incorretamente
    const XML_ISO_8859_1_ENCODING_INCORRETO: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<NotaFiscal>
  <Emitente>Com\xE9rcio de Pe\xE7as S\xE3o Jo\xE3o Ltda</Emitente>
  <Endereco>Pra\xE7a da S\xE9, n\xBA 100 - 1\xAA Loja</Endereco>
  <Observacao>Opera\xE7\xE3o isenta de tributa\xE7\xE3o</Observacao>
</NotaFiscal>";

    #[test]
    /// `cargo test -- --show-output deserialize_xml_iso_8859_1`
    fn deserialize_xml_iso_8859_1() -> XmlParserResult<()> {
        let decoder = XML_ISO_8859_1.into_decoding_reader()?;
        let nf: NotaFiscal = from_reader(decoder)?;

        assert_eq!(nf.emitente, "Comércio de Peças São João Ltda");
        assert_eq!(nf.endereco, "Praça da Sé, nº 100 - 1ª Loja");
        assert_eq!(nf.observacao, "Operação isenta de tributação");

        println!("ISO-8859-1 -> {nf:#?}");
        Ok(())
    }

    #[test]
    /// Garante que sem `DecodingReader` o `quick-xml` 0.42 rejeita os bytes não-UTF-8.
    fn fail_without_decoding_reader_iso_8859_1() {
        let result: Result<NotaFiscal, _> = from_reader(XML_ISO_8859_1);
        assert!(matches!(
            result,
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_)))
        ));
    }

    #[test]
    /// Testa leitura por eventos (`Reader`) combinada com `into_xml_reader`, validando texto acentuado.
    fn read_events_iso_8859_1() -> XmlParserResult<()> {
        let mut reader = XML_ISO_8859_1.into_xml_reader()?;
        let mut buf = Vec::new();
        let mut emitente = None;
        let mut inside_emitente = false;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if e.name().into_inner() == "Emitente" => {
                    inside_emitente = true;
                }
                Event::Text(e) if inside_emitente => {
                    emitente = Some(e.into_inner().into_owned());
                    break;
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        assert_eq!(emitente.as_deref(), Some("Comércio de Peças São João Ltda"));
        Ok(())
    }

    #[test]
    /// Modo estrito (`into_decoding_reader`): XML sem atributo `encoding` com bytes ISO-8859-1 deve falhar.
    fn reject_iso_8859_1_without_encoding() -> XmlParserResult<()> {
        let decoder = XML_ISO_8859_1_SEM_ENCODING.into_decoding_reader()?;
        let result: Result<NotaFiscal, _> = from_reader(decoder);

        assert!(matches!(
            result,
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_)))
        ));
        Ok(())
    }

    #[test]
    /// Modo estrito (`into_decoding_reader`): XML declarando `UTF-8` mas com bytes ISO-8859-1 deve falhar.
    fn reject_iso_8859_1_with_wrong_encoding() -> XmlParserResult<()> {
        let decoder = XML_ISO_8859_1_ENCODING_INCORRETO.into_decoding_reader()?;
        let result: Result<NotaFiscal, _> = from_reader(decoder);

        assert!(matches!(
            result,
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_)))
        ));
        Ok(())
    }

    #[test]
    /// Modo resiliente (`XmlPathExt::parse_xml`): recupera XML ISO-8859-1 mesmo sem `encoding` via fallback.
    fn fallback_parse_xml_iso_8859_1_without_encoding() -> XmlParserResult<()> {
        let tmp = TempXmlFile::new("iso_sem_enc.xml", XML_ISO_8859_1_SEM_ENCODING)?;
        let nf: NotaFiscal = tmp.0.parse_xml()?;

        assert_eq!(nf.emitente, "Comércio de Peças São João Ltda");
        assert_eq!(nf.endereco, "Praça da Sé, nº 100 - 1ª Loja");
        assert_eq!(nf.observacao, "Operação isenta de tributação");
        Ok(())
    }

    #[test]
    /// Modo resiliente (`XmlPathExt::parse_xml`): recupera XML ISO-8859-1 mesmo declarando `UTF-8` incorretamente.
    fn fallback_parse_xml_iso_8859_1_with_wrong_encoding() -> XmlParserResult<()> {
        let tmp = TempXmlFile::new("iso_enc_errado.xml", XML_ISO_8859_1_ENCODING_INCORRETO)?;
        let nf: NotaFiscal = tmp.0.parse_xml()?;

        assert_eq!(nf.emitente, "Comércio de Peças São João Ltda");
        assert_eq!(nf.endereco, "Praça da Sé, nº 100 - 1ª Loja");
        assert_eq!(nf.observacao, "Operação isenta de tributação");
        Ok(())
    }
}
