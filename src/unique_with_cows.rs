#![allow(dead_code)]
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

/// Identificação única de uma Estrutura
///
/// Escolher campos que identificam unicamente uma Estrutura
pub trait GetID<T> {
    /// id identifica um único item de um document fiscal
    ///
    /// NFe pode conter vários itens, assim id: (chave, n_item)
    ///
    /// CTe contém apenas um item, assim id: chave
    fn get_id(&self) -> T;
}

pub trait UniqueIdentification<T, Structure> {
    /// Get Structure with unique ID
    fn get_unique_id(&self) -> Vec<Structure>
    where
        Structure: GetID<T> + Clone,
        T: Ord;

    fn get_unique_v1(&self) -> Vec<Structure>
    where
        Structure: GetID<T> + Clone,
        T: Eq + Hash;

    fn get_unique_v2(&self) -> Cow<[Structure]>
    where
        Structure: GetID<T> + Clone + Debug,
        T: Eq + Hash;
}

impl<T, Structure> UniqueIdentification<T, Structure> for [Structure] {
    fn get_unique_id(&self) -> Vec<Structure>
    where
        Structure: GetID<T> + Clone,
        T: Ord,
    {
        self.iter()
            .map(|elem| (elem.get_id(), elem))
            .collect::<BTreeMap<T, &Structure>>() // único ordenado
            .into_values()
            .cloned()
            .collect::<Vec<Structure>>()
    }

    fn get_unique_v1(&self) -> Vec<Structure>
    where
        Structure: GetID<T> + Clone,
        T: Eq + Hash,
    {
        let mut unique = HashSet::new();
        self.iter()
            .filter(|element| unique.insert(element.get_id()))
            .cloned()
            .collect::<Vec<Structure>>()
    }

    /**
    Cow means "clone on write"

    <https://dhghomon.github.io/easy_rust/Chapter_42.html>

    <https://blog.logrocket.com/using-cow-rust-efficient-memory-utilization>

    <https://dev.to/kgrech/6-things-you-can-do-with-the-cow-in-rust-4l55>
    */
    fn get_unique_v2(&self) -> Cow<[Structure]>
    where
        Structure: GetID<T> + Clone + Debug,
        T: Eq + Hash,
    {
        let mut seen = HashSet::new();
        for element in self {
            if !seen.insert(element.get_id()) {
                let mut unique = HashSet::new();
                return Cow::Owned(
                    self.iter()
                        .filter(|element| unique.insert(element.get_id()))
                        .cloned()
                        .collect::<Vec<Structure>>(),
                );
            }
        }
        Cow::Borrowed(self)
    }
}

pub fn get_unique_v1<T, Structure>(elements: &[Structure]) -> Vec<Structure>
where
    Structure: GetID<T> + Clone,
    T: Eq + Hash,
{
    let mut unique = HashSet::new();
    elements
        .iter()
        .filter(|element| unique.insert(element.get_id()))
        .cloned()
        .collect::<Vec<Structure>>()
}

pub fn get_unique_v2<T, Structure>(elements: &[Structure]) -> Cow<[Structure]>
where
    Structure: GetID<T> + Clone + Debug,
    T: Eq + Hash,
{
    let mut seen = HashSet::new();
    for element in elements {
        //print!("element: {element:?}");
        if !seen.insert(element.get_id()) {
            //println!(" <-- Duplicated element\n");
            let mut unique = HashSet::new();
            return Cow::Owned(
                elements
                    .iter()
                    .filter(|element| unique.insert(element.get_id()))
                    .cloned()
                    .collect::<Vec<Structure>>(),
            );
        }
        //println!();
    }
    //println!("Unique -> Borrowed\n");
    Cow::Borrowed(elements)
}

#[cfg(test)]
mod clone_on_write {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Element {
        chave: Option<String>,
        n_item: Option<u32>,
        ncm: String,
        value: f64,
    }

    impl GetID<Option<(String, u32)>> for &Element {
        fn get_id(&self) -> Option<(String, u32)> {
            if let (Some(nfe), Some(n_item)) = (&self.chave, self.n_item) {
                Some((nfe.clone(), n_item))
            } else {
                None
            }
        }
    }

    #[test]
    /// `cargo test -- --show-output get_unique`
    ///
    /// rustfmt src/unique_with_cows.rs
    fn get_unique() {
        let element1 = Element {
            chave: Some("12abc345".to_string()),
            n_item: Some(1),
            ncm: "123.456.789".to_string(),
            value: 2.45,
        };

        let element2 = Element {
            chave: Some("12abc345".to_string()),
            n_item: Some(2),
            ncm: "123.456.789".to_string(),
            value: 67.9,
        };

        //let elements = vec![element1, element2];
        let elements: Vec<&Element> = vec![&element1, &element2, &element1];
        for (index, element) in elements.iter().enumerate() {
            println!("elements[{index}]: {element:?}");
        }
        println!();

        let unique_v1: Vec<&Element> = elements.get_unique_id();
        let unique_v2: Vec<&Element> = elements.get_unique_v1();
        let unique_v3: Cow<[&Element]> = elements.get_unique_v2();

        println!("unique_v1: {unique_v1:?}");

        assert_eq!(unique_v1, [&element1, &element2]);
        assert_eq!(unique_v1, unique_v2);
        assert_eq!(*unique_v1, *unique_v3);
    }
}
