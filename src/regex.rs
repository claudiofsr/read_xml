use regex::Regex;
use std::sync::LazyLock as Lazy;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

/// Example:
///
/// <https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html>
pub static REGEX_CANCELAMENTO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        Cancelamento
    ",
    )
    // Call expect or unwrap here to get the Regex value.
    // expect provides a better error message on panic.
    .expect("Failed to compile REGEX_CANCELAMENTO static regex.")
});

pub static REGEX_CENTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        # non-capturing group: (?:regex)
        ^(:? # Anchor the following group to the start of the string
            CNPJ|CPF|CST|
            Chave|NCM|
            Registro|Identifica|
            Cancelado|
            Estado
        ) # End of anchored group
        | # OR (this is an alternative for the whole pattern, not anchored)
        Código
        | # OR
        Versão
    ",
    )
    // Call expect or unwrap here to get the Regex value.
    // expect provides a better error message on panic.
    .expect("Failed to compile REGEX_CENTER static regex.")
});

pub static REGEX_VALUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        Total|Valor
    ",
    )
    .expect("Failed to compile REGEX_VALUE static regex.")
});

pub static REGEX_ALIQ: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        Alíquota
    ",
    )
    .expect("Failed to compile REGEX_ALIQ static regex.")
});

pub static REGEX_DATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        ^(:?Data|Dia|Ano)
    ",
    )
    .expect("Failed to compile REGEX_DATE static regex.")
});

pub static REGEX_FIELDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        \b # word boundary
        (:?
            CTe|protCTe|
            NFe|protNFe|
            evento|retEvento|
            eventoCTe|retEventoCTe|
            evtMovOpFin
        )
        \b # word boundary
    ",
    )
    .expect("Failed to compile REGEX_FIELDS static regex.")
});

pub static REGEX_ERROR_MISSING_FIELD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        missing\s*field
    ",
    )
    .expect("Failed to compile REGEX_ERROR_MISSING_FIELD static regex.")
});

pub static REGEX_ERROR_DUPLICATE_FIELD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        duplicate\s*field
    ",
    )
    .expect("Failed to compile EGEX_ERROR_DUPLICATE_FIELD static regex.")
});

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_regex_center
#[cfg(test)]
mod tests_regex_center {
    use super::*;

    // Test 1: Verify compilation and basic access
    #[test]
    fn test_regex_center_compiles() {
        // Accessing the static variable for the first time triggers
        // the LazyLock's initialization closure.
        // If the expect() call within the closure failed, this test (and program)
        // would panic, indicating a failure in regex compilation.
        // So, simply accessing it successfully is the test.
        let _ = &*REGEX_CENTER; // Dereference LazyLock to get a reference to the Regex
        println!("REGEX_CENTER compiled successfully!"); // Optional: confirm in test output
    }

    // Test 2: Positive matches (strings that *should* match)
    #[test]
    fn test_regex_center_positive_matches() {
        // Keywords that should match when at the start (due to ^)
        assert!(REGEX_CENTER.is_match("CNPJ 12.345.678/0001-90")); // Matches at start
        assert!(REGEX_CENTER.is_match("cpf 123.456.789-00")); // Matches at start (case-insensitive)
        assert!(REGEX_CENTER.is_match("CST 010"));
        assert!(REGEX_CENTER.is_match("CHAVE DA NOTA"));
        assert!(REGEX_CENTER.is_match("NCM 1234.56.78"));
        assert!(REGEX_CENTER.is_match("Registro xyz"));
        assert!(REGEX_CENTER.is_match("Identifica tudo"));
        assert!(REGEX_CENTER.is_match("Cancelado em 2023"));
        assert!(REGEX_CENTER.is_match("Estado: Rio de Janeiro")); // Case-insensitive

        // Keywords that should match anywhere
        assert!(REGEX_CENTER.is_match("Some text with Código within")); // Matches anywhere
        assert!(REGEX_CENTER.is_match("Outra linha com VERSÃO")); // Matches anywhere (case-insensitive)
        assert!(REGEX_CENTER.is_match("O Código está aqui"));
        assert!(REGEX_CENTER.is_match("Versão final"));

        // Edge case: Just the keyword itself
        assert!(REGEX_CENTER.is_match("CNPJ"));
        assert!(REGEX_CENTER.is_match("Código")); // case-insensitive, unanchored
    }

    // Test 3: Negative matches (strings that should *not* match)
    #[test]
    fn test_regex_center_negative_matches() {
        // Strings where anchored keywords appear but *not* at the start
        assert!(!REGEX_CENTER.is_match("Alguma coisa CNPJ 123"));
        assert!(!REGEX_CENTER.is_match("My CPF number: 123"));
        assert!(!REGEX_CENTER.is_match("Text with CST embedded"));

        // Strings without any matching keywords
        assert!(!REGEX_CENTER.is_match("Just a normal sentence."));
        assert!(!REGEX_CENTER.is_match("This has numbers 12345 and text"));
        assert!(!REGEX_CENTER.is_match("Não tem nenhuma das palavras chave"));

        // Strings with words similar to keywords but not an exact match (considering | breaks)
        // The current regex *doesn't* use word boundaries \b, so it will match substrings.
        // e.g., "codigo" will match in "codigos". Let's test this expected behavior
        // and test words that *don't* contain any keyword.
        assert!(!REGEX_CENTER.is_match("Identidade")); // Does not contain "Identifica" as substring case-insensitively
        assert!(!REGEX_CENTER.is_match("VersionDetails")); // Does not contain "Versão" (versao)
        assert!(!REGEX_CENTER.is_match("Estadual")); // Does not contain "Estado" (estado)
    }

    // Test 4: Explicitly check substring matching for unanchored terms (as regex doesn't use \b)
    #[test]
    fn test_regex_center_substring_matches() {
        assert!(REGEX_CENTER.is_match("códigos")); // "código" is a substring
        assert!(REGEX_CENTER.is_match("reversão")); // "versão" is a substring
    }
}
