#![allow(dead_code)]
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
struct Element {
    chave: Option<String>,
    n_item: Option<u32>,
    ncm: String,
    value: f64,
}

impl Element {
    fn get_id(&self) -> Option<(String, u32)> {
        if let (Some(nfe), Some(n_item)) = (&self.chave, self.n_item) {
            Some((nfe.clone(), n_item))
        } else {
            None
        }
    }
}

/// HashSet + Retain
fn get_unique_v1(elements: &mut Vec<Element>) -> Vec<Element> {
    let mut unique = HashSet::new();
    elements.retain(|element| unique.insert(element.get_id()));
    elements.clone()
}

/// HashSet + Filter
fn get_unique_v2(elements: &[Element]) -> Vec<Element> {
    let mut unique = HashSet::new();
    elements
        .iter()
        .filter(|element| unique.insert(element.get_id()))
        .cloned()
        .collect::<Vec<Element>>()
}

/// Map + HashMap
fn get_unique_v3(elements: &[Element]) -> Vec<Element> {
    elements
        .iter()
        .map(|elem| (elem.get_id(), elem))
        .collect::<HashMap<_, _>>() // único
        .into_values()
        .cloned()
        .collect::<Vec<Element>>()
}

/// Map + BTreeMap
fn get_unique_v4(elements: &[Element]) -> Vec<Element> {
    elements
        .iter()
        .map(|elem| (elem.get_id(), elem))
        .collect::<BTreeMap<_, _>>() // único ordenado
        .into_values()
        .cloned()
        .collect::<Vec<Element>>()
}

/// Map + HashMap + Rayon
fn get_unique_v5(elements: &[Element]) -> Vec<Element> {
    elements
        .par_iter()
        .map(|elem| (elem.get_id(), elem))
        .collect::<HashMap<_, _>>() // único
        .into_values()
        .cloned()
        .collect::<Vec<Element>>()
}

/// Map + BTreeMap + Rayon
fn get_unique_v6(elements: &[Element]) -> Vec<Element> {
    elements
        .par_iter()
        .map(|elem| (elem.get_id(), elem))
        .collect::<BTreeMap<_, _>>() // único ordenado
        .into_values()
        .cloned()
        .collect::<Vec<Element>>()
}

fn benchmark_get_unique(c: &mut Criterion) {
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

    let mut elements = Vec::new();

    for element in [element1, element2] {
        for index in 0..10_000 {
            let mut elem = element.clone();
            elem.n_item = Some(index);
            elements.push(elem);
        }
    }

    let mut group = c.benchmark_group("Get Unique");

    group.warm_up_time(std::time::Duration::from_secs(10));
    group.measurement_time(std::time::Duration::from_secs(60));
    group.sample_size(10_000);

    group.bench_function("HashSet + Retain", |b| {
        b.iter(|| {
            let _unique = black_box(get_unique_v1(&mut elements));
        })
    });

    group.bench_function("HashSet + Filter", |b| {
        b.iter(|| {
            let _unique = black_box(get_unique_v2(&elements));
        })
    });

    group.bench_function("Map + HashMap", |b| {
        b.iter(|| {
            let _unique = black_box(get_unique_v3(&elements));
        })
    });

    group.bench_function("Map + BTreeMap", |b| {
        b.iter(|| {
            let _unique = black_box(get_unique_v4(&elements));
        })
    });

    group.bench_function("Map + HashMap + Rayon", |b| {
        b.iter(|| {
            let _unique = black_box(get_unique_v5(&elements));
        })
    });

    group.bench_function("Map + BTreeMap + Rayon", |b| {
        b.iter(|| {
            let _unique = black_box(get_unique_v6(&elements));
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_get_unique);
criterion_main!(benches);
