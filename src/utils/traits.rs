use chrono::{Datelike, NaiveDate};
use claudiofsr_lib::StrExtension;
use rayon::prelude::*;
use std::{cmp::Ord, collections::HashMap, hash::Hash};

pub trait GetFirst {
    /**
        Get first chars from `Option<T>`.

        Example:
        ```
        use read_xml::GetFirst;

        let string = String::from("12.345.678/1000-99");
        let cnpj: Option<String> = Some(string);
        let cnpj_base: Option<String> = cnpj.get_first(10);

        assert_eq!(cnpj_base, Some("12.345.678".to_string()));
        ```
    */
    fn get_first(&self, num: usize) -> Option<String>;
}

impl GetFirst for Option<String> {
    fn get_first(&self, num: usize) -> Option<String> {
        self.as_ref()
            .map(|cnpj| cnpj.get_first_n_chars(num).to_string())
    }
}

/// Trait marcadora local para identificar tipos que representam strings nativas de forma segura.
///
/// Como esta trait é privada e local ao nosso crate, o compilador garante que
/// nenhum tipo de terceiros (como `NaiveDate`) poderá implementá-la, eliminando
/// conflitos de coerência de forma totalmente segura no Rust estável.
trait IsString: AsRef<str> {}

impl IsString for String {}
impl IsString for &str {}

/// Trait de extensão para enriquecer objetos `Option<T>` contendo tipos textuais.
///
/// Oferece métodos utilitários de alta performance para sanitização, contagem
/// de caracteres e conversões numéricas sem alocações desnecessárias.
pub trait OptExt {
    /// Retorna a contagem de caracteres (e não de bytes) contidos na string, se presente.
    ///
    /// Esta operação é executada em tempo linear $O(N)$ sobre a quantidade de caracteres,
    /// sem realizar nenhuma alocação de memória na Heap.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use read_xml::OptExt;
    ///
    /// let opt_str: Option<&str> = Some("Olá Atenção!");
    /// assert_eq!(opt_str.count(), 12);
    /// ```
    fn count(&self) -> usize;

    /// Tenta converter a string para um ponto flutuante `f64`.
    ///
    /// Este método é compatível tanto com o formato padrão (ponto como separador decimal)
    /// quanto com o formato brasileiro (ponto como separador de milhar e vírgula como
    /// separador decimal).
    ///
    /// # Detalhes de Performance
    /// - **Sem Alocação**: Se a string não contiver vírgulas (formato internacional padrão),
    ///   a conversão é realizada diretamente sobre a fatia de string original, sem alocar memória na Heap.
    /// - **Alocação Única**: Caso precise remover pontos de milhar e converter a vírgula,
    ///   uma nova `String` é alocada com capacidade pré-definida para evitar realocações durante o processamento.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use read_xml::OptExt;
    ///
    /// // Exemplos de formatação monetária brasileira:
    /// assert_eq!(Some("2,03").to_float64(), Some(2.03));
    /// assert_eq!(Some("1.234.567,56").to_float64(), Some(1234567.56));
    /// assert_eq!(Some("25,63").to_float64(), Some(25.63));
    /// assert_eq!(Some("1.234,03").to_float64(), Some(1234.03));
    /// assert_eq!(Some("-1.234,03").to_float64(), Some(-1234.03));
    ///
    /// // Exemplo no formato padrão (sem alocações adicionais):
    /// assert_eq!(Some("1234.56").to_float64(), Some(1234.56));
    /// ```
    fn to_float64(&self) -> Option<f64>;

    /// Retorna a chave de forma sanitizada (sem espaços nas bordas e em caixa baixa).
    ///
    /// Retorna `None` caso o resultado da sanitização seja uma string vazia,
    /// evitando alocar memória na Heap para strings sem conteúdo.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use read_xml::OptExt;
    ///
    /// let opt_mixed: Option<&str> = Some("  ChAvE  ");
    /// assert_eq!(opt_mixed.get_key(), Some("chave".to_string()));
    /// ```
    fn get_key(&self) -> Option<String>;

    /// Retorna a string sanitizada (sem espaços nas bordas).
    ///
    /// Retorna `None` caso o resultado da sanitização seja uma string vazia,
    /// evitando alocações desnecessárias.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use read_xml::OptExt;
    ///
    /// let opt_spaces: Option<&str> = Some("  Conteúdo  ");
    /// assert_eq!(opt_spaces.get_not_empty(), Some("Conteúdo".to_string()));
    /// ```
    fn get_not_empty(&self) -> Option<String>;
}

// -----------------------------------------------------------------------------
// Implementação para tipos textuais (String, &str)
// -----------------------------------------------------------------------------
impl<T> OptExt for Option<T>
where
    T: IsString,
{
    #[inline]
    fn count(&self) -> usize {
        self.as_ref()
            .map(|s| s.as_ref().chars().count())
            .unwrap_or_default()
    }

    #[inline]
    fn to_float64(&self) -> Option<f64> {
        let s = self.as_ref()?.as_ref().trim();

        if s.is_empty() {
            return None;
        }

        // Remove separadores de milhar '.' e substitui ',' por '.'
        if s.contains(',') {
            // Formato BR: "1.234.567,89" ou "1234,89"
            let mut buffer = String::with_capacity(s.len());
            for c in s.chars() {
                match c {
                    ',' => buffer.push('.'),
                    '.' => {} // descarta separador de milhar
                    _ => buffer.push(c),
                }
            }
            buffer.parse::<f64>().ok()
        } else {
            // Formato ISO: "1234567.89" (Zero-allocation)
            s.parse::<f64>().ok()
        }
    }

    #[inline]
    fn get_key(&self) -> Option<String> {
        let s = self.as_ref()?.as_ref().trim();
        (!s.is_empty()).then(|| s.to_lowercase())
    }

    #[inline]
    fn get_not_empty(&self) -> Option<String> {
        let s = self.as_ref()?.as_ref().trim();
        (!s.is_empty()).then(|| s.to_string())
    }
}

// -----------------------------------------------------------------------------
// Implementação para NaiveDate (Dinâmica, Matemática e sem Alocações na Heap)
// -----------------------------------------------------------------------------
impl OptExt for Option<NaiveDate> {
    #[inline]
    fn count(&self) -> usize {
        match self {
            Some(date) => {
                let year = date.year();

                // Número real de dígitos do valor absoluto do ano (ex: 10250 -> 5 dígitos)
                let digits = year.unsigned_abs().checked_ilog10().unwrap_or(0) as usize + 1;

                // O Chrono prefixa com "+" anos maiores que 9999 (ex: +10250) e com "-" anos negativos (ex: -0004)
                let sign_char = if !(0..=9999).contains(&year) { 1 } else { 0 };

                // O Chrono sempre preenche o ano com zeros à esquerda para ter no mínimo 4 dígitos (ex: 99 -> "0099")
                let year_chars = digits.max(4) + sign_char;

                // Comprimento total = Caracteres do Ano + hífen (1) + Mês (2) + hífen (1) + Dia (2)
                year_chars + 6
            }
            None => 0,
        }
    }

    #[inline]
    fn to_float64(&self) -> Option<f64> {
        None
    }

    #[inline]
    fn get_key(&self) -> Option<String> {
        self.map(|date| date.to_string())
    }

    #[inline]
    fn get_not_empty(&self) -> Option<String> {
        self.map(|date| date.to_string())
    }
}

pub trait GroupBy<K, V>
where
    K: Clone + Eq + PartialEq + Ord + PartialOrd + Hash + Send + Sync,
    V: Clone + Copy + std::ops::AddAssign + Send + Sync,
{
    /// Map Reduce -> HashMap
    fn group_by_key(&self) -> HashMap<K, V>;
}

impl<K, V> GroupBy<K, V> for [(K, V)]
where
    K: Clone + Eq + PartialEq + Ord + PartialOrd + Hash + Send + Sync,
    V: Clone + Copy + std::ops::AddAssign + Send + Sync,
{
    fn group_by_key(&self) -> HashMap<K, V> {
        self.par_iter() // rayon: parallel iterator
            .map(ToOwned::to_owned)
            .fold(HashMap::new, |mut accumulator, (key, value)| {
                accumulator
                    .entry(key)
                    .and_modify(|previous_value| *previous_value += value)
                    .or_insert(value);
                accumulator
            })
            .reduce(HashMap::new, |mut hashmap_acc, hashmap| {
                hashmap.into_iter().for_each(|(key_b, value_b)| {
                    hashmap_acc
                        .entry(key_b)
                        .and_modify(|previous_value| *previous_value += value_b)
                        .or_insert(value_b);
                });
                hashmap_acc
            })
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output traits_group_by_key
#[cfg(test)]
mod traits_group_by_key {
    use super::*;

    #[test]
    /// `cargo test -- --show-output group_tuples`
    fn group_tuples() {
        let tuples = vec![
            ("zz", 5),
            ("ab", 2),
            ("cd", 1),
            ("ab", 2),
            ("cd", 1),
            ("ab", 2),
        ];

        let map_reduce: HashMap<&str, u64> = tuples.group_by_key();

        println!("tuples: {tuples:?}");
        println!("map_reduce: {map_reduce:?}");

        assert_eq!(map_reduce, HashMap::from([("ab", 6), ("cd", 2), ("zz", 5)]));
    }

    #[test]
    /// `cargo test -- --show-output dispach_table`
    ///
    /// <https://stackoverflow.com/questions/51372702/how-do-i-make-a-dispatch-table-in-rust>
    fn dispach_table() {
        let dispatch = {
            let mut temp: HashMap<&str, fn(i32, i32) -> i32> = HashMap::new();
            temp.insert("+", |a, b| a + b);
            temp.insert("-", |a, b| a - b);
            temp
        };

        let plus = dispatch["+"];
        println!("2 + 3 = {}", plus(2, 3));

        let minus = dispatch["-"];
        println!("2 - 3 = {}", minus(2, 3));

        assert_eq!(plus(2, 3), 5);
        assert_eq!(minus(2, 3), -1);
    }
}

/// Run tests with:
/// cargo test -- --show-output tests_opt_ext
#[cfg(test)]
mod tests_opt_ext {
    use super::*;

    #[test]
    fn test_opt_ext_count() {
        let opt_string: Option<String> = Some("12345,67".to_string());
        let opt_str: Option<&str> = Some("1,234 abc ");
        let opt_none: Option<String> = None;

        assert_eq!(opt_string.count(), 8);
        assert_eq!(opt_str.count(), 10);
        assert_eq!(opt_none.count(), 0);
    }

    #[test]
    fn test_opt_ext_to_float64() {
        let opt_comma: Option<String> = Some("12345,67".to_string());
        let opt_dot: Option<&str> = Some("12345.67");
        let opt_invalid: Option<&str> = Some("abc");
        let opt_none: Option<String> = None;

        assert_eq!(opt_comma.to_float64(), Some(12345.67));
        assert_eq!(opt_dot.to_float64(), Some(12345.67));
        assert_eq!(opt_invalid.to_float64(), None);
        assert_eq!(opt_none.to_float64(), None);
    }

    #[test]
    fn test_opt_ext_get_key() {
        let opt_mixed: Option<String> = Some("  ChAvE_TeStE  ".to_string());
        let opt_empty: Option<&str> = Some("   ");
        let opt_none: Option<String> = None;

        assert_eq!(opt_mixed.get_key(), Some("chave_teste".to_string()));
        assert_eq!(opt_empty.get_key(), None);
        assert_eq!(opt_none.get_key(), None);
    }

    #[test]
    fn test_opt_ext_get_not_empty() {
        let opt_spaces: Option<String> = Some("  Conteudo Preservado  ".to_string());
        let opt_empty: Option<&str> = Some("   ");
        let opt_none: Option<String> = None;

        assert_eq!(
            opt_spaces.get_not_empty(),
            Some("Conteudo Preservado".to_string())
        );
        assert_eq!(opt_empty.get_not_empty(), None);
        assert_eq!(opt_none.get_not_empty(), None);
    }

    #[test]
    /// `cargo test -- --show-output test_opt_ext_count_naive_date`
    fn test_opt_ext_count_naive_date() {
        // Ano padrão: "2025-08-12" (10 chars)
        let opt_date_std = NaiveDate::from_ymd_opt(2025, 8, 12);

        // Ano com 2 dígitos: Chrono formata com zeros "0099-01-01" (10 chars)
        let opt_date_pequeno = NaiveDate::from_ymd_opt(99, 1, 1);

        // Ano negativo: "-0004-01-01" (11 chars)
        let opt_date_passado = NaiveDate::from_ymd_opt(-4, 1, 1);

        // Ano com 5 dígitos: "+10250-12-31" (12 chars)
        let opt_date_futuro = NaiveDate::from_ymd_opt(10250, 12, 31);

        let opt_date_none: Option<NaiveDate> = None;

        // Compara o retorno do nosso método matemático com a formatação real de String
        assert_eq!(
            opt_date_std.count(),
            opt_date_std.unwrap().to_string().len()
        );
        assert_eq!(
            opt_date_pequeno.count(),
            opt_date_pequeno.unwrap().to_string().len()
        );
        assert_eq!(
            opt_date_passado.count(),
            opt_date_passado.unwrap().to_string().len()
        );

        println!("opt_date_futuro: {opt_date_futuro:?}");
        let dt_futuro_count = opt_date_futuro.count();
        println!("dt_futuro_count: {dt_futuro_count}");
        let dt_futuro_len = opt_date_futuro.unwrap().to_string().len();
        println!("dt_futuro_len: {dt_futuro_len}");

        // Agora bate perfeitamente com os 12 caracteres (incluindo o "+")
        assert_eq!(dt_futuro_count, dt_futuro_len);

        // Verificando valores diretos absolutos
        assert_eq!(opt_date_std.count(), 10);
        assert_eq!(opt_date_pequeno.count(), 10);
        assert_eq!(opt_date_passado.count(), 11);
        assert_eq!(opt_date_futuro.count(), 12);
        assert_eq!(opt_date_none.count(), 0);
    }

    /// cargo test -- --show-output test_opt_ext_to_float64_brazilian_format
    #[test]
    fn test_opt_ext_to_float64_brazilian_format() {
        let valor1: Option<&str> = Some("2,03");
        let valor2: Option<&str> = Some("1.234.567,56");
        let valor3: Option<&str> = Some("25,63");
        let valor4: Option<&str> = Some("1.234,03");
        let valor5: Option<&str> = Some("-1.234,03");
        let valor6: Option<&str> = Some("  1.234.567,56  ");
        let valor7: Option<&str> = Some("  ");
        let valor8: Option<&str> = Some("");
        let valor9: Option<&str> = Some(" abc ");
        let valor10: Option<&str> = Some(" ab,c ");
        let valor11: Option<&str> = None;

        let valores = [
            valor1, valor2, valor3, valor4, valor5, valor6, valor7, valor8, valor9, valor10,
        ];

        for (i, v) in valores.iter().enumerate() {
            let some_f64 = v.to_float64();
            if let Some(f64) = some_f64 {
                assert!(f64 / 1.0 == f64);
            }
            println!("valor[{n}]: {v:?}", n = i + 1);
        }

        assert_eq!(valor1.to_float64(), Some(2.03));
        assert_eq!(valor2.to_float64(), Some(1234567.56));
        assert_eq!(valor3.to_float64(), Some(25.63));
        assert_eq!(valor4.to_float64(), Some(1234.03));
        assert_eq!(valor5.to_float64(), Some(-1234.03));
        assert_eq!(valor6.to_float64(), Some(1234567.56));
        assert!(valor7.to_float64().is_none());
        assert!(valor8.to_float64().is_none());
        assert!(valor9.to_float64().is_none());
        assert!(valor10.to_float64().is_none());
        assert!(valor11.to_float64().is_none());
    }
}
