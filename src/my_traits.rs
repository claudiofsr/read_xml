use claudiofsr_lib::StrExtension;
use rayon::prelude::*;
use std::{cmp::Ord, collections::HashMap, hash::Hash, ops::Deref};

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

pub trait OptExt<T> {
    /**
        Count chars from `Option<T>`.

        Example:
        ```
        use read_xml::OptExt;

        let string = String::from("12345,67");
        let opt_string: Option<String> = Some(string);
        let opt_str: Option<&str> = Some("1,234 abc ");

        assert_eq!(opt_string.count(), 8);
        assert_eq!(opt_str.count(), 10);
        ```
    */
    fn count(&self) -> usize;

    /**
        Try to Convert `Option<T>` to `Option<f64>`.

        Example:
        ```
        use read_xml::OptExt;

        let string = String::from("12345,67");
        let opt_string: Option<String> = Some(string);

        assert_eq!(opt_string.to_float64(), Some(12345.67));
        ```
    */
    fn to_float64(&self) -> Option<f64>;

    /// Get key after
    ///
    /// `.map(|s| s.trim().to_lowercase())`
    ///
    /// `.filter(|s| !s.is_empty())`
    fn get_key(&self) -> Option<String>
    where
        T: Deref<Target = str>;

    /// Get field after
    ///
    /// `.map(|s| s.trim())`
    ///
    /// `.filter(|s| !s.is_empty())`
    fn get_not_empty(&self) -> Option<String>
    where
        T: Deref<Target = str>;
}

impl<T> OptExt<T> for Option<T>
where
    T: ToString,
{
    fn count(&self) -> usize {
        self.as_ref()
            .map(|var| var.to_string().chars().count())
            .unwrap_or_default()
    }

    fn to_float64(&self) -> Option<f64> {
        self.as_ref()
            .and_then(|var| var.to_string().replace(',', ".").parse::<f64>().ok())
    }

    fn get_key(&self) -> Option<String>
    where
        T: Deref<Target = str>,
    {
        self.as_ref()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
    }

    fn get_not_empty(&self) -> Option<String>
    where
        T: Deref<Target = str>,
    {
        self.as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
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

#[cfg(test)]
mod traits {
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
