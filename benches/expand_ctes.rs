use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use read_xml::GraphExtension;
use std::collections::{HashMap, HashSet};
use std::hint::black_box;

// =============================================================================
// GERADORES DE CENÁRIOS DE DADOS FISCAIS REAIS (CHAVES DE 44 DÍGITOS)
// =============================================================================

fn gerar_chave_cte(id: usize) -> String {
    format!("3522091234569600010857001{:019}", id)
}

/// Cenário 1: Cadeias longas lineares (1 -> 2 -> 3 -> ... -> 10).
fn gerar_cenario_cadeias_longas(
    total_cadeias: usize,
    tamanho_cadeia: usize,
) -> HashMap<String, HashSet<String>> {
    let mut grafo = HashMap::new();
    let mut id_global = 0;

    for _ in 0..total_cadeias {
        let base_id = id_global;
        for passo in 0..(tamanho_cadeia - 1) {
            let u = gerar_chave_cte(base_id + passo);
            let v = gerar_chave_cte(base_id + passo + 1);
            grafo.entry(u).or_insert_with(HashSet::new).insert(v);
        }
        id_global += tamanho_cadeia;
    }

    grafo
}

/// Cenário 2: Lote fiscal real com 5.000 CT-es distribuídos em grupos
/// heterogêneos de 1, 2, 3, 5 e 10 documentos.
fn gerar_cenario_lote_misto(total_documentos: usize) -> HashMap<String, HashSet<String>> {
    let mut grafo = HashMap::new();
    let mut id = 0;

    while id < total_documentos {
        let modo = id % 5;
        match modo {
            // Isolado (1 elemento)
            0 => {
                let k = gerar_chave_cte(id);
                grafo.insert(k, HashSet::new());
                id += 1;
            }
            // Par (2 elementos)
            1 => {
                let a = gerar_chave_cte(id);
                let b = gerar_chave_cte(id + 1);
                grafo.entry(a).or_insert_with(HashSet::new).insert(b);
                id += 2;
            }
            // Trio (3 elementos)
            2 => {
                let a = gerar_chave_cte(id);
                let b = gerar_chave_cte(id + 1);
                let c = gerar_chave_cte(id + 2);
                grafo
                    .entry(a)
                    .or_insert_with(HashSet::new)
                    .insert(b.clone());
                grafo.entry(b).or_insert_with(HashSet::new).insert(c);
                id += 3;
            }
            // Quinteto (5 elementos com ciclo)
            3 => {
                let a = gerar_chave_cte(id);
                let b = gerar_chave_cte(id + 1);
                let c = gerar_chave_cte(id + 2);
                let d = gerar_chave_cte(id + 3);
                let e = gerar_chave_cte(id + 4);

                grafo
                    .entry(a.clone())
                    .or_insert_with(HashSet::new)
                    .insert(b);
                grafo
                    .entry(a.clone())
                    .or_insert_with(HashSet::new)
                    .insert(c.clone());
                grafo
                    .entry(c)
                    .or_insert_with(HashSet::new)
                    .insert(d.clone());
                grafo
                    .entry(d)
                    .or_insert_with(HashSet::new)
                    .insert(e.clone());
                grafo.entry(e).or_insert_with(HashSet::new).insert(a);
                id += 5;
            }
            // Deceto (10 elementos em rede)
            _ => {
                for offset in 0..9 {
                    let u = gerar_chave_cte(id + offset);
                    let v = gerar_chave_cte(id + offset + 1);
                    grafo.entry(u).or_insert_with(HashSet::new).insert(v);
                }
                id += 10;
            }
        }
    }

    grafo
}

// =============================================================================
// DEFINIÇÃO DOS BENCHMARKS
// =============================================================================

fn benchmark_expand_ctes(c: &mut Criterion) {
    // -------------------------------------------------------------------------
    // Grupo 1: Cadeias Longas (100 cadeias de 10 CT-es)
    // -------------------------------------------------------------------------
    let mut group_cadeias = c.benchmark_group("Cadeias Longas (100 cadeias de 10 CT-es)");
    group_cadeias.warm_up_time(std::time::Duration::from_secs(2));
    group_cadeias.measurement_time(std::time::Duration::from_secs(10));

    let grafo_cadeias = gerar_cenario_cadeias_longas(100, 10);

    group_cadeias.bench_function("Algoritmo Antigo (Ponto Fixo / Worklist)", |b| {
        b.iter_batched(
            || grafo_cadeias.clone(),
            |mut g| {
                g.expand_ctes_old(black_box(false));
                g
            },
            BatchSize::SmallInput,
        )
    });

    group_cadeias.bench_function("Algoritmo Novo (Union-Find / DSU de graph.rs)", |b| {
        b.iter_batched(
            || grafo_cadeias.clone(),
            |mut g| {
                g.expand_ctes(black_box(false));
                g
            },
            BatchSize::SmallInput,
        )
    });

    group_cadeias.finish();

    // -------------------------------------------------------------------------
    // Grupo 2: Lote Fiscal Misto Realista (5.000 CT-es)
    // -------------------------------------------------------------------------
    let mut group_lote = c.benchmark_group("Lote Fiscal Realista (5.000 CT-es)");
    group_lote.warm_up_time(std::time::Duration::from_secs(3));
    group_lote.measurement_time(std::time::Duration::from_secs(12));
    group_lote.sample_size(80); // Reduz de 100 para 80 medições

    let grafo_lote = gerar_cenario_lote_misto(5000);

    group_lote.bench_function("Algoritmo Antigo (Ponto Fixo / Worklist)", |b| {
        b.iter_batched(
            || grafo_lote.clone(),
            |mut g| {
                g.expand_ctes_old(black_box(false));
                g
            },
            BatchSize::LargeInput,
        )
    });

    group_lote.bench_function("Algoritmo Novo (Union-Find / DSU de graph.rs)", |b| {
        b.iter_batched(
            || grafo_lote.clone(),
            |mut g| {
                g.expand_ctes(black_box(false));
                g
            },
            BatchSize::LargeInput,
        )
    });

    group_lote.finish();
}

criterion_group!(benches, benchmark_expand_ctes);
criterion_main!(benches);
