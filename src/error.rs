use std::error::Error;

/// Alias para erros dinâmicos que podem ser enviados entre threads de forma segura.
pub type XmlParserError = Box<dyn Error + Send + Sync + 'static>;

/// Alias para Resultados padrão do projeto.
pub type XmlParserResult<T> = Result<T, XmlParserError>;
