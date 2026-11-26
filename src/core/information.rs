//! # Centralização de Documentos Fiscais e Despacho de Parsers
//!
//! Define a taxonomia unificada de documentos (`Information`) e a trait de abstração
//! de desserialização (`StructExtension`).

use quick_xml::events::Event;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::Path;

use crate::{
    Arguments, REGEX_ERROR_DUPLICATE_FIELD, REGEX_ERROR_MISSING_FIELD, REGEX_FIELDS, XmlBufReadExt,
    XmlParserError, XmlParserResult, XmlPathExt,
    xml_structs::{
        cancelamento_cte::{InfoCteCancel, ProcCancCte},
        cancelamento_nfe::{InfoNfeCancel, ProcCancNfe},
        cte::{Cte, CteOs, CteOsProc, CteProc, CteSimp, CteSimpProc, Gtve, GtveProc, InfoCte},
        cte_evento::{self, InfoCteEvento, ProcEventoCte},
        efinanceira::{EFinanceira, InfoEFinanceira},
        nfe::{InfoNfe, Nfe, NfeProc},
        nfe_evento::{self, InfoNfeEvento, ProcEventoNfe},
    },
};

/// Enumera os tipos de documentos fiscais suportados pelo motor de processamento.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum Information {
    Cte(Box<InfoCte>),
    Nfe(Vec<InfoNfe>),
    EventoCte(Box<InfoCteEvento>),
    EventoNfe(Box<InfoNfeEvento>),
    CancelamentoCte(Box<InfoCteCancel>),
    CancelamentoNfe(Box<InfoNfeCancel>),
    EFinanceira(Vec<InfoEFinanceira>),
    #[default]
    None,
}

impl Information {
    /// Retorna `true` se a variante for [`Self::None`].
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(*self, Self::None)
    }

    /// Retorna `true` se contiver algum documento fiscal válido (diferente de `None`).
    #[inline]
    pub const fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub const fn is_cte(&self) -> bool {
        matches!(*self, Self::Cte(_))
    }

    pub const fn is_nfe(&self) -> bool {
        matches!(*self, Self::Nfe(_))
    }

    pub const fn is_evento_cte(&self) -> bool {
        matches!(*self, Self::EventoCte(_))
    }

    pub const fn is_evento_nfe(&self) -> bool {
        matches!(*self, Self::EventoNfe(_))
    }

    pub const fn is_efinanceira(&self) -> bool {
        matches!(*self, Self::EFinanceira(_))
    }
}

/// Trait implementada por estruturas que realizam parse a partir de streams XML.
pub trait StructExtension: Sized + DeserializeOwned {
    /// Realiza a abertura do arquivo XML e desserialização com buffer otimizado de 64 KB.
    fn xml_parse(path: &Path) -> XmlParserResult<Self> {
        path.parse_xml_with_capacity(64 * 1024)
    }

    /// Desserializa XML diretamente de um slice em memória (entradas de `.zip`).
    fn xml_parse_bytes(bytes: &[u8]) -> XmlParserResult<Self> {
        parse_xml_bytes(bytes)
    }

    /// Tenta o parse em memória e diagnostica anomalias de schema em caso de falha estrutural.
    fn bytes_to_info(
        bytes: &[u8],
        entry_label: &Path,
        arguments: &Arguments,
    ) -> XmlParserResult<Information> {
        match Self::xml_parse_bytes(bytes) {
            Ok(proc) => Ok(proc.get_information(entry_label, arguments)),
            Err(err) => Self::diagnosticar_erro_de_schema(&err, entry_label),
        }
    }

    /// Converte a struct de schema para a variante de domínio [`Information`].
    fn get_information(&self, xml_path: &Path, arguments: &Arguments) -> Information;

    /// Analisa erros de desserialização e emite sugestões de correção no `stderr`
    /// caso identifique que a struct Rust está desalinhada com o XML físico.
    fn diagnosticar_erro_de_schema(
        err: &XmlParserError,
        xml_path: &Path,
    ) -> XmlParserResult<Information> {
        let error_str = err.to_string();

        // Se o erro NÃO for apenas uma discordância de nó raiz comum (tentativa de outros parsers),
        // significa que encontramos o arquivo correto, mas a Struct Rust precisa de ajuste de campos.
        if !REGEX_FIELDS.is_match(&error_str) {
            let typename = std::any::type_name::<Self>();

            eprintln!("\n=== Inconsistência de Estrutura Detectada ===");
            eprintln!("Nome da Estrutura: {typename}");
            eprintln!("Detalhe do Erro:   {error_str}\n");

            if REGEX_ERROR_MISSING_FIELD.is_match(&error_str) {
                eprintln!("Sugestão de correção na Struct:");
                eprintln!("  Altere:");
                eprintln!("    field `nome_campo`: Tipo");
                eprintln!("  Para:");
                eprintln!("    #[serde(default)]");
                eprintln!("    field `nome_campo`: Option<Tipo>\n");
            }

            if REGEX_ERROR_DUPLICATE_FIELD.is_match(&error_str) {
                eprintln!("Sugestão de correção na Struct (Múltiplas Ocorrências):");
                eprintln!("  Altere:");
                eprintln!("    field `nome_campo`: Tipo");
                eprintln!("  Para:");
                eprintln!("    #[serde(default)]");
                eprintln!("    field `nome_campo`: Vec<Tipo>\n");
            }

            eprintln!("Para inspecionar os campos exatos deste arquivo XML, execute:");
            eprintln!("  read-xml -s {:?}\n", xml_path);
        }

        // Retorna None permitindo que outros parsers tentem ou que o fluxo continue
        Ok(Information::None)
    }
}

/// Desserializa um buffer XML com detecção automática de encoding e fallback Windows-1252.
pub fn parse_xml_bytes<T: DeserializeOwned>(bytes: &[u8]) -> XmlParserResult<T> {
    let decoder = bytes.into_decoding_reader()?;
    match quick_xml::de::from_reader(decoder) {
        Ok(parsed) => Ok(parsed),
        Err(quick_xml::DeError::InvalidXml(quick_xml::Error::Encoding(_))) => {
            let mut decoder = quick_xml::encoding::DecodingReader::new(bytes);
            decoder.set_encoding(encoding_rs::WINDOWS_1252);
            Ok(quick_xml::de::from_reader(decoder)?)
        }
        Err(err) => Err(err.into()),
    }
}

fn read_root_tag_from_reader<R: std::io::BufRead>(
    reader: &mut quick_xml::Reader<quick_xml::encoding::DecodingReader<R>>,
) -> Result<Option<String>, quick_xml::Error> {
    let mut buf = Vec::with_capacity(512);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let raw_name = e.name().into_inner();
                let clean_name = raw_name
                    .split(':')
                    .next_back()
                    .unwrap_or(raw_name)
                    .to_string();
                return Ok(Some(clean_name));
            }
            Ok(Event::Eof) => return Ok(None),
            Err(err) => return Err(err),
            _ => {}
        }
        buf.clear();
    }
}

/// Inspeciona os primeiros bytes em memória para identificar a tag raiz sem desserializar tudo.
pub fn peek_root_tag_bytes(bytes: &[u8]) -> XmlParserResult<Option<String>> {
    let mut reader = bytes.into_xml_reader()?;
    match read_root_tag_from_reader(&mut reader) {
        Ok(tag) => Ok(tag),
        Err(quick_xml::Error::Encoding(_)) => {
            let mut decoder = quick_xml::encoding::DecodingReader::new(bytes);
            decoder.set_encoding(encoding_rs::WINDOWS_1252);
            let mut fallback = quick_xml::Reader::from_reader(decoder);
            Ok(read_root_tag_from_reader(&mut fallback)?)
        }
        Err(err) => Err(XmlParserError::Xml(err)),
    }
}

/// Desserializa uma entrada XML lida em memória (proveniente de arquivo `.zip`).
pub fn get_xml_serialized_from_bytes(
    entry_label: &str,
    bytes: &[u8],
    arguments: &Arguments,
) -> Option<Information> {
    let root_tag = match peek_root_tag_bytes(bytes) {
        Ok(Some(tag)) => tag,
        Ok(None) => return None,
        Err(err) => {
            eprintln!("Aviso: Falha ao ler XML '{entry_label}': {err}");
            return None;
        }
    };

    let path_label = Path::new(entry_label);
    let res = match root_tag.as_str() {
        "cteProc" => CteProc::bytes_to_info(bytes, path_label, arguments),
        "CTe" => Cte::bytes_to_info(bytes, path_label, arguments),
        "GTVeProc" => GtveProc::bytes_to_info(bytes, path_label, arguments),
        "GTVe" => Gtve::bytes_to_info(bytes, path_label, arguments),
        "cteSimpProc" => CteSimpProc::bytes_to_info(bytes, path_label, arguments),
        "CTeSimp" => CteSimp::bytes_to_info(bytes, path_label, arguments),
        "cteOSProc" => CteOsProc::bytes_to_info(bytes, path_label, arguments),
        "CTeOS" => CteOs::bytes_to_info(bytes, path_label, arguments),
        "nfeProc" => NfeProc::bytes_to_info(bytes, path_label, arguments),
        "NFe" => Nfe::bytes_to_info(bytes, path_label, arguments),
        "procEventoCTe" => ProcEventoCte::bytes_to_info(bytes, path_label, arguments),
        "eventoCTe" => cte_evento::Evento::bytes_to_info(bytes, path_label, arguments),
        "procEventoNFe" => ProcEventoNfe::bytes_to_info(bytes, path_label, arguments),
        "evento" => nfe_evento::Evento::bytes_to_info(bytes, path_label, arguments),
        "procCancCTe" => ProcCancCte::bytes_to_info(bytes, path_label, arguments),
        "procCancNFe" => ProcCancNfe::bytes_to_info(bytes, path_label, arguments),
        "eFinanceira" => EFinanceira::bytes_to_info(bytes, path_label, arguments),
        _ => return None,
    };

    res.ok()
        .filter(Information::is_some)
        .and_then(crate::filter_information)
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_info
#[cfg(test)]
mod tests_info {
    use super::*;

    #[test]
    fn test_peek_root_tag_com_namespace_e_declaracao() {
        let xml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- Comentario inicial -->\n<nfe:nfeProc versao=\"4.00\" xmlns:nfe=\"http://www.portalfiscal.inf.br/nfe\"><nfe:NFe></nfe:NFe></nfe:nfeProc>";
        let tag = peek_root_tag_bytes(xml).unwrap();
        assert_eq!(tag, Some("nfeProc".to_string()));
    }
}
