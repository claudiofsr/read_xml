//! # Codec Binário Zstd + Postcard e Estruturas Auxiliares para Spill em Disco
//!
//! Fornece sharding seguro para FD (`NUM_SHARDS`), serialização binária sem conflitos com
//! atributos `serialize_with` de planilhas e iteradores de *K-Way Merge Sort*.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::{
    InfoReceita, XmlParserError, XmlParserResult, cmp_info_cte, cmp_info_efin, cmp_info_nfe,
    compare_receita_rows, cte::InfoCte, efinanceira::InfoEFinanceira, nfe::InfoNfe,
};

pub const NUM_SHARDS: usize = 64;

#[inline]
pub fn shard_index<K: Hash + ?Sized>(key: &K) -> usize {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    (hasher.finish() as usize) % NUM_SHARDS
}

/// Diretório temporário em disco local (evita `/tmp` tmpfs em RAM no Manjaro/Linux)
/// removido automaticamente quando o pipeline termina ou em caso de erro.
pub struct SpillDir {
    pub path: PathBuf,
}

impl SpillDir {
    pub fn create() -> XmlParserResult<Self> {
        let name = format!(".read_xml_spill_{}", std::process::id());
        let base = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
        let dir = base.join(&name);
        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        if fs::create_dir_all(&dir).is_ok() {
            return Ok(Self { path: dir });
        }
        let fallback = std::env::temp_dir().join(name);
        if fallback.exists() {
            let _ = fs::remove_dir_all(&fallback);
        }
        fs::create_dir_all(&fallback)?;
        Ok(Self { path: fallback })
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for SpillDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

type ZstdEncoder = zstd::stream::write::Encoder<'static, BufWriter<File>>;

/// Conjunto de `NUM_SHARDS` arquivos comprimidos com Zstd protegidos individualmente por Mutex.
pub struct ShardedZstdWriters {
    encoders: Vec<Mutex<Option<ZstdEncoder>>>,
}

impl ShardedZstdWriters {
    pub fn new(spill_dir: &SpillDir, prefix: &str) -> XmlParserResult<Self> {
        let mut encoders = Vec::with_capacity(NUM_SHARDS);
        for p in 0..NUM_SHARDS {
            let path = spill_dir.file(&format!("{prefix}_{p:03}.zst"));
            let file = File::create(path)?;
            let buf = BufWriter::with_capacity(32 * 1024, file);
            let enc = zstd::stream::write::Encoder::new(buf, 1)?;
            encoders.push(Mutex::new(Some(enc)));
        }
        Ok(Self { encoders })
    }

    pub fn write_chunk(&self, shard: usize, bytes: &[u8]) -> XmlParserResult<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let mut guard = self.encoders[shard]
            .lock()
            .map_err(|_| XmlParserError::Export("Mutex poisoned in ShardedZstdWriters".into()))?;
        if let Some(enc) = guard.as_mut() {
            enc.write_all(bytes)?;
        }
        Ok(())
    }

    pub fn finish_all(self) -> XmlParserResult<()> {
        for m in self.encoders {
            let mut guard = m
                .into_inner()
                .map_err(|_| XmlParserError::Export("Mutex poisoned on finish".into()))?;
            if let Some(enc) = guard.take() {
                enc.finish()?;
            }
        }
        Ok(())
    }
}

/// Serializa um registro `T` com framing `[len: u32 LE][payload postcard]` para buffer em RAM.
#[inline]
pub fn encode_framed<T: Serialize>(item: &T, out_buf: &mut Vec<u8>) -> XmlParserResult<()> {
    let bytes = postcard::to_stdvec(item)
        .map_err(|e| XmlParserError::Export(format!("Erro ao serializar postcard: {e}")))?;
    let len: u32 = bytes.len().try_into()?;
    out_buf.extend_from_slice(&len.to_le_bytes());
    out_buf.extend_from_slice(&bytes);
    Ok(())
}

/// Escreve um `Vec<T>` completo em um único arquivo `.zst` delimitado.
pub fn write_vec_zst<T: Serialize>(path: &Path, items: &[T]) -> XmlParserResult<()> {
    if items.is_empty() {
        return Ok(());
    }
    let file = File::create(path)?;
    let mut enc = zstd::stream::write::Encoder::new(BufWriter::with_capacity(64 * 1024, file), 1)?;
    let mut scratch = Vec::with_capacity(1024);
    for item in items {
        scratch.clear();
        encode_framed(item, &mut scratch)?;
        enc.write_all(&scratch)?;
    }
    enc.finish()?;
    Ok(())
}

/// Lê integralmente um arquivo `.zst` delimitado para um `Vec<T>`.
pub fn read_vec_zst<T: DeserializeOwned>(path: &Path) -> XmlParserResult<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut reader = ShardStreamReader::<T>::open(path)?;
    let mut out = Vec::new();
    while let Some(item) = reader.next_item()? {
        out.push(item);
    }
    Ok(out)
}

/// Leitor em streaming para um único shard `.zst`.
pub struct ShardStreamReader<T> {
    decoder: zstd::stream::read::Decoder<'static, BufReader<File>>,
    len_buf: [u8; 4],
    data_buf: Vec<u8>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned> ShardStreamReader<T> {
    pub fn open(path: &Path) -> XmlParserResult<Self> {
        let file = File::open(path)?;
        let buffered = BufReader::with_capacity(32 * 1024, file);
        let decoder = zstd::stream::read::Decoder::with_buffer(buffered)?;
        Ok(Self {
            decoder,
            len_buf: [0u8; 4],
            data_buf: Vec::with_capacity(1024),
            _marker: std::marker::PhantomData,
        })
    }

    pub fn next_item(&mut self) -> XmlParserResult<Option<T>> {
        match self.decoder.read_exact(&mut self.len_buf) {
            Ok(()) => {
                let len = u32::from_le_bytes(self.len_buf) as usize;
                self.data_buf.resize(len, 0);
                self.decoder.read_exact(&mut self.data_buf)?;
                let item = postcard::from_bytes(&self.data_buf).map_err(|e| {
                    XmlParserError::Export(format!("Erro ao desserializar postcard: {e}"))
                })?;
                Ok(Some(item))
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

/// Trait de ordenação reutilizada no K-Way Merge Sort em disco.
pub trait MergeOrder {
    fn cmp_order(&self, other: &Self) -> Ordering;
}

impl MergeOrder for InfoReceita {
    fn cmp_order(&self, other: &Self) -> Ordering {
        compare_receita_rows(self, other)
    }
}

impl MergeOrder for InfoCte {
    fn cmp_order(&self, other: &Self) -> Ordering {
        cmp_info_cte(self, other)
    }
}

impl MergeOrder for InfoNfe {
    fn cmp_order(&self, other: &Self) -> Ordering {
        cmp_info_nfe(self, other)
    }
}

impl MergeOrder for InfoEFinanceira {
    fn cmp_order(&self, other: &Self) -> Ordering {
        cmp_info_efin(self, other)
    }
}

impl MergeOrder for String {
    fn cmp_order(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

struct HeapEntry<T> {
    item: T,
    reader_idx: usize,
}

impl<T: MergeOrder> PartialEq for HeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item.cmp_order(&other.item) == Ordering::Equal && self.reader_idx == other.reader_idx
    }
}

impl<T: MergeOrder> Eq for HeapEntry<T> {}

impl<T: MergeOrder> Ord for HeapEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invertido para o BinaryHeap atuar como min-heap
        other
            .item
            .cmp_order(&self.item)
            .then_with(|| other.reader_idx.cmp(&self.reader_idx))
    }
}

impl<T: MergeOrder> PartialOrd for HeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Iterador de K-Way Merge Sort sobre múltiplos shards previamente ordenados.
/// Mantém apenas 1 registro por shard em memória RAM.
pub struct KWayMergeIter<T> {
    readers: Vec<ShardStreamReader<T>>,
    heap: BinaryHeap<HeapEntry<T>>,
}

impl<T: DeserializeOwned + MergeOrder> KWayMergeIter<T> {
    pub fn open(paths: &[PathBuf]) -> XmlParserResult<Self> {
        let mut readers = Vec::new();
        let mut heap = BinaryHeap::new();

        for path in paths {
            if !path.exists() {
                continue;
            }
            let mut reader = ShardStreamReader::<T>::open(path)?;
            if let Some(first) = reader.next_item()? {
                let idx = readers.len();
                readers.push(reader);
                heap.push(HeapEntry {
                    item: first,
                    reader_idx: idx,
                });
            }
        }

        Ok(Self { readers, heap })
    }
}

impl<T: DeserializeOwned + MergeOrder> Iterator for KWayMergeIter<T> {
    type Item = XmlParserResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let top = self.heap.pop()?;
        let idx = top.reader_idx;
        match self.readers[idx].next_item() {
            Ok(Some(next_item)) => {
                self.heap.push(HeapEntry {
                    item: next_item,
                    reader_idx: idx,
                });
            }
            Ok(None) => {}
            Err(err) => return Some(Err(err)),
        }
        Some(Ok(top.item))
    }
}

#[inline]
fn date_to_ce(d: Option<NaiveDate>) -> Option<i32> {
    d.map(|dt| dt.num_days_from_ce())
}

#[inline]
fn ce_to_date(days: Option<i32>) -> Option<NaiveDate> {
    days.and_then(NaiveDate::from_num_days_from_ce_opt)
}

// =============================================================================
// SHADOW CODECS (evitam conflito de `serialize_with` para Excel/CSV no Postcard)
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct SpillCte {
    pub versao: Option<String>,
    pub emitente_cnpj: Option<String>,
    pub emitente_cpf: Option<String>,
    pub emitente_crt: Option<u8>,
    pub emitente_nome: Option<String>,
    pub emitente_fantasia: Option<String>,
    pub emitente_ender_municipio: Option<String>,
    pub emitente_ender_estado: Option<String>,
    pub remetente_cnpj: Option<String>,
    pub remetente_cpf: Option<String>,
    pub remetente_crt: Option<u8>,
    pub remetente_nome: Option<String>,
    pub remetente_fantasia: Option<String>,
    pub remetente_ender_municipio: Option<String>,
    pub remetente_ender_estado: Option<String>,
    pub remetente_nfes: Vec<String>,
    pub destinatario_cnpj: Option<String>,
    pub destinatario_cpf: Option<String>,
    pub destinatario_crt: Option<u8>,
    pub destinatario_nome: Option<String>,
    pub destinatario_fantasia: Option<String>,
    pub destinatario_ender_municipio: Option<String>,
    pub destinatario_ender_estado: Option<String>,
    pub expedidor_cnpj: Option<String>,
    pub expedidor_cpf: Option<String>,
    pub expedidor_crt: Option<u8>,
    pub expedidor_nome: Option<String>,
    pub expedidor_fantasia: Option<String>,
    pub expedidor_ender_municipio: Option<String>,
    pub expedidor_ender_estado: Option<String>,
    pub recebedor_cnpj: Option<String>,
    pub recebedor_cpf: Option<String>,
    pub recebedor_crt: Option<u8>,
    pub recebedor_nome: Option<String>,
    pub recebedor_fantasia: Option<String>,
    pub recebedor_ender_municipio: Option<String>,
    pub recebedor_ender_estado: Option<String>,
    pub tomador_cnpj: Option<String>,
    pub tomador_cpf: Option<String>,
    pub tomador_nome: Option<String>,
    pub tomador_fantasia: Option<String>,
    pub tomador_ender_municipio: Option<String>,
    pub tomador_ender_estado: Option<String>,
    pub tomador_codigo: Option<u8>,
    pub cte: Option<String>,
    pub doc_tipo: String,
    pub cancelado: Option<String>,
    pub numero_da_nota: Option<u32>,
    pub nat_operacao: Option<String>,
    pub uf_inicio: Option<String>,
    pub mun_inicio: Option<String>,
    pub uf_termino: Option<String>,
    pub mun_termino: Option<String>,
    pub local_entrega_nome: Option<String>,
    pub obs_gerais: Option<String>,
    pub cfop: Option<u16>,
    pub data_emissao: Option<i32>,
    pub cte_complementar: Vec<String>,
    pub cte_anteriores: Vec<String>,
    pub nfes_vinculados: Vec<String>,
    pub nfes: Vec<String>,
    pub ncm_descricao: Vec<String>,
    pub valor_total_nfes: Option<f64>,
    pub valor_total: Option<f64>,
    pub aliq_pis: Option<f64>,
    pub aliq_cofins: Option<f64>,
    pub v_pis: Option<f64>,
    pub v_cofins: Option<f64>,
    pub v_ipi: Option<f64>,
    pub v_bc_iss: Option<f64>,
    pub v_iss: Option<f64>,
    pub v_bc_icms: Option<f64>,
    pub aliq_icms: Option<f64>,
    pub v_icms: Option<f64>,
    pub v_icms_st: Option<f64>,
}

impl From<InfoCte> for SpillCte {
    fn from(c: InfoCte) -> Self {
        Self {
            versao: c.versao,
            emitente_cnpj: c.emitente_cnpj,
            emitente_cpf: c.emitente_cpf,
            emitente_crt: c.emitente_crt,
            emitente_nome: c.emitente_nome,
            emitente_fantasia: c.emitente_fantasia,
            emitente_ender_municipio: c.emitente_ender_municipio,
            emitente_ender_estado: c.emitente_ender_estado,
            remetente_cnpj: c.remetente_cnpj,
            remetente_cpf: c.remetente_cpf,
            remetente_crt: c.remetente_crt,
            remetente_nome: c.remetente_nome,
            remetente_fantasia: c.remetente_fantasia,
            remetente_ender_municipio: c.remetente_ender_municipio,
            remetente_ender_estado: c.remetente_ender_estado,
            remetente_nfes: c.remetente_nfes,
            destinatario_cnpj: c.destinatario_cnpj,
            destinatario_cpf: c.destinatario_cpf,
            destinatario_crt: c.destinatario_crt,
            destinatario_nome: c.destinatario_nome,
            destinatario_fantasia: c.destinatario_fantasia,
            destinatario_ender_municipio: c.destinatario_ender_municipio,
            destinatario_ender_estado: c.destinatario_ender_estado,
            expedidor_cnpj: c.expedidor_cnpj,
            expedidor_cpf: c.expedidor_cpf,
            expedidor_crt: c.expedidor_crt,
            expedidor_nome: c.expedidor_nome,
            expedidor_fantasia: c.expedidor_fantasia,
            expedidor_ender_municipio: c.expedidor_ender_municipio,
            expedidor_ender_estado: c.expedidor_ender_estado,
            recebedor_cnpj: c.recebedor_cnpj,
            recebedor_cpf: c.recebedor_cpf,
            recebedor_crt: c.recebedor_crt,
            recebedor_nome: c.recebedor_nome,
            recebedor_fantasia: c.recebedor_fantasia,
            recebedor_ender_municipio: c.recebedor_ender_municipio,
            recebedor_ender_estado: c.recebedor_ender_estado,
            tomador_cnpj: c.tomador_cnpj,
            tomador_cpf: c.tomador_cpf,
            tomador_nome: c.tomador_nome,
            tomador_fantasia: c.tomador_fantasia,
            tomador_ender_municipio: c.tomador_ender_municipio,
            tomador_ender_estado: c.tomador_ender_estado,
            tomador_codigo: c.tomador_codigo,
            cte: c.cte,
            doc_tipo: c.doc_tipo,
            cancelado: c.cancelado,
            numero_da_nota: c.numero_da_nota,
            nat_operacao: c.nat_operacao,
            uf_inicio: c.uf_inicio,
            mun_inicio: c.mun_inicio,
            uf_termino: c.uf_termino,
            mun_termino: c.mun_termino,
            local_entrega_nome: c.local_entrega_nome,
            obs_gerais: c.obs_gerais,
            cfop: c.cfop,
            data_emissao: date_to_ce(c.data_emissao),
            cte_complementar: c.cte_complementar,
            cte_anteriores: c.cte_anteriores,
            nfes_vinculados: c.nfes_vinculados,
            nfes: c.nfes,
            ncm_descricao: c.ncm_descricao,
            valor_total_nfes: c.valor_total_nfes,
            valor_total: c.valor_total,
            aliq_pis: c.aliq_pis,
            aliq_cofins: c.aliq_cofins,
            v_pis: c.v_pis,
            v_cofins: c.v_cofins,
            v_ipi: c.v_ipi,
            v_bc_iss: c.v_bc_iss,
            v_iss: c.v_iss,
            v_bc_icms: c.v_bc_icms,
            aliq_icms: c.aliq_icms,
            v_icms: c.v_icms,
            v_icms_st: c.v_icms_st,
        }
    }
}

impl From<SpillCte> for InfoCte {
    fn from(c: SpillCte) -> Self {
        Self {
            versao: c.versao,
            emitente_cnpj: c.emitente_cnpj,
            emitente_cpf: c.emitente_cpf,
            emitente_crt: c.emitente_crt,
            emitente_nome: c.emitente_nome,
            emitente_fantasia: c.emitente_fantasia,
            emitente_ender_municipio: c.emitente_ender_municipio,
            emitente_ender_estado: c.emitente_ender_estado,
            remetente_cnpj: c.remetente_cnpj,
            remetente_cpf: c.remetente_cpf,
            remetente_crt: c.remetente_crt,
            remetente_nome: c.remetente_nome,
            remetente_fantasia: c.remetente_fantasia,
            remetente_ender_municipio: c.remetente_ender_municipio,
            remetente_ender_estado: c.remetente_ender_estado,
            remetente_nfes: c.remetente_nfes,
            destinatario_cnpj: c.destinatario_cnpj,
            destinatario_cpf: c.destinatario_cpf,
            destinatario_crt: c.destinatario_crt,
            destinatario_nome: c.destinatario_nome,
            destinatario_fantasia: c.destinatario_fantasia,
            destinatario_ender_municipio: c.destinatario_ender_municipio,
            destinatario_ender_estado: c.destinatario_ender_estado,
            expedidor_cnpj: c.expedidor_cnpj,
            expedidor_cpf: c.expedidor_cpf,
            expedidor_crt: c.expedidor_crt,
            expedidor_nome: c.expedidor_nome,
            expedidor_fantasia: c.expedidor_fantasia,
            expedidor_ender_municipio: c.expedidor_ender_municipio,
            expedidor_ender_estado: c.expedidor_ender_estado,
            recebedor_cnpj: c.recebedor_cnpj,
            recebedor_cpf: c.recebedor_cpf,
            recebedor_crt: c.recebedor_crt,
            recebedor_nome: c.recebedor_nome,
            recebedor_fantasia: c.recebedor_fantasia,
            recebedor_ender_municipio: c.recebedor_ender_municipio,
            recebedor_ender_estado: c.recebedor_ender_estado,
            tomador_cnpj: c.tomador_cnpj,
            tomador_cpf: c.tomador_cpf,
            tomador_nome: c.tomador_nome,
            tomador_fantasia: c.tomador_fantasia,
            tomador_ender_municipio: c.tomador_ender_municipio,
            tomador_ender_estado: c.tomador_ender_estado,
            tomador_codigo: c.tomador_codigo,
            cte: c.cte,
            doc_tipo: c.doc_tipo,
            cancelado: c.cancelado,
            numero_da_nota: c.numero_da_nota,
            nat_operacao: c.nat_operacao,
            uf_inicio: c.uf_inicio,
            mun_inicio: c.mun_inicio,
            uf_termino: c.uf_termino,
            mun_termino: c.mun_termino,
            local_entrega_nome: c.local_entrega_nome,
            obs_gerais: c.obs_gerais,
            cfop: c.cfop,
            data_emissao: ce_to_date(c.data_emissao),
            cte_complementar: c.cte_complementar,
            cte_anteriores: c.cte_anteriores,
            nfes_vinculados: c.nfes_vinculados,
            nfes: c.nfes,
            ncm_descricao: c.ncm_descricao,
            valor_total_nfes: c.valor_total_nfes,
            valor_total: c.valor_total,
            aliq_pis: c.aliq_pis,
            aliq_cofins: c.aliq_cofins,
            v_pis: c.v_pis,
            v_cofins: c.v_cofins,
            v_ipi: c.v_ipi,
            v_bc_iss: c.v_bc_iss,
            v_iss: c.v_iss,
            v_bc_icms: c.v_bc_icms,
            aliq_icms: c.aliq_icms,
            v_icms: c.v_icms,
            v_icms_st: c.v_icms_st,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpillNfe {
    pub versao: Option<String>,
    pub emitente_cnpj: Option<String>,
    pub emitente_cpf: Option<String>,
    pub emitente_crt: Option<u8>,
    pub emitente_nome: Option<String>,
    pub emitente_fantasia: Option<String>,
    pub emitente_ender_municipio: Option<String>,
    pub emitente_ender_estado: Option<String>,
    pub destinatario_cnpj: Option<String>,
    pub destinatario_cpf: Option<String>,
    pub destinatario_nome: Option<String>,
    pub destinatario_ender_municipio: Option<String>,
    pub destinatario_ender_estado: Option<String>,
    pub nfe: Option<String>,
    pub doc_tipo: String,
    pub cancelado: Option<String>,
    pub modelo: Option<String>,
    pub tp_nf: Option<String>,
    pub nat_operacao: Option<String>,
    pub numero_da_nota: Option<u32>,
    pub data_emissao: Option<i32>,
    pub data_saida: Option<i32>,
    pub n_item: Option<u32>,
    pub numero_de_itens: usize,
    pub n_di: Option<u64>,
    pub descricao: Option<String>,
    pub cfop: Option<u16>,
    pub ncm: Option<String>,
    pub info_adic_contribuinte: Option<String>,
    pub info_adic_fisco: Option<String>,
    pub cst_pis: Option<u8>,
    pub cst_cofins: Option<u8>,
    pub ctes: Vec<String>,
    pub nfes_relacionadas: Vec<String>,
    pub tomadores: Vec<String>,
    pub valor_total_ctes: Option<f64>,
    pub valor_total_nfe: Option<f64>,
    pub valor_total_itens: Option<f64>,
    pub v_prod: Option<f64>,
    pub v_desc: Option<f64>,
    pub v_frete: Option<f64>,
    pub v_seg: Option<f64>,
    pub aliq_pis: Option<f64>,
    pub aliq_cofins: Option<f64>,
    pub v_pis: Option<f64>,
    pub v_cofins: Option<f64>,
    pub v_ipi: Option<f64>,
    pub v_bc_iss: Option<f64>,
    pub v_iss: Option<f64>,
    pub v_bc_icms: Option<f64>,
    pub aliq_icms: Option<f64>,
    pub v_icms: Option<f64>,
    pub v_icms_st: Option<f64>,
    pub v_outro: Option<f64>,
    pub v_item_total: Option<f64>,
}

impl From<InfoNfe> for SpillNfe {
    fn from(n: InfoNfe) -> Self {
        Self {
            versao: n.versao,
            emitente_cnpj: n.emitente_cnpj,
            emitente_cpf: n.emitente_cpf,
            emitente_crt: n.emitente_crt,
            emitente_nome: n.emitente_nome,
            emitente_fantasia: n.emitente_fantasia,
            emitente_ender_municipio: n.emitente_ender_municipio,
            emitente_ender_estado: n.emitente_ender_estado,
            destinatario_cnpj: n.destinatario_cnpj,
            destinatario_cpf: n.destinatario_cpf,
            destinatario_nome: n.destinatario_nome,
            destinatario_ender_municipio: n.destinatario_ender_municipio,
            destinatario_ender_estado: n.destinatario_ender_estado,
            nfe: n.nfe,
            doc_tipo: n.doc_tipo,
            cancelado: n.cancelado,
            modelo: n.modelo,
            tp_nf: n.tp_nf,
            nat_operacao: n.nat_operacao,
            numero_da_nota: n.numero_da_nota,
            data_emissao: date_to_ce(n.data_emissao),
            data_saida: date_to_ce(n.data_saida),
            n_item: n.n_item,
            numero_de_itens: n.numero_de_itens,
            n_di: n.n_di,
            descricao: n.descricao,
            cfop: n.cfop,
            ncm: n.ncm,
            info_adic_contribuinte: n.info_adic_contribuinte,
            info_adic_fisco: n.info_adic_fisco,
            cst_pis: n.cst_pis,
            cst_cofins: n.cst_cofins,
            ctes: n.ctes,
            nfes_relacionadas: n.nfes_relacionadas,
            tomadores: n.tomadores,
            valor_total_ctes: n.valor_total_ctes,
            valor_total_nfe: n.valor_total_nfe,
            valor_total_itens: n.valor_total_itens,
            v_prod: n.v_prod,
            v_desc: n.v_desc,
            v_frete: n.v_frete,
            v_seg: n.v_seg,
            aliq_pis: n.aliq_pis,
            aliq_cofins: n.aliq_cofins,
            v_pis: n.v_pis,
            v_cofins: n.v_cofins,
            v_ipi: n.v_ipi,
            v_bc_iss: n.v_bc_iss,
            v_iss: n.v_iss,
            v_bc_icms: n.v_bc_icms,
            aliq_icms: n.aliq_icms,
            v_icms: n.v_icms,
            v_icms_st: n.v_icms_st,
            v_outro: n.v_outro,
            v_item_total: n.v_item_total,
        }
    }
}

impl From<SpillNfe> for InfoNfe {
    fn from(n: SpillNfe) -> Self {
        Self {
            versao: n.versao,
            emitente_cnpj: n.emitente_cnpj,
            emitente_cpf: n.emitente_cpf,
            emitente_crt: n.emitente_crt,
            emitente_nome: n.emitente_nome,
            emitente_fantasia: n.emitente_fantasia,
            emitente_ender_municipio: n.emitente_ender_municipio,
            emitente_ender_estado: n.emitente_ender_estado,
            destinatario_cnpj: n.destinatario_cnpj,
            destinatario_cpf: n.destinatario_cpf,
            destinatario_nome: n.destinatario_nome,
            destinatario_ender_municipio: n.destinatario_ender_municipio,
            destinatario_ender_estado: n.destinatario_ender_estado,
            nfe: n.nfe,
            doc_tipo: n.doc_tipo,
            cancelado: n.cancelado,
            modelo: n.modelo,
            tp_nf: n.tp_nf,
            nat_operacao: n.nat_operacao,
            numero_da_nota: n.numero_da_nota,
            data_emissao: ce_to_date(n.data_emissao),
            data_saida: ce_to_date(n.data_saida),
            n_item: n.n_item,
            numero_de_itens: n.numero_de_itens,
            n_di: n.n_di,
            descricao: n.descricao,
            cfop: n.cfop,
            ncm: n.ncm,
            info_adic_contribuinte: n.info_adic_contribuinte,
            info_adic_fisco: n.info_adic_fisco,
            cst_pis: n.cst_pis,
            cst_cofins: n.cst_cofins,
            ctes: n.ctes,
            nfes_relacionadas: n.nfes_relacionadas,
            tomadores: n.tomadores,
            valor_total_ctes: n.valor_total_ctes,
            valor_total_nfe: n.valor_total_nfe,
            valor_total_itens: n.valor_total_itens,
            v_prod: n.v_prod,
            v_desc: n.v_desc,
            v_frete: n.v_frete,
            v_seg: n.v_seg,
            aliq_pis: n.aliq_pis,
            aliq_cofins: n.aliq_cofins,
            v_pis: n.v_pis,
            v_cofins: n.v_cofins,
            v_ipi: n.v_ipi,
            v_bc_iss: n.v_bc_iss,
            v_iss: n.v_iss,
            v_bc_icms: n.v_bc_icms,
            aliq_icms: n.aliq_icms,
            v_icms: n.v_icms,
            v_icms_st: n.v_icms_st,
            v_outro: n.v_outro,
            v_item_total: n.v_item_total,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpillEFinanceira {
    pub id: Option<String>,
    pub ind_retificacao: Option<String>,
    pub nr_recibo: Option<String>,
    pub cnpj_do_declarante: Option<String>,
    pub ni_do_declarado: Option<String>,
    pub nome_declarado: Option<String>,
    pub ano_mes_caixa: Option<i32>,
    pub tp_conta: Option<String>,
    pub sub_tp_conta: Option<String>,
    pub num_conta: Option<String>,
    pub dt_abertura_conta: Option<i32>,
    pub forma_abertura_conta: Option<String>,
    pub dt_encerramento_conta: Option<i32>,
    pub moeda: Option<String>,
    pub pais_reportado: Option<String>,
    pub tot_creditos: Option<f64>,
    pub tot_debitos: Option<f64>,
    pub tot_creditos_mesma_titularidade: Option<f64>,
    pub tot_debitos_mesma_titularidade: Option<f64>,
    pub vlr_ult_dia: Option<f64>,
}

impl From<InfoEFinanceira> for SpillEFinanceira {
    fn from(e: InfoEFinanceira) -> Self {
        Self {
            id: e.id,
            ind_retificacao: e.ind_retificacao,
            nr_recibo: e.nr_recibo,
            cnpj_do_declarante: e.cnpj_do_declarante,
            ni_do_declarado: e.ni_do_declarado,
            nome_declarado: e.nome_declarado,
            ano_mes_caixa: date_to_ce(e.ano_mes_caixa),
            tp_conta: e.tp_conta,
            sub_tp_conta: e.sub_tp_conta,
            num_conta: e.num_conta,
            dt_abertura_conta: date_to_ce(e.dt_abertura_conta),
            forma_abertura_conta: e.forma_abertura_conta,
            dt_encerramento_conta: date_to_ce(e.dt_encerramento_conta),
            moeda: e.moeda,
            pais_reportado: e.pais_reportado,
            tot_creditos: e.tot_creditos,
            tot_debitos: e.tot_debitos,
            tot_creditos_mesma_titularidade: e.tot_creditos_mesma_titularidade,
            tot_debitos_mesma_titularidade: e.tot_debitos_mesma_titularidade,
            vlr_ult_dia: e.vlr_ult_dia,
        }
    }
}

impl From<SpillEFinanceira> for InfoEFinanceira {
    fn from(e: SpillEFinanceira) -> Self {
        Self {
            id: e.id,
            ind_retificacao: e.ind_retificacao,
            nr_recibo: e.nr_recibo,
            cnpj_do_declarante: e.cnpj_do_declarante,
            ni_do_declarado: e.ni_do_declarado,
            nome_declarado: e.nome_declarado,
            ano_mes_caixa: ce_to_date(e.ano_mes_caixa),
            tp_conta: e.tp_conta,
            sub_tp_conta: e.sub_tp_conta,
            num_conta: e.num_conta,
            dt_abertura_conta: ce_to_date(e.dt_abertura_conta),
            forma_abertura_conta: e.forma_abertura_conta,
            dt_encerramento_conta: ce_to_date(e.dt_encerramento_conta),
            moeda: e.moeda,
            pais_reportado: e.pais_reportado,
            tot_creditos: e.tot_creditos,
            tot_debitos: e.tot_debitos,
            tot_creditos_mesma_titularidade: e.tot_creditos_mesma_titularidade,
            tot_debitos_mesma_titularidade: e.tot_debitos_mesma_titularidade,
            vlr_ult_dia: e.vlr_ult_dia,
        }
    }
}
