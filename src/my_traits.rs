use claudiofsr_lib::GetNChars;
use rayon::prelude::*;
use std::{
    cmp::Ord,
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
    ops::Deref,
};

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
        self
            .as_ref()
            .map(|cnpj| {
                cnpj
                    .get_first_n_chars(num)
                    .to_string()
            })
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

pub trait UniqueElements<T> {
    /**
    Get unique and ordered elements from `Vec<T>`.

    Example:
    ```
        use read_xml::UniqueElements;

        let mut items1: Vec<u16> = Vec::new();
        let mut items2: Vec<u32> = vec![1, 3, 1, 2, 2, 4, 3];
        let mut items3: Vec<&str> = vec!["foo", "foo", "bar", "foo"];
        let mut items4: Vec<char> = vec!['f', 'o', 'o', ' ', 'b', 'a', 'r'];

        items1.unique_ordered();
        items2.unique_ordered();
        items3.unique_ordered();
        items4.unique_ordered();

        assert!(items1.is_empty());
        assert_eq!(items1, [0u16;0]);
        assert_eq!(items2, [1, 2, 3, 4]);
        assert_eq!(items3, ["bar", "foo"]);
        assert_eq!(items4, [' ', 'a', 'b', 'f', 'o', 'r']);
    ```
    */
    fn unique_ordered(&mut self);

    /**
    Get unique and ordered elements from `Vec<T>`.

    Example:
    ```
        use read_xml::UniqueElements;

        let mut items1: Vec<u16> = Vec::new();
        let mut items2: Vec<u32> = vec![1, 3, 1, 2, 2, 4, 3];
        let mut items3: Vec<&str> = vec!["foo", "foo", "bar", "foo"];
        let mut items4: Vec<char> = vec!['f', 'o', 'o', ' ', 'b', 'a', 'r'];

        items1 = items1.unique_elements();
        items2 = items2.unique_elements();
        items3 = items3.unique_elements();
        items4 = items4.unique_elements();

        assert!(items1.is_empty());
        assert_eq!(items1, [0u16;0]);
        assert_eq!(items2, [1, 2, 3, 4]);
        assert_eq!(items3, ["bar", "foo"]);
        assert_eq!(items4, [' ', 'a', 'b', 'f', 'o', 'r']);
    ```
    */
    fn unique_elements(&mut self) -> Vec<T>;
}

impl<T> UniqueElements<T> for Vec<T>
where
    T: Clone + Hash + Ord,
{
    fn unique_ordered(&mut self) {
        self.sort_unstable();
        self.dedup();
    }

    fn unique_elements(&mut self) -> Vec<T> {
        let vector: Vec<T> = self
            .iter()
            .collect::<BTreeSet<&T>>()
            .into_iter()
            .cloned()
            .collect();

        vector
    }
}

#[allow(dead_code)]
pub fn unique_by_hashset<T>(vs: &[T]) -> Vec<T>
where
    T: Clone + Hash + Eq,
{
    vs.iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .cloned()
        .collect()
}

#[allow(dead_code)]
pub fn unique_by_btreeset<T>(vs: &[T]) -> Vec<T>
where
    T: Clone + Hash + Ord,
{
    vs.iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .cloned()
        .collect()
}

#[cfg(test)]
mod sort_dedup {
    use super::*;

    #[test]
    /// `cargo test -- --show-output remove_duplicates`
    ///
    /// rustfmt src/my_traits.rs
    fn remove_duplicates() {
        let mut elements = vec![1, 2, 4, 2, 5, 3, 2];
        println!("elements: {elements:?}");

        elements.unique_ordered();
        println!("elements.unique_ordered(): {elements:?}");

        assert_eq!(elements, vec![1, 2, 3, 4, 5])
    }

    #[test]
    /// `cargo test -- --show-output unique_by_hash`
    fn unique_by_hash() {
        let elements = vec![1, 2, 3, 5, 2, 4, 4, 2, 3, 3];
        println!("elements: {elements:?}");

        let mut h = unique_by_hashset(&elements);
        println!("unique_by_hashset: {:?}", h);
        h.sort();

        let b = unique_by_btreeset(&elements);
        println!("unique_by_btreeset: {:?}", b);

        assert_eq!(h, [1, 2, 3, 4, 5]);
        assert_eq!(b, [1, 2, 3, 4, 5]);
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
