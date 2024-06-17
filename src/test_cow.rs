#![allow(dead_code)]
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    fmt::Debug
};

// https://blog.logrocket.com/using-cow-rust-efficient-memory-utilization/
// https://dev.to/kgrech/6-things-you-can-do-with-the-cow-in-rust-4l55

#[derive(Debug, Clone)]
struct Element {
    chave: Option<String>,
    n_item: Option<u32>,
    ncm: String,
    value: f64,
}

impl GetID for Element {
    fn get_id(&self) -> Option<(String, u32)> {
        if let (Some(nfe), Some(n_item)) = (&self.chave, self.n_item) {
            Some((nfe.clone(), n_item))
        } else {
            None
        }
    }
}

pub trait GetID {
    /// id identifica um único item de um document fiscal
    /// 
    /// id: (chave, n_item)
    /// 
    /// NFe pode conter vários itens
    /// 
    /// CTe contém apenas um item
    fn get_id(&self) -> Option<(String, u32)>;
}

pub trait UniqueKey<Structure> {
    fn get_unique(&self) -> Vec<Structure>
    where
        Structure: GetID + Clone;
}

impl<Structure> UniqueKey<Structure> for [Structure] {
    fn get_unique(&self) -> Vec<Structure>
    where
        Structure: GetID + Clone,
    {
        self.iter()
            .map(|elem| (elem.get_id(), elem))
            .collect::<BTreeMap<_, _>>() // único ordenado
            .into_values()
            .cloned()
            .collect::<Vec<Structure>>()
    }
}

pub fn get_unique_v1<Structure>(elements: &[Structure]) -> Vec<Structure>
where
    Structure: GetID + Clone,
{
    let mut unique = HashSet::new();
    elements
        .iter()
        .filter(|element| unique.insert(element.get_id()))
        .cloned()
        .collect::<Vec<Structure>>()
}

pub fn get_unique_v2<Structure>(elements: &[Structure]) -> Cow<[Structure]>
where
    Structure: GetID + Clone + Debug,
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

    #[test]
    /// `cargo test -- --show-output get_unique`
    ///
    /// rustfmt src/test_cow.rs
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
        let elements = vec![element1.clone(), element2, element1];
        for (index, element) in elements.iter().enumerate() { 
            println!("elements[{index}]: {element:?}"); 
        }
        println!();

        let unique_v1 = get_unique_v1(&elements);
        let unique_v2 = get_unique_v2(&elements);
        let unique_v3 = elements.get_unique();

        println!("unique_v1: {unique_v1:?}");
        println!("unique_v2: {unique_v2:?}");
        println!("unique_v3: {unique_v3:?}");
    }
}
