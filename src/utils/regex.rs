use regex::{Regex, RegexSet};
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
//        Regex de Formatacao de colunas de arquivos xlsx                     //
//----------------------------------------------------------------------------//

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

/// Vincula uma chave de estilo de célula ao seu respectivo validador Regex.
pub struct FormatRule {
    pub key: &'static str,
    pub regex: &'static Lazy<Regex>,
}

/// Registro único de regras de formatação do projeto.
///
/// Modificar, adicionar ou reordenar itens aqui atualiza automaticamente
/// a ordem de compilação do RegexSet e o mapeamento de estilos.
pub static FORMAT_RULES: [FormatRule; 4] = [
    FormatRule {
        key: "center",
        regex: &REGEX_CENTER,
    },
    FormatRule {
        key: "value",
        regex: &REGEX_VALUE,
    },
    FormatRule {
        key: "aliq",
        regex: &REGEX_ALIQ,
    },
    FormatRule {
        key: "date",
        regex: &REGEX_DATE,
    },
];

/// O RegexSet é gerado mapeando dinamicamente a ordem definida no array `FORMAT_RULES`.
pub static REGEX_COLUMN_SET: Lazy<RegexSet> = Lazy::new(|| {
    let patterns: Vec<&str> = FORMAT_RULES
        .iter()
        .map(|rule| rule.regex.as_str())
        .collect();

    RegexSet::new(patterns).expect("Failed to compile REGEX_COLUMN_SET static regex set.")
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

//----------------------------------------------------------------------------//
//                          Tests for Format Rules                            //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_format_rules
#[cfg(test)]
mod tests_format_rules {
    use super::*;

    // Test 1: Garante que o RegexSet compila com sucesso e possui o mesmo
    // número de padrões que o nosso array central de regras de formatação.
    #[test]
    fn test_regex_column_set_compiles() {
        let _ = &*REGEX_COLUMN_SET; // Força a inicialização preguiçosa
        assert_eq!(
            REGEX_COLUMN_SET.len(),
            FORMAT_RULES.len(),
            "O número de padrões no RegexSet difere do tamanho de FORMAT_RULES."
        );
    }

    // Test 2: Garante a correspondência exata dos índices retornados pelo RegexSet
    // com as chaves lógicas mapeadas em FORMAT_RULES.
    #[test]
    fn test_format_rules_matching_indexes() {
        // Casos de teste mapeando: (Texto de entrada, Chave esperada, Índice esperado)
        let test_cases = [
            ("CNPJ da Empresa", "center", 0),
            ("Valor do ICMS", "value", 1),
            ("Alíquota do Simples Nacional", "aliq", 2),
            ("Data de Emissão do Documento", "date", 3),
        ];

        for (input, expected_key, expected_index) in test_cases {
            let matches: Vec<usize> = REGEX_COLUMN_SET.matches(input).iter().collect();

            // Garante que houve correspondência
            assert!(
                !matches.is_empty(),
                "Falha: A entrada '{input}' deveria ter casado com algum padrão."
            );

            // Garante que o primeiro casamento corresponde ao índice esperado
            let matched_index = matches[0];
            assert_eq!(
                matched_index, expected_index,
                "Erro de correspondência: A entrada '{input}' retornou o índice {matched_index}, mas esperava-se {expected_index}."
            );

            // Garante que o índice correspondido aponta para a chave textual correta
            assert_eq!(
                FORMAT_RULES[matched_index].key, expected_key,
                "Desalinhamento: O índice {matched_index} aponta para a chave '{}', mas esperava-se '{}'.",
                FORMAT_RULES[matched_index].key, expected_key
            );
        }
    }

    // Test 3: Garante que strings neutras que não possuem palavras-chave
    // retornem um conjunto vazio de correspondências (nenhum índice ativo).
    #[test]
    fn test_regex_column_set_no_match() {
        let input = "Texto genérico sem informações fiscais ou numéricas";
        let matches = REGEX_COLUMN_SET.matches(input);

        assert!(
            matches.iter().next().is_none(),
            "Erro: Uma entrada neutra não deveria ter casado com nenhuma regra."
        );
    }
}
