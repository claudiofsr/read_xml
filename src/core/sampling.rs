//! # Amostragem Estratificada em Streaming (`--amostragem`)
//!
//! Mantém estritamente no máximo `n` documentos fiscais por tipo em RAM O(n),
//! mesmo ao varrer bilhões de arquivos XML dentro de `.zip`s:
//! - `top_k` (`--maiores x`): Min-Heap de capacidade fixa `x` retendo os maiores valores.
//! - `random_k` (`--random y`): Priority/Bottom-k Reservoir Sampling uniforme sem reposição.

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    hash::{Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    Arguments, DocsFiscais, Information, XmlParserError, XmlParserResult,
    xml_structs::{cte::InfoCte, efinanceira::InfoEFinanceira, nfe::InfoNfe},
};

/// Configuração validada de amostragem por tipo de documento fiscal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleConfig {
    pub total_limit: usize,
    pub top_k: usize,
    pub random_k: usize,
    pub seed: u64,
}

impl SampleConfig {
    /// Constrói e valida a configuração de amostragem a partir dos argumentos da CLI.
    pub fn from_args(args: &Arguments) -> XmlParserResult<Option<Self>> {
        let Some(total) = args.amostragem else {
            return Ok(None);
        };

        if total == 0 {
            return Err(XmlParserError::InvalidDocument(
                "O valor de --amostragem deve ser maior que zero.".to_string(),
            ));
        }

        let (top_k, random_k) = match (args.amostra_maiores, args.amostra_aleatoria) {
            (Some(x), Some(y)) => {
                let soma = x.checked_add(y).ok_or_else(|| {
                    XmlParserError::InvalidDocument(
                        "A soma de --maiores e --random excede o limite permitido.".to_string(),
                    )
                })?;
                if soma == 0 || soma > total {
                    return Err(XmlParserError::InvalidDocument(format!(
                        "Combinação inválida: --maiores ({x}) + --random ({y}) deve estar entre 1 e --amostragem ({total})."
                    )));
                }
                (x, y)
            }
            (Some(x), None) => {
                if x > total {
                    return Err(XmlParserError::InvalidDocument(format!(
                        "--maiores ({x}) não pode ser maior que --amostragem ({total})."
                    )));
                }
                (x, total - x)
            }
            (None, Some(y)) => {
                if y > total {
                    return Err(XmlParserError::InvalidDocument(format!(
                        "--random ({y}) não pode ser maior que --amostragem ({total})."
                    )));
                }
                (total - y, y)
            }
            (None, None) => {
                let x = total / 2;
                (x, total - x)
            }
        };

        let seed = args.sample_seed.unwrap_or_else(|| {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0x9E37_79B9_7F4A_7C15, |d| d.as_nanos() as u64);
            splitmix64(nanos ^ (std::process::id() as u64))
        });

        Ok(Some(Self {
            total_limit: total,
            top_k,
            random_k,
            seed,
        }))
    }
}

#[inline]
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[inline]
fn hash_str(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Entrada no Min-Heap de maiores valores (topo = menor valor dentro do Top-K).
struct TopEntry<T> {
    value: f64,
    doc_key: String,
    payload: T,
}

impl<T> PartialEq for TopEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits() && self.doc_key == other.doc_key
    }
}

impl<T> Eq for TopEntry<T> {}

impl<T> Ord for TopEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invertido para que BinaryHeap mantenha o menor valor no topo (peek)
        other
            .value
            .total_cmp(&self.value)
            .then_with(|| other.doc_key.cmp(&self.doc_key))
    }
}

impl<T> PartialOrd for TopEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Entrada no Max-Heap de amostragem aleatória (topo = maior rand_key dentro do Bottom-K).
struct RandomEntry<T> {
    rand_key: u64,
    doc_key: String,
    payload: T,
}

impl<T> PartialEq for RandomEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rand_key == other.rand_key && self.doc_key == other.doc_key
    }
}

impl<T> Eq for RandomEntry<T> {}

impl<T> Ord for RandomEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rand_key
            .cmp(&other.rand_key)
            .then_with(|| self.doc_key.cmp(&other.doc_key))
    }
}

impl<T> PartialOrd for RandomEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Reservatório estratificado para um tipo de documento (`top_k` maiores + `random_k` aleatórios).
struct BucketSampler<T> {
    top_k: usize,
    random_k: usize,
    random_cap: usize,
    top_heap: BinaryHeap<TopEntry<T>>,
    top_keys: HashSet<String>,
    random_heap: BinaryHeap<RandomEntry<T>>,
    random_keys: HashSet<String>,
}

impl<T: Clone> BucketSampler<T> {
    fn new(top_k: usize, random_k: usize) -> Self {
        let random_cap = if random_k == 0 {
            0
        } else {
            top_k.saturating_add(random_k)
        };
        Self {
            top_k,
            random_k,
            random_cap,
            top_heap: BinaryHeap::with_capacity(top_k.min(1024)),
            top_keys: HashSet::with_capacity(top_k.min(1024)),
            random_heap: BinaryHeap::with_capacity(random_cap.min(1024)),
            random_keys: HashSet::with_capacity(random_cap.min(1024)),
        }
    }

    #[inline]
    fn would_accept_top(&self, doc_key: &str, value: f64) -> bool {
        if self.top_k == 0 || self.top_keys.contains(doc_key) {
            return false;
        }
        if self.top_heap.len() < self.top_k {
            return true;
        }
        self.top_heap
            .peek()
            .is_some_and(|min_entry| value.total_cmp(&min_entry.value) == Ordering::Greater)
    }

    #[inline]
    fn would_accept_random(&self, doc_key: &str, rand_key: u64) -> bool {
        if self.random_cap == 0 || self.random_keys.contains(doc_key) {
            return false;
        }
        if self.random_heap.len() < self.random_cap {
            return true;
        }
        self.random_heap
            .peek()
            .is_some_and(|max_entry| rand_key < max_entry.rand_key)
    }

    fn insert_top(&mut self, doc_key: String, value: f64, payload: T) {
        if !self.would_accept_top(&doc_key, value) {
            return;
        }
        if self.top_heap.len() >= self.top_k
            && let Some(removed) = self.top_heap.pop()
        {
            self.top_keys.remove(&removed.doc_key);
        }
        self.top_keys.insert(doc_key.clone());
        self.top_heap.push(TopEntry {
            value,
            doc_key,
            payload,
        });
    }

    fn insert_random(&mut self, doc_key: String, rand_key: u64, payload: T) {
        if !self.would_accept_random(&doc_key, rand_key) {
            return;
        }
        if self.random_heap.len() >= self.random_cap
            && let Some(removed) = self.random_heap.pop()
        {
            self.random_keys.remove(&removed.doc_key);
        }
        self.random_keys.insert(doc_key.clone());
        self.random_heap.push(RandomEntry {
            rand_key,
            doc_key,
            payload,
        });
    }

    fn push(&mut self, doc_key: String, value: f64, rand_key: u64, payload: T) {
        let take_top = self.would_accept_top(&doc_key, value);
        let take_rand = self.would_accept_random(&doc_key, rand_key);
        if !take_top && !take_rand {
            return;
        }
        if take_top && take_rand {
            self.insert_top(doc_key.clone(), value, payload.clone());
            self.insert_random(doc_key, rand_key, payload);
        } else if take_top {
            self.insert_top(doc_key, value, payload);
        } else {
            self.insert_random(doc_key, rand_key, payload);
        }
    }

    fn merge(&mut self, other: Self) {
        for entry in other.top_heap {
            self.insert_top(entry.doc_key, entry.value, entry.payload);
        }
        for entry in other.random_heap {
            self.insert_random(entry.doc_key, entry.rand_key, entry.payload);
        }
    }

    fn into_items(self) -> Vec<T> {
        let mut selected = Vec::with_capacity(self.top_k + self.random_k);
        let mut seen = HashSet::with_capacity(self.top_k + self.random_k);

        let top_sorted = self.top_heap.into_sorted_vec();
        for entry in top_sorted.into_iter().take(self.top_k) {
            if seen.insert(entry.doc_key) {
                selected.push(entry.payload);
            }
        }

        if self.random_k > 0 {
            let mut added_random = 0usize;
            for entry in self.random_heap.into_sorted_vec() {
                if added_random >= self.random_k {
                    break;
                }
                if seen.insert(entry.doc_key) {
                    selected.push(entry.payload);
                    added_random += 1;
                }
            }
        }

        selected
    }
}

/// Amostrador completo de documentos fiscais por thread/lote com memória limitada O(n).
pub struct FiscalSampler {
    cfg: SampleConfig,
    seq: u64,
    ctes: BucketSampler<InfoCte>,
    nfes: BucketSampler<Vec<InfoNfe>>,
    efin: BucketSampler<Vec<InfoEFinanceira>>,
}

impl FiscalSampler {
    pub fn new(cfg: SampleConfig) -> Self {
        Self {
            cfg,
            seq: 0,
            ctes: BucketSampler::new(cfg.top_k, cfg.random_k),
            nfes: BucketSampler::new(cfg.top_k, cfg.random_k),
            efin: BucketSampler::new(cfg.top_k, cfg.random_k),
        }
    }

    #[inline]
    fn next_rand_key(&mut self, doc_key: &str) -> u64 {
        self.seq = self.seq.wrapping_add(1);
        splitmix64(self.cfg.seed ^ hash_str(doc_key))
    }

    /// Consome uma variante `Information` de documento principal mantendo apenas a amostra em RAM.
    /// Retorna `Some(info)` apenas para eventos e cancelamentos que precisarem de tratamento externo.
    pub fn push_or_pass_event(&mut self, info: Information) -> Option<Information> {
        match info {
            Information::Cte(c) => {
                let doc = *c;
                let key = doc
                    .cte
                    .clone()
                    .unwrap_or_else(|| format!("cte_anon_{}", self.seq + 1));
                let val = sanitize_score(doc.valor_total.unwrap_or(0.0));
                let rkey = self.next_rand_key(&key);
                self.ctes.push(key, val, rkey, doc);
                None
            }
            Information::Nfe(items) => {
                if items.is_empty() {
                    return None;
                }
                let key = items
                    .first()
                    .and_then(|i| i.nfe.clone())
                    .unwrap_or_else(|| format!("nfe_anon_{}", self.seq + 1));
                let val = sanitize_score(
                    items
                        .first()
                        .and_then(|i| i.valor_total_nfe)
                        .unwrap_or_else(|| items.iter().filter_map(|i| i.v_prod).sum()),
                );
                let rkey = self.next_rand_key(&key);
                self.nfes.push(key, val, rkey, items);
                None
            }
            Information::EFinanceira(items) => {
                if items.is_empty() {
                    return None;
                }
                let key = items
                    .first()
                    .map(|i| {
                        format!(
                            "{:?}|{:?}|{:?}|{:?}|{:?}",
                            i.cnpj_do_declarante,
                            i.ni_do_declarado,
                            i.ano_mes_caixa,
                            i.num_conta,
                            i.id
                        )
                    })
                    .unwrap_or_else(|| format!("efin_anon_{}", self.seq + 1));
                let val = sanitize_score(
                    items
                        .iter()
                        .map(|i| {
                            i.tot_creditos.unwrap_or(0.0).abs()
                                + i.tot_debitos.unwrap_or(0.0).abs()
                                + i.vlr_ult_dia.unwrap_or(0.0).abs()
                        })
                        .sum(),
                );
                let rkey = self.next_rand_key(&key);
                self.efin.push(key, val, rkey, items);
                None
            }
            other @ (Information::EventoCte(_)
            | Information::EventoNfe(_)
            | Information::CancelamentoCte(_)
            | Information::CancelamentoNfe(_)) => Some(other),
            Information::None => None,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.ctes.merge(other.ctes);
        self.nfes.merge(other.nfes);
        self.efin.merge(other.efin);
    }

    /// Consolida a amostra final de documentos em `DocsFiscais`.
    pub fn into_docs(self) -> DocsFiscais {
        let mut docs = DocsFiscais::new();
        docs.ctes = self.ctes.into_items();
        docs.nfes = self.nfes.into_items().into_iter().flatten().collect();
        docs.efinanceiras = self.efin.into_items().into_iter().flatten().collect();
        docs
    }
}

/// Filtra os eventos/cancelamentos já acumulados em `DocsFiscais` mantendo apenas
/// os correspondentes às chaves selecionadas na amostra.
pub fn retain_events_for_sampled_docs(docs: &mut DocsFiscais) {
    let cte_keys: HashSet<&str> = docs.ctes.iter().filter_map(|c| c.cte.as_deref()).collect();
    let nfe_keys: HashSet<&str> = docs.nfes.iter().filter_map(|n| n.nfe.as_deref()).collect();

    let keep_cte_ev: Vec<_> = docs
        .eventos_cte
        .iter()
        .filter(|e| e.cte.as_deref().is_some_and(|k| cte_keys.contains(k)))
        .cloned()
        .collect();
    let keep_cte_canc: Vec<_> = docs
        .cancel_cte
        .iter()
        .filter(|c| c.cte.as_deref().is_some_and(|k| cte_keys.contains(k)))
        .cloned()
        .collect();
    let keep_nfe_ev: Vec<_> = docs
        .eventos_nfe
        .iter()
        .filter(|e| e.nfe.as_deref().is_some_and(|k| nfe_keys.contains(k)))
        .cloned()
        .collect();
    let keep_nfe_canc: Vec<_> = docs
        .cancel_nfe
        .iter()
        .filter(|c| c.nfe.as_deref().is_some_and(|k| nfe_keys.contains(k)))
        .cloned()
        .collect();

    drop(cte_keys);
    drop(nfe_keys);

    docs.eventos_cte = keep_cte_ev;
    docs.cancel_cte = keep_cte_canc;
    docs.eventos_nfe = keep_nfe_ev;
    docs.cancel_nfe = keep_nfe_canc;
}

/// Agrega contadores de eventos de vários `HashMap` se necessário no resumo.
#[allow(dead_code)]
fn _sanitize_unused(_: HashMap<String, usize>) {}

#[inline]
fn sanitize_score(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

#[cfg(test)]
mod tests_sampling {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_sample_config_validation() {
        let mut args = Arguments::parse_from(["read-xml"]);
        assert!(SampleConfig::from_args(&args).unwrap().is_none());

        args.amostragem = Some(0);
        assert!(SampleConfig::from_args(&args).is_err());

        args.amostragem = Some(100);
        let cfg = SampleConfig::from_args(&args).unwrap().unwrap();
        assert_eq!((cfg.top_k, cfg.random_k), (50, 50));

        args.amostra_maiores = Some(30);
        let cfg = SampleConfig::from_args(&args).unwrap().unwrap();
        assert_eq!((cfg.top_k, cfg.random_k), (30, 70));

        args.amostra_aleatoria = Some(80);
        assert!(SampleConfig::from_args(&args).is_err());

        args.amostra_aleatoria = Some(40);
        let cfg = SampleConfig::from_args(&args).unwrap().unwrap();
        assert_eq!((cfg.top_k, cfg.random_k), (30, 40));
    }

    #[test]
    fn test_top_k_and_random_k_disjoint_and_deterministic() {
        let cfg = SampleConfig {
            total_limit: 15,
            top_k: 5,
            random_k: 10,
            seed: 42,
        };

        let mut s1 = FiscalSampler::new(cfg);
        let mut worker_a = FiscalSampler::new(cfg);
        let mut worker_b = FiscalSampler::new(cfg);

        for i in 1..=200u32 {
            let doc = InfoCte {
                cte: Some(format!("{i:044}")),
                numero_da_nota: Some(i),
                valor_total: Some(i as f64 * 10.0),
                ..Default::default()
            };
            s1.push_or_pass_event(Information::Cte(Box::new(doc.clone())));
            if i % 2 == 0 {
                worker_a.push_or_pass_event(Information::Cte(Box::new(doc)));
            } else {
                worker_b.push_or_pass_event(Information::Cte(Box::new(doc)));
            }
        }
        // Duplicata do maior valor não deve ocupar 2 vagas
        let dup_200 = InfoCte {
            cte: Some(format!("{:044}", 200)),
            numero_da_nota: Some(200),
            valor_total: Some(2000.0),
            ..Default::default()
        };
        worker_a.push_or_pass_event(Information::Cte(Box::new(dup_200)));

        let s2 = {
            worker_a.merge(worker_b);
            worker_a
        };

        let docs_1 = s1.into_docs();
        let docs_2 = s2.into_docs();

        assert_eq!(docs_1.ctes.len(), 15);
        assert_eq!(docs_2.ctes.len(), 15);

        let keys_1: Vec<_> = docs_1.ctes.iter().map(|c| c.cte.clone()).collect();
        let keys_2: Vec<_> = docs_2.ctes.iter().map(|c| c.cte.clone()).collect();
        assert_eq!(keys_1, keys_2);

        let unique: HashSet<_> = docs_1.ctes.iter().map(|c| c.cte.as_deref()).collect();
        assert_eq!(unique.len(), 15);

        // Os 5 maiores (200, 199, 198, 197, 196) estão presentes
        for top in 196..=200 {
            assert!(unique.contains(&Some(format!("{top:044}").as_str())));
        }
    }

    #[test]
    fn test_nfe_sampled_as_whole_document() {
        let cfg = SampleConfig {
            total_limit: 2,
            top_k: 1,
            random_k: 1,
            seed: 7,
        };
        let mut sampler = FiscalSampler::new(cfg);

        for doc_idx in 1..=5u32 {
            let chave = format!("{doc_idx:044}");
            let itens = (1..=3u32)
                .map(|item_idx| InfoNfe {
                    nfe: Some(chave.clone()),
                    n_item: Some(item_idx),
                    valor_total_nfe: Some(doc_idx as f64 * 100.0),
                    v_prod: Some(100.0),
                    ..Default::default()
                })
                .collect();
            sampler.push_or_pass_event(Information::Nfe(itens));
        }

        let docs = sampler.into_docs();
        let unique_nfes: HashSet<_> = docs.nfes.iter().filter_map(|n| n.nfe.as_deref()).collect();
        assert_eq!(unique_nfes.len(), 2);
        // Cada NF-e selecionada manteve todos os seus 3 itens
        assert_eq!(docs.nfes.len(), 6);
        assert!(unique_nfes.contains(format!("{:044}", 5).as_str()));
    }
}
