#![allow(unused)]
use rayon::prelude::*;
use std::collections::HashMap;

// --- Teste1 --- //

#[derive(Debug, Clone, PartialEq)]
struct Teste1 {
    cte: Option<String>,
    valor: f64,
}

impl Teste1 {
    fn new(cte: &str, valor: f64) -> Self {
        Teste1 {
            cte: Some(cte.to_string()),
            valor,
        }
    }
}

impl GetKey for Teste1 {
    fn get_chave(&self) -> Option<String> {
        self.cte.clone()
    }
}

// --- Teste2 --- //

#[derive(Debug, Clone, PartialEq)]
struct Teste2 {
    nfe: Option<String>,
    valor: f64,
}

impl Teste2 {
    fn new(nfe: &str, valor: f64) -> Self {
        Teste2 {
            nfe: Some(nfe.to_string()),
            valor,
        }
    }
}

impl GetKey for Teste2 {
    fn get_chave(&self) -> Option<String> {
        self.nfe.clone()
    }
}

// --- GetKey --- //

pub trait GetKey {
    fn get_chave(&self) -> Option<String>;
}

/**
The GroupByHashMapExt trait is an extension method for hashmaps that
groups elements by their keys and aggregates values into vectors,
allowing for efficient key-based data aggregation in Rust applications.
*/
pub trait GroupByHashMapExt<Structure> {
    fn group_by_hashmap_key_vector_v1(&self) -> HashMap<String, Vec<Structure>>;
    fn group_by_hashmap_key_vector_v2(&self) -> HashMap<String, Vec<&Structure>>;
}

impl<Structure> GroupByHashMapExt<Structure> for [Structure]
where
    Structure: GetKey + Clone + Send + Sync,
{
    fn group_by_hashmap_key_vector_v1(&self) -> HashMap<String, Vec<Structure>> {
        self.par_iter() // rayon: parallel iterator
            .flat_map(|structure| {
                structure.get_chave().as_ref().map(
                    |cte| (cte.clone(), structure), // (key, value)
                )
            })
            .fold(
                HashMap::new,
                |mut acc: HashMap<String, Vec<Structure>>, (key, value)| {
                    acc.entry(key).or_default().push(value.clone());
                    acc
                },
            )
            .reduce(HashMap::new, |mut acc, map| {
                map.into_iter().for_each(|(key, values)| {
                    acc.entry(key).or_default().extend(values);
                });
                acc
            })
    }

    fn group_by_hashmap_key_vector_v2(&self) -> HashMap<String, Vec<&Structure>> {
        self.par_iter() // rayon: parallel iterator
            .flat_map(|structure| {
                structure.get_chave().as_ref().map(
                    |cte| (cte.clone(), structure), // (key, value)
                )
            })
            .fold(
                HashMap::new,
                |mut acc: HashMap<String, Vec<&Structure>>, (key, value)| {
                    acc.entry(key).or_default().push(value);
                    acc
                },
            )
            .reduce(HashMap::new, |mut acc, map| {
                map.into_iter().for_each(|(key, values)| {
                    acc.entry(key).or_default().extend(values);
                });
                acc
            })
    }
}

#[cfg(test)]
mod group_by {
    use super::*;

    #[test]
    /// `cargo test -- --show-output group_by_hashmap`
    ///
    /// rustfmt src/group_by_hashmap.rs
    fn group_by_hashmap() {
        let vec_a: Vec<Teste1> = vec![
            Teste1::new("cte1", 10.0),
            Teste1::new("cte2", 20.0),
            Teste1 {
                cte: None,
                valor: 20.0,
            },
            Teste1::new("cte1", 40.0),
            Teste1::new("cte2", 70.0),
            Teste1::new("cte1", 60.0),
            Teste1::new("cte3", 20.0),
            Teste1::new("cte2", 70.0),
        ];

        let vec_b: Vec<Teste2> = vec![
            Teste2::new("nfe1", 10.0),
            Teste2::new("nfe2", 20.0),
            Teste2 {
                nfe: None,
                valor: 20.0,
            },
            Teste2::new("nfe2", 50.0),
        ];

        let result_a: HashMap<String, Vec<Teste1>> = vec_a.group_by_hashmap_key_vector_v1();
        let result_b: HashMap<String, Vec<Teste2>> = vec_b.group_by_hashmap_key_vector_v1();

        println!("{result_a:?}");
        println!("{result_b:?}");

        assert_eq!(
            HashMap::from([
                (
                    "cte1".to_string(),
                    vec![
                        Teste1::new("cte1", 10.0),
                        Teste1::new("cte1", 40.0),
                        Teste1::new("cte1", 60.0)
                    ]
                ),
                (
                    "cte2".to_string(),
                    vec![
                        Teste1::new("cte2", 20.0),
                        Teste1::new("cte2", 70.0),
                        Teste1::new("cte2", 70.0)
                    ]
                ),
                ("cte3".to_string(), vec![Teste1::new("cte3", 20.0)]),
            ]),
            result_a
        );
    }
}
