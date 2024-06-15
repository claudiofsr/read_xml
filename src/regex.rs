use once_cell::sync::Lazy;
use regex::Regex;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

/// Example:
///
/// <https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html>
pub static REGEX_CANCELAMENTO: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Cancelamento
    ").unwrap()
);

pub static REGEX_CENTER: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        # non-capturing group: (?:regex)
        ^(:?
            CNPJ|CPF|CST|
            Chave|NCM|
            Registro|Identifica|
            Cancelado|
            Estado
        )|
        Código|
        Versão
    ").unwrap()
);

pub static REGEX_VALUE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Total|Valor
    ").unwrap()
);

pub static REGEX_ALIQ: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Alíquota
    ").unwrap()
);

pub static REGEX_DATE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        ^(:?Data|Dia|Ano)
    ").unwrap()
);

pub static REGEX_FIELDS: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?x)
        \b # word boundary
        (:?
            CTe|protCTe|
            NFe|protNFe|
            evento|retEvento|
            eventoCTe|retEventoCTe|
            evtMovOpFin
        )
        \b # word boundary
    ").unwrap()
);

pub static REGEX_ERROR_MISSING_FIELD: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        missing\s*field
    ").unwrap()
);

pub static REGEX_ERROR_DUPLICATE_FIELD: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        duplicate\s*field
    ").unwrap()
);