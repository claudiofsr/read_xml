//! # Leitura Paralela Streaming de Fontes XML (`.zip` e `.xml` soltos)
//!
//! Lê arquivos `.zip` diretamente sem extrair nenhum XML no disco, paralelizando
//! a descompressão entre múltiplos núcleos mesmo quando há poucos arquivos `.zip`,
//! e lê também arquivos `.xml` já descompactados (soltos) em paralelo via Rayon.

use indicatif::ProgressBar;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{self, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{Arguments, XmlParserError, XmlParserResult, get_path};

/// Leitor posicionado sem lock compartilhado de cursor no SO (`pread` no Unix).
/// Permite clonar `ZipArchive<PreadFile>` entre threads do Rayon com zero cópias do arquivo.
#[derive(Clone)]
struct PreadFile {
    file: Arc<File>,
    len: u64,
    pos: u64,
}

impl PreadFile {
    fn open(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self {
            file: Arc::new(file),
            len,
            pos: 0,
        })
    }
}

impl Read for PreadFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        #[cfg(unix)]
        let n = {
            use std::os::unix::fs::FileExt;
            self.file.read_at(buf, self.pos)?
        };
        #[cfg(windows)]
        let n = {
            use std::os::windows::fs::FileExt;
            self.file.seek_read(buf, self.pos)?
        };
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for PreadFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let next = match pos {
            SeekFrom::Start(n) => Some(n),
            SeekFrom::End(offset) => self.len.checked_add_signed(offset),
            SeekFrom::Current(offset) => self.pos.checked_add_signed(offset),
        };
        self.pos = next.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Tentativa de seek antes do início do arquivo ZIP",
            )
        })?;
        Ok(self.pos)
    }
}

#[inline]
fn has_ext_ignore_case(name: &str, ext: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

/// Retorna `true` se o caminho possui extensão `.xml` (case-insensitive).
#[inline]
fn is_xml_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
}

/// Descobre os arquivos `.zip` e `.xml` a partir de `-p` / `--path`
/// (arquivo único ou diretório recursivo).
pub fn get_xml_entries(arguments: &Arguments) -> XmlParserResult<Vec<PathBuf>> {
    let target = get_path(&arguments.path)?;

    if target.is_file() {
        let valid = target
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("xml"));
        if valid {
            return Ok(vec![target]);
        }
        return Err(XmlParserError::InvalidDocument(format!(
            "O arquivo '{}' não possui extensão .zip nem .xml",
            target.display()
        )));
    }

    let max_depth = arguments.max_depth.unwrap_or(usize::MAX);
    let mut entries: Vec<PathBuf> = WalkDir::new(target)
        .max_depth(max_depth)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("xml")
            })
        })
        .map(|entry| entry.into_path())
        .collect();

    entries.sort();
    Ok(entries)
}

fn count_nested_zip_xmls(bytes: &[u8], depth: usize) -> u64 {
    if depth >= 8 {
        return 0;
    }
    let Ok(mut nested) = ZipArchive::new(Cursor::new(bytes)) else {
        return 0;
    };
    let mut total = 0u64;
    let mut inner_buf = Vec::new();
    for idx in 0..nested.len() {
        let Ok(mut entry) = nested.by_index(idx) else {
            continue;
        };
        if entry.is_dir() {
            continue;
        }
        let Ok(name) = entry.name() else {
            continue;
        };
        if has_ext_ignore_case(&name, "xml") {
            total += 1;
        } else if has_ext_ignore_case(&name, "zip") {
            inner_buf.clear();
            if entry.read_to_end(&mut inner_buf).is_ok() {
                total += count_nested_zip_xmls(&inner_buf, depth + 1);
            }
        }
    }
    total
}

fn inspect_zip_xml_count(path: &Path) -> u64 {
    // XML solto (já descompactado) conta como uma única entrada.
    if is_xml_path(path) {
        return 1;
    }
    let Ok(pread) = PreadFile::open(path) else {
        return 0;
    };
    let Ok(mut archive) = ZipArchive::new(pread) else {
        return 0;
    };

    let mut direct_xml = 0u64;
    let mut nested_indices = Vec::new();

    for idx in 0..archive.len() {
        let Some(Ok(name)) = archive.name_for_index(idx) else {
            continue;
        };
        let name = name.as_ref();

        if name.ends_with('/') {
            continue;
        }
        if has_ext_ignore_case(name, "xml") {
            direct_xml += 1;
        } else if has_ext_ignore_case(name, "zip") {
            nested_indices.push(idx);
        }
    }

    if nested_indices.is_empty() {
        return direct_xml;
    }

    let mut nested_xml = 0u64;
    let mut buf = Vec::new();
    for idx in nested_indices {
        if let Ok(mut entry) = archive.by_index(idx) {
            buf.clear();
            if entry.read_to_end(&mut buf).is_ok() {
                nested_xml += count_nested_zip_xmls(&buf, 1);
            }
        }
    }
    direct_xml + nested_xml
}

/// Filtra as fontes que contêm XML: `.zip` com pelo menos um `.xml` (direto ou aninhado)
/// e arquivos `.xml` soltos, retornando `(fontes_com_xml, qtd_zips_ignorados_sem_xml, total_entradas)`.
pub fn partition_sources_with_xml(zip_paths: &[PathBuf]) -> (Vec<PathBuf>, usize, u64) {
    let counts: Vec<u64> = zip_paths.iter().map(|p| inspect_zip_xml_count(p)).collect();

    let mut active = Vec::with_capacity(zip_paths.len());
    let mut skipped = 0usize;
    let mut total_entries = 0u64;

    for (path, count) in zip_paths.iter().zip(counts) {
        if count > 0 {
            active.push(path.clone());
            total_entries += count;
        } else {
            skipped += 1;
        }
    }

    (active, skipped, total_entries)
}

/// Conta o total de entradas XML (Central Directory dos `.zip`s + `.xml` soltos)
/// para dimensionar barras de progresso.
pub fn count_source_entries(zip_paths: &[PathBuf]) -> u64 {
    zip_paths
        .par_iter()
        .map(|path| {
            if is_xml_path(path) {
                return 1;
            }
            PreadFile::open(path)
                .ok()
                .and_then(|f| ZipArchive::new(f).ok())
                .map_or(0, |a| a.len() as u64)
        })
        .sum()
}

fn process_nested_zip<F>(
    bytes: &[u8],
    parent_label: &str,
    pb: Option<&ProgressBar>,
    on_xml: &F,
    depth: usize,
) where
    F: Fn(&str, &[u8]) + Sync,
{
    const MAX_NESTED_DEPTH: usize = 8;
    if depth >= MAX_NESTED_DEPTH {
        eprintln!("Aviso: profundidade máxima de ZIP aninhado atingida em '{parent_label}'");
        return;
    }

    let Ok(mut nested) = ZipArchive::new(Cursor::new(bytes)) else {
        eprintln!("Aviso: ZIP aninhado inválido '{parent_label}'");
        return;
    };

    let mut inner_buf = Vec::with_capacity(64 * 1024);
    for idx in 0..nested.len() {
        let Ok(mut entry) = nested.by_index(idx) else {
            continue;
        };
        if entry.is_dir() {
            continue;
        }
        let Ok(name) = entry.name() else {
            continue;
        };
        let full_label = format!("{parent_label}::{name}");
        if has_ext_ignore_case(&name, "xml") {
            if let Some(bar) = pb {
                bar.inc(1);
            }
            inner_buf.clear();
            if entry.read_to_end(&mut inner_buf).is_ok() {
                on_xml(&full_label, &inner_buf);
            }
        } else if has_ext_ignore_case(&name, "zip") {
            inner_buf.clear();
            if entry.read_to_end(&mut inner_buf).is_ok() {
                process_nested_zip(&inner_buf, &full_label, pb, on_xml, depth + 1);
            }
        }
    }
}

/// Percorre todas as entradas `.xml` (e `.zip` aninhados) da lista de fontes,
/// incluindo arquivos `.xml` soltos, invocando `on_xml(label, bytes)` em paralelo
/// com buffers reutilizáveis por thread.
pub fn for_each_xml<F>(
    source_paths: &[PathBuf],
    pb: Option<&ProgressBar>,
    on_xml: F,
) -> XmlParserResult<()>
where
    F: Fn(&str, &[u8]) + Sync,
{
    // Arquivos .xml soltos (já descompactados): leitura paralela entre arquivos.
    let (xml_paths, zip_paths): (Vec<&PathBuf>, Vec<&PathBuf>) =
        source_paths.iter().partition(|p| is_xml_path(p));

    xml_paths.par_iter().for_each(|xml_path| {
        if let Some(bar) = pb {
            bar.inc(1);
        }
        match std::fs::read(xml_path) {
            Ok(bytes) => on_xml(&xml_path.display().to_string(), &bytes),
            Err(err) => eprintln!("Aviso: falha ao ler XML {}: {err}", xml_path.display()),
        }
    });

    for zip_path in zip_paths {
        let pread = match PreadFile::open(zip_path) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Aviso: falha ao abrir ZIP {}: {err}", zip_path.display());
                continue;
            }
        };
        let archive = match ZipArchive::new(pread) {
            Ok(a) => a,
            Err(err) => {
                eprintln!("Aviso: ZIP corrompido {}: {err}", zip_path.display());
                continue;
            }
        };

        let zip_display = zip_path.display().to_string();
        let total = archive.len();

        (0..total).into_par_iter().for_each_init(
            || (archive.clone(), Vec::with_capacity(64 * 1024)),
            |(local_archive, buf), idx| {
                let Ok(mut entry) = local_archive.by_index(idx) else {
                    return;
                };
                if entry.is_dir() {
                    return;
                }

                let Ok(name) = entry.name() else {
                    return;
                };
                let label = format!("{zip_display}::{name}");

                if has_ext_ignore_case(&name, "xml") {
                    if let Some(bar) = pb {
                        bar.inc(1);
                    }
                    buf.clear();
                    if let Err(err) = entry.read_to_end(buf) {
                        eprintln!("Aviso: falha ao descomprimir '{label}': {err}");
                        return;
                    }
                    on_xml(&label, buf);
                } else if has_ext_ignore_case(&name, "zip") {
                    buf.clear();
                    if let Err(err) = entry.read_to_end(buf) {
                        eprintln!("Aviso: falha ao ler ZIP aninhado '{label}': {err}");
                        return;
                    }
                    process_nested_zip(buf, &label, pb, &on_xml, 1);
                }
                if buf.capacity() > 4 * 1024 * 1024 {
                    *buf = Vec::with_capacity(64 * 1024);
                }
            },
        );
    }

    Ok(())
}
