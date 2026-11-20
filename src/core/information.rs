//! # Centralização de Documentos Fiscais e Despacho de Parsers
//!
//! Define a taxonomia unificada de documentos (`Information`) e a trait de abstração
//! de desserialização (`StructExtension`).

use quick_xml::events::Event;
use rayon::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::Path;
use walkdir::DirEntry;

use crate::{
    Arguments, MultiProgressBar, REGEX_ERROR_DUPLICATE_FIELD, REGEX_ERROR_MISSING_FIELD,
    REGEX_FIELDS, XmlParserError, XmlParserResult, XmlPathExt,
    xml_structs::{
        cancelamento_cte::{InfoCteCancel, ProcCancCte},
        cancelamento_nfe::{InfoNfeCancel, ProcCancNfe},
        cte::{Cte, CteProc, InfoCte},
        cte_evento::{InfoCteEvento, ProcEventoCte},
        efinanceira::{EFinanceira, InfoEFinanceira},
        nfe::{InfoNfe, Nfe, NfeProc},
        nfe_evento::{InfoNfeEvento, ProcEventoNfe},
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

    /// Tenta o parse e diagnostica anomalias de schema em caso de falha estrutural.
    fn struct_to_info(xml_path: &Path, arguments: &Arguments) -> XmlParserResult<Information> {
        match Self::xml_parse(xml_path) {
            Ok(proc) => Ok(proc.get_information(xml_path, arguments)),
            Err(err) => Self::diagnosticar_erro_de_schema(&err, xml_path),
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
            eprintln!("  read_xml -s {:?}\n", xml_path);
        }

        // Retorna None permitindo que outros parsers tentem ou que o fluxo continue
        Ok(Information::None)
    }
}

/// Inspeciona os primeiros bytes do XML para identificar a tag raiz sem carregar
/// ou desserializar o documento inteiro em memória.
pub fn peek_root_tag(path: &Path) -> XmlParserResult<Option<String>> {
    let mut reader = path.open_xml_reader_with_capacity(2048)?;
    let mut buf = Vec::with_capacity(512);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let raw_name = e.name().into_inner();
                // Remove namespace se houver (ex: "nfe:nfeProc" -> "nfeProc")
                let clean_name = raw_name
                    .split(':')
                    .next_back()
                    .unwrap_or(raw_name)
                    .to_string();
                return Ok(Some(clean_name));
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(XmlParserError::Xml(err)),
            _ => {}
        }
        buf.clear();
    }

    Ok(None)
}

pub fn get_xml_serialized(xml_path: &Path, arguments: &Arguments) -> Option<Information> {
    // Propaga ou registra caso haja um erro real de I/O
    let root_tag = match peek_root_tag(xml_path) {
        Ok(Some(tag)) => tag,
        Ok(None) => return None,
        Err(err) => {
            eprintln!("Aviso: Falha de I/O ao ler {:?}: {}", xml_path, err);
            return None;
        }
    };

    let res = match root_tag.as_str() {
        "cteProc" => CteProc::struct_to_info(xml_path, arguments),
        "CTe" => Cte::struct_to_info(xml_path, arguments),
        "nfeProc" => NfeProc::struct_to_info(xml_path, arguments),
        "NFe" => Nfe::struct_to_info(xml_path, arguments),
        "procEventoCTe" | "eventoCTe" => ProcEventoCte::struct_to_info(xml_path, arguments),
        "procEventoNFe" | "evento" => ProcEventoNfe::struct_to_info(xml_path, arguments),
        "procCancCTe" => ProcCancCte::struct_to_info(xml_path, arguments),
        "procCancNFe" => ProcCancNfe::struct_to_info(xml_path, arguments),
        "eFinanceira" => EFinanceira::struct_to_info(xml_path, arguments),
        _ => return None,
    };

    res.ok().filter(Information::is_some)
}

/// Executa a varredura paralela dos arquivos XML reportando progresso.
pub fn get_all_info(
    xml_entries: &[DirEntry],
    multi_progressbar: &mut MultiProgressBar,
    arguments: &Arguments,
) -> Vec<Information> {
    let infos: Vec<Information> = xml_entries
        .par_iter() // rayon parallel iterator
        .filter_map(|entry| {
            multi_progressbar.show_parse.inc(1);
            get_xml_serialized(entry.path(), arguments)
        })
        .collect();

    multi_progressbar.show_parse.finish();
    infos
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_info
#[cfg(test)]
mod tests_info {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_peek_root_tag_com_namespace_e_declaracao() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- Comentario inicial -->\n<nfe:nfeProc versao=\"4.00\" xmlns:nfe=\"http://www.portalfiscal.inf.br/nfe\"><nfe:NFe></nfe:NFe></nfe:nfeProc>"
        ).unwrap();

        let tag = peek_root_tag(temp_file.path()).unwrap();
        assert_eq!(tag, Some("nfeProc".to_string()));
    }
}
