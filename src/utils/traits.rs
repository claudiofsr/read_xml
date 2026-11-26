use chrono::{Datelike, NaiveDate};
use claudiofsr_lib::StrExtension;
use quick_xml::{DeError, Reader, de::from_reader, encoding::DecodingReader, events::Event};
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::{
    cmp::Ord,
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{self, BufRead, BufReader},
    path::Path,
};

use crate::{XmlParserError, XmlParserResult};

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

    /// Retorna a chave de forma sanitizada (sem espaços nas bordas e em caixa alta).
    ///
    /// Retorna `None` caso o resultado da sanitização seja uma string vazia,
    /// evitando alocações desnecessárias na Heap.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use read_xml::OptExt;
    ///
    /// let opt_mixed: Option<&str> = Some("  ChAvE  ");
    /// assert_eq!(opt_mixed.get_key(), Some("CHAVE".to_string()));
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
        normalize_key(self.as_ref()?.as_ref())
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

// ---------------------------------------------------------------------------//

/// Verifica em passada única (zero-allocation) se a string possui bordas com espaços,
/// quebras de linha ou tabulações.
#[inline]
fn needs_line_break_strip(s: &str) -> bool {
    s.starts_with(char::is_whitespace)
        || s.ends_with(char::is_whitespace)
        || s.contains(['\r', '\n', '\t'])
}

/// Verifica em passada única (zero-allocation) se a string possui bordas com espaços,
/// quebras de linha, tabulações ou dois ou mais espaços consecutivos.
#[inline]
fn needs_space_collapse(s: &str) -> bool {
    let mut prev_ws = false;
    let mut first = true;
    let mut last_ws = false;

    for ch in s.chars() {
        if matches!(ch, '\r' | '\n' | '\t') {
            return true;
        }
        let is_ws = ch.is_whitespace();
        if (first && is_ws) || (prev_ws && is_ws) {
            return true;
        }
        first = false;
        prev_ws = is_ws;
        last_ws = is_ws;
    }

    last_ws
}

/// Remove quebras de linha (`\r\n`, `\r`, `\n`) e tabulações (`\t`),
/// substituindo-as por espaço simples e aparando bordas (preserva múltiplos espaços internos).
#[inline]
pub fn strip_line_breaks(s: &str) -> String {
    s.trim()
        .replace("\r\n", " ")
        .replace(['\r', '\n', '\t'], " ")
}

/// Normaliza uma chave fiscal ou identificador em caixa alta (padrão oficial SEFAZ/CNPJ alfanumérico),
/// removendo espaços nas bordas e retornando `None` se vazia.
#[inline]
pub fn normalize_key(raw: &str) -> Option<String> {
    let s = raw.trim();
    (!s.is_empty()).then(|| s.to_uppercase())
}

/// Formata CNPJ numérico ou alfanumérico (NT 2026.004) em caixa alta (`XX.XXX.XXX/XXXX-XX`).
pub fn format_cnpj_alnum(raw: &str) -> String {
    let trimmed = raw.trim();
    let clean: String = trimmed
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    if clean.len() == 14 && clean.bytes().any(|b| b.is_ascii_alphabetic()) {
        let upper = clean.to_ascii_uppercase();
        return format!(
            "{}.{}.{}/{}-{}",
            &upper[0..2],
            &upper[2..5],
            &upper[5..8],
            &upper[8..12],
            &upper[12..14]
        );
    }
    trimmed.format_cnpj()
}

/// Extrai a chave de acesso de 44 dígitos em caixa alta a partir do atributo `@Id`
/// (ex.: `"CTe4126..."` ou `"NFe3525..."`), removendo o prefixo sem diferenciar maiúsculas/minúsculas.
#[inline]
pub fn extract_chave_from_id(id: Option<&str>, prefix: &str) -> Option<String> {
    let raw = id?.trim();
    let rest = match raw.get(..prefix.len()) {
        Some(head) if head.eq_ignore_ascii_case(prefix) => &raw[prefix.len()..],
        _ => raw,
    };
    normalize_key(rest)
}

/// Normalização textual em lote para campos e coleções de documentos fiscais.
pub trait SanitizeText {
    /// Remove quebras de linha, tabulações e espaços nas bordas, preservando múltiplos espaços internos.
    fn strip_line_breaks(&mut self);

    /// Remove quebras de linha, tabulações, espaços nas bordas e colapsa 2+ espaços em 1 espaço.
    fn clean_spaces(&mut self);
}

impl SanitizeText for String {
    #[inline]
    fn strip_line_breaks(&mut self) {
        if needs_line_break_strip(self) {
            *self = strip_line_breaks(self);
        }
    }

    #[inline]
    fn clean_spaces(&mut self) {
        if needs_space_collapse(self) {
            *self = self.split_whitespace().collect::<Vec<_>>().join(" ");
        }
    }
}

impl<T: SanitizeText> SanitizeText for Option<T> {
    #[inline]
    fn strip_line_breaks(&mut self) {
        if let Some(val) = self {
            val.strip_line_breaks();
        }
    }

    #[inline]
    fn clean_spaces(&mut self) {
        if let Some(val) = self {
            val.clean_spaces();
        }
    }
}

impl<T: SanitizeText> SanitizeText for Vec<T> {
    #[inline]
    fn strip_line_breaks(&mut self) {
        for item in self {
            item.strip_line_breaks();
        }
    }

    #[inline]
    fn clean_spaces(&mut self) {
        for item in self {
            item.clean_spaces();
        }
    }
}

/// Extensão paralela para normalização textual em lote sobre slices.
pub trait SanitizeSliceExt {
    fn strip_line_breaks_par(&mut self);
    fn clean_spaces_par(&mut self);
}

impl<T: SanitizeText + Send> SanitizeSliceExt for [T] {
    fn strip_line_breaks_par(&mut self) {
        self.par_iter_mut()
            .for_each(SanitizeText::strip_line_breaks);
    }

    fn clean_spaces_par(&mut self) {
        self.par_iter_mut().for_each(SanitizeText::clean_spaces);
    }
}

/// Macro DRY que gera `impl SanitizeText` a partir de uma única lista de campos textuais.
#[macro_export]
macro_rules! impl_sanitize_text {
    ($ty:ty, $( $field:ident ),+ $(,)?) => {
        impl $crate::SanitizeText for $ty {
            fn strip_line_breaks(&mut self) {
                $( $crate::SanitizeText::strip_line_breaks(&mut self.$field); )+
            }

            fn clean_spaces(&mut self) {
                $( $crate::SanitizeText::clean_spaces(&mut self.$field); )+
            }
        }
    };
}

#[cfg(test)]
mod tests_sanitize_text {
    use super::*;

    #[test]
    fn test_strip_line_breaks_vs_clean_spaces() {
        let raw = "  Linha 1\r\nLinha  2\tFim   ";

        let mut s1 = raw.to_string();
        s1.strip_line_breaks();
        assert_eq!(s1, "Linha 1 Linha  2 Fim");

        let mut s2 = raw.to_string();
        s2.clean_spaces();
        assert_eq!(s2, "Linha 1 Linha 2 Fim");
    }
}

// ---------------------------------------------------------------------------//

pub type XmlDecodingReader<R = BufReader<File>> = DecodingReader<R>;
pub type XmlEventReader<R = BufReader<File>> = Reader<DecodingReader<R>>;

/// Extensão para qualquer fonte `BufRead` (`&[u8]`, `BufReader<File>`, entradas de `.zip`, etc.),
/// detectando automaticamente BOM (`UTF-8`/`UTF-16`) e declaração `<?xml ... encoding="..."?>`.
pub trait XmlBufReadExt: BufRead + Sized {
    /// Envolve o stream em um `DecodingReader` configurado com o encoding do XML.
    fn into_decoding_reader(mut self) -> io::Result<DecodingReader<Self>> {
        let enc = {
            // Espia os bytes iniciais do buffer sem consumi-los.
            let peek = self.fill_buf()?;
            let trimmed = peek
                .strip_prefix(b"\xEF\xBB\xBF")
                .unwrap_or(peek)
                .trim_ascii_start();

            if trimmed.starts_with(b"<?xml") {
                // Uma declaração <?xml ...?> válida cabe com folga nos primeiros 4 KB
                // e é sempre o 1º evento a partir de `trimmed`.
                let head = &trimmed[..trimmed.len().min(4096)];
                let mut probe = Reader::from_reader(head);
                let mut buf = Vec::with_capacity(64);

                if let Ok(Event::Decl(decl)) = probe.read_event_into(&mut buf) {
                    if let Some(Ok(label)) = decl.encoding()
                        && encoding_rs::Encoding::for_label(label.as_bytes()).is_none()
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Encoding XML desconhecido ou não suportado: {label}"),
                        ));
                    }
                    Some(probe.encoding())
                } else {
                    None
                }
            } else {
                None
            }
        };

        let mut decoder = DecodingReader::new(self);

        // Só sobrescreve quando a declaração XML indica explicitamente um encoding não-UTF-8;
        // para UTF-8 e UTF-16 (via BOM/padrão), o próprio `DecodingReader` já resolve.
        if let Some(enc) = enc.filter(|e| e.name() != "UTF-8") {
            decoder.set_encoding(enc);
        }

        Ok(decoder)
    }

    /// Envolve o stream em um `quick_xml::Reader` pronto para leitura por eventos.
    fn into_xml_reader(self) -> io::Result<Reader<DecodingReader<Self>>> {
        Ok(Reader::from_reader(self.into_decoding_reader()?))
    }
}

impl<R: BufRead> XmlBufReadExt for R {}

/// Extensão ergonômica para caminhos de arquivo (`&Path`, `PathBuf`, `&str`).
pub trait XmlPathExt {
    fn open_decoding_reader(&self) -> XmlParserResult<XmlDecodingReader>;
    fn open_decoding_reader_with_capacity(
        &self,
        capacity: usize,
    ) -> XmlParserResult<XmlDecodingReader>;

    fn open_xml_reader(&self) -> XmlParserResult<XmlEventReader>;
    fn open_xml_reader_with_capacity(&self, capacity: usize) -> XmlParserResult<XmlEventReader>;

    /// Desserializa um arquivo XML com buffer padrão de 64 KB,
    /// detecção automática de encoding e fallback seguro para Windows-1252.
    fn parse_xml<T: DeserializeOwned>(&self) -> XmlParserResult<T>;

    /// Desserializa um arquivo XML com capacidade de buffer customizada,
    /// detecção automática de encoding e fallback seguro para Windows-1252.
    fn parse_xml_with_capacity<T: DeserializeOwned>(&self, capacity: usize) -> XmlParserResult<T>;
}

impl<P: AsRef<Path> + ?Sized> XmlPathExt for P {
    fn open_decoding_reader(&self) -> XmlParserResult<XmlDecodingReader> {
        self.open_decoding_reader_with_capacity(8 * 1024)
    }

    fn open_decoding_reader_with_capacity(
        &self,
        capacity: usize,
    ) -> XmlParserResult<XmlDecodingReader> {
        let path = self.as_ref();
        let file = File::open(path).map_err(|err| XmlParserError::IoContext {
            source: err,
            path: path.to_path_buf(),
        })?;

        let buf_reader = BufReader::with_capacity(capacity, file);
        buf_reader
            .into_decoding_reader()
            .map_err(|err| XmlParserError::IoContext {
                source: err,
                path: path.to_path_buf(),
            })
    }

    fn open_xml_reader(&self) -> XmlParserResult<XmlEventReader> {
        Ok(Reader::from_reader(self.open_decoding_reader()?))
    }

    fn open_xml_reader_with_capacity(&self, capacity: usize) -> XmlParserResult<XmlEventReader> {
        Ok(Reader::from_reader(
            self.open_decoding_reader_with_capacity(capacity)?,
        ))
    }

    fn parse_xml<T: DeserializeOwned>(&self) -> XmlParserResult<T> {
        self.parse_xml_with_capacity(64 * 1024)
    }

    fn parse_xml_with_capacity<T: DeserializeOwned>(&self, capacity: usize) -> XmlParserResult<T> {
        let decoder = self.open_decoding_reader_with_capacity(capacity)?;

        match from_reader(decoder) {
            Ok(parsed) => Ok(parsed),
            Err(DeError::InvalidXml(quick_xml::Error::Encoding(_))) => {
                // Fallback seguro apenas quando o XML omitiu/mentiu o encoding (ex.: legado Windows-1252)
                let path = self.as_ref();
                let file = File::open(path).map_err(|err| XmlParserError::IoContext {
                    source: err,
                    path: path.to_path_buf(),
                })?;

                let mut decoder = DecodingReader::new(BufReader::with_capacity(capacity, file));
                decoder.set_encoding(encoding_rs::WINDOWS_1252);
                Ok(from_reader(decoder)?)
            }
            Err(err) => Err(err.into()),
        }
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

        assert_eq!(opt_mixed.get_key(), Some("CHAVE_TESTE".to_string()));
        assert_eq!(opt_empty.get_key(), None);
        assert_eq!(opt_none.get_key(), None);
        assert_eq!(format_cnpj_alnum("12abc3450001de"), "12.ABC.345/0001-DE");
        assert_eq!(
            extract_chave_from_id(
                Some("nfe35260112abc3450001de550010000001231000000123"),
                "NFe"
            ),
            Some("35260112ABC3450001DE550010000001231000000123".to_string())
        );
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

#[cfg(test)]
mod tests_xml_edge_cases {
    use super::*;
    use quick_xml::de::from_reader;
    use quick_xml::events::Event;
    use serde::Deserialize;
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct DummyDoc {
        nome: String,
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

    fn to_utf16_le_with_bom(s: &str) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xFE];
        for unit in s.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes
    }

    fn to_utf16_be_with_bom(s: &str) -> Vec<u8> {
        let mut bytes = vec![0xFE, 0xFF];
        for unit in s.encode_utf16() {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
        bytes
    }

    #[test]
    /// Arquivo e slice completamente vazios (0 bytes) não devem causar panic.
    fn edge_empty_file_and_bytes() -> XmlParserResult<()> {
        let empty: &[u8] = b"";

        // Modo eventos: deve retornar Eof imediatamente
        let mut reader = empty.into_xml_reader()?;
        let mut buf = Vec::new();
        assert!(matches!(reader.read_event_into(&mut buf)?, Event::Eof));

        // Modo Serde (bytes): deve retornar Err
        let res_bytes: Result<DummyDoc, _> = from_reader(empty.into_decoding_reader()?);
        assert!(res_bytes.is_err());

        // Modo Serde (arquivo em disco): deve retornar Err
        let tmp = TempXmlFile::new("edge_empty.xml", empty)?;
        assert!(tmp.0.parse_xml::<DummyDoc>().is_err());
        Ok(())
    }

    #[test]
    /// Arquivo contendo apenas espaços, tabs e quebras de linha.
    fn edge_whitespace_only() -> XmlParserResult<()> {
        let ws: &[u8] = b"   \n\t\r\n   ";

        let res: Result<DummyDoc, _> = from_reader(ws.into_decoding_reader()?);
        assert!(res.is_err());

        let mut reader = ws.into_xml_reader()?;
        let mut buf = Vec::new();
        // Pula Event::Text de espaços e chega em Eof sem erro nem panic
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Eof => break,
                _ => buf.clear(),
            }
        }
        Ok(())
    }

    #[test]
    /// Caminho inexistente deve retornar XmlParserError::IoContext sem panic.
    fn edge_nonexistent_file() {
        let fake_path = Path::new("/caminho/que/nao/existe/xml_fantasma_12345.xml");
        let res = fake_path.parse_xml::<DummyDoc>();

        assert!(matches!(res, Err(XmlParserError::IoContext { .. })));
    }

    #[test]
    /// Texto comum que não é XML.
    fn edge_plain_text_not_xml() -> XmlParserResult<()> {
        let txt: &[u8] = b"Isso definitivamente nao e um XML valido.";
        let res: Result<DummyDoc, _> = from_reader(txt.into_decoding_reader()?);
        assert!(res.is_err());

        let tmp = TempXmlFile::new("edge_plain_text.xml", txt)?;
        assert!(tmp.0.parse_xml::<DummyDoc>().is_err());
        Ok(())
    }

    #[test]
    /// Arquivo binário (ex.: assinatura de PNG / bytes nulos) disfarçado de .xml.
    fn edge_binary_garbage_disguised_as_xml() -> XmlParserResult<()> {
        let fake_png: &[u8] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\xFF\xFE\xFD\x00\x81\x8D";

        let res: Result<DummyDoc, _> = from_reader(fake_png.into_decoding_reader()?);
        assert!(res.is_err());

        let tmp = TempXmlFile::new("edge_fake_png.xml", fake_png)?;
        assert!(tmp.0.parse_xml::<DummyDoc>().is_err());
        Ok(())
    }

    #[test]
    /// Declaração <?xml ... cortada no meio (sem fechamento '?>').
    fn edge_truncated_xml_declaration() -> XmlParserResult<()> {
        let truncated_decl: &[u8] = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1";

        let res: Result<DummyDoc, _> = from_reader(truncated_decl.into_decoding_reader()?);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    /// XML ISO-8859-1 truncado no meio do conteúdo (sem fechar as tags).
    fn edge_truncated_body_iso_8859_1() -> XmlParserResult<()> {
        let truncated_body: &[u8] =
            b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><Doc><Nome>Jo\xE3o";

        let res: Result<DummyDoc, _> = from_reader(truncated_body.into_decoding_reader()?);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    /// Tags de abertura e fechamento incompatíveis (</Outro> fechando <Nome>).
    fn edge_mismatched_tags() -> XmlParserResult<()> {
        let mismatched: &[u8] = b"<Doc><Nome>Teste</Outro></Doc>";

        let res: Result<DummyDoc, _> = from_reader(mismatched.into_decoding_reader()?);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    /// Encoding inventado/absurdo deve ser rejeitado (inclusive pelo fallback em arquivo).
    fn edge_absurd_unknown_encoding() -> XmlParserResult<()> {
        let absurd: &[u8] =
            b"<?xml version=\"1.0\" encoding=\"Klingon-9000\"?><Doc><Nome>Oi</Nome></Doc>";

        assert!(absurd.into_decoding_reader().is_err());

        let tmp = TempXmlFile::new("edge_absurd_enc.xml", absurd)?;
        assert!(tmp.0.parse_xml::<DummyDoc>().is_err());
        Ok(())
    }

    #[test]
    /// UTF-8 com BOM (\xEF\xBB\xBF) e espaços antes de <?xml ...?>.
    fn edge_utf8_with_bom_and_leading_whitespace() -> XmlParserResult<()> {
        let xml_bom: &[u8] =
            b"\xEF\xBB\xBF  \n<?xml version=\"1.0\" encoding=\"UTF-8\"?><Doc><Nome>A\xC3\xA7\xC3\xA3o</Nome></Doc>";

        let doc: DummyDoc = from_reader(xml_bom.into_decoding_reader()?)?;
        assert_eq!(doc.nome, "Ação");
        Ok(())
    }

    #[test]
    /// Fast-path da micro-otimização: XML UTF-8 começando direto na tag raiz (sem <?xml ...?>).
    fn edge_utf8_without_xml_decl_fast_path() -> XmlParserResult<()> {
        let xml_no_decl: &[u8] = b"  \n<Doc><Nome>Sem cabe\xC3\xA7alho</Nome></Doc>";

        let doc: DummyDoc = from_reader(xml_no_decl.into_decoding_reader()?)?;
        assert_eq!(doc.nome, "Sem cabeçalho");
        Ok(())
    }

    #[test]
    /// UTF-16LE e UTF-16BE com BOM devem funcionar via into_decoding_reader e parse_xml.
    fn edge_utf16_le_and_be_with_bom() -> XmlParserResult<()> {
        let raw = r#"<?xml version="1.0" encoding="UTF-16"?><Doc><Nome>Ação UTF-16 €</Nome></Doc>"#;

        let le_bytes = to_utf16_le_with_bom(raw);
        let doc_le: DummyDoc = from_reader(le_bytes.as_slice().into_decoding_reader()?)?;
        assert_eq!(doc_le.nome, "Ação UTF-16 €");

        let be_bytes = to_utf16_be_with_bom(raw);
        let tmp_be = TempXmlFile::new("edge_utf16_be.xml", &be_bytes)?;
        let doc_be: DummyDoc = tmp_be.0.parse_xml()?;
        assert_eq!(doc_be.nome, "Ação UTF-16 €");
        Ok(())
    }
}
