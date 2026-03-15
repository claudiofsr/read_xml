use std::path::PathBuf;
use thiserror::Error;

/// Alias para Resultados padrão do projeto usando nosso Enum
// O '= ()' diz ao Rust: "Se eu não disser o que é T, assuma que é 'nada' (unit type)"
pub type XmlParserResult<T = ()> = Result<T, XmlParserError>;

#[derive(Error, Debug)]
pub enum XmlParserError {
    /// Erro vindo da biblioteca CSV
    #[error("Erro no processamento de CSV: {0}")]
    Csv(#[from] csv::Error),

    /// Erro vindo da biblioteca de Excel
    #[error("Erro ao gerar planilha Excel: {0}")]
    Excel(#[from] rust_xlsxwriter::XlsxError),

    /// Versão simples: Permite que `?` funcione em qualquer comando de I/O (std::io::Error)
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    /// Versão com Contexto: Usada manualmente com .map_err para indicar QUAL arquivo falhou
    #[error("Falha no arquivo {path}: {source}")]
    IoContext {
        source: std::io::Error,
        path: PathBuf,
    },

    /// Erro genérico para validações lógicas (ex: chave inválida)
    #[error("Documento inválido: {0}")]
    InvalidDocument(String),

    /// Erro vindo do crate de Excel ou CSV
    #[error("Erro ao exportar dados: {0}")]
    Export(String),

    /// Erro de parse de datas
    #[error("Data inválida: {0}")]
    DateParse(String),

    /// Erro de conversão de tipos numéricos (ex: usize para u16/u32)
    #[error("Valor numérico fora do limite permitido: {0}")]
    NumericConversion(#[from] std::num::TryFromIntError),

    /// Erro na sintaxe do template da barra de progresso (indicatif)
    #[error("Erro no template da barra de progresso: {0}")]
    ProgressBarTemplate(#[from] indicatif::style::TemplateError),

    /// Erros de leitura de baixo nível do quick-xml (Reader)
    #[error("Erro de leitura XML: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Erros de desserialização (Serde) do quick-xml
    #[error("Erro na estrutura/desserialização do XML: {0}")]
    XmlDe(#[from] quick_xml::DeError),

    /// Caso precise capturar erros de bibliotecas que ainda usam Box
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
