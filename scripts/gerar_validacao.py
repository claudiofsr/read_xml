#!/usr/bin/env python3
"""
gerar_validacao.py — Atualiza `src/xml_validation.rs` automaticamente a partir
das structs em `src/xml_structs/*.rs` e, opcionalmente, inspeciona diretórios reais
de XMLs (`--path`) para apontar tags não mapeadas com contexto da tag pai.

Exemplos:
    ./scripts/gerar_validacao.py
    ./scripts/gerar_validacao.py --path "/home/claudio/Documents/Derat_SPO/Magazine Luiza SA/Info da Receita"
    ./scripts/gerar_validacao.py --path "/home/claudio/Documents/Derat_SPO/Usina de Laticinios Jussara SA/Info da Receita"
"""

import argparse
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor
import os
from pathlib import Path
import re
import sys
import xml.parsers.expat

XML_STRUCTS_DIR = Path("src/xml_structs")
OUTPUT_FILE = Path("src/xml_validation.rs")

DOMINIOS = {
    "comum": [
        "agente",
        "aut_xml",
        "endereco",
        "entrega",
        "impostos",
        "integrated_dev_env",
        "ret_evento",
    ],
    "nfe": ["nfe", "nfe_detalhamento", "nfe_evento", "cancelamento_nfe", "pagamento"],
    "cte": ["cte", "cte_detalhamento", "cte_evento", "cancelamento_cte"],
    "efin": ["efinanceira"],
}

ROOT_TAGS = {
    "nfeProc",
    "NFe",
    "cteProc",
    "CTe",
    "eFinanceira",
    "procEventoCTe",
    "eventoCTe",
    "procEventoNFe",
    "evento",
    "procCancCTe",
    "procCancNFe",
    "GTVeProc",
    "GTVe",
    "cteSimpProc",
    "CTeSimp",
}

XMLDSIG_PREFIXES = (
    "Signature",
    "SignedInfo",
    "CanonicalizationMethod",
    "SignatureMethod",
    "Reference",
    "Transforms",
    "Transform",
    "DigestMethod",
    "DigestValue",
    "SignatureValue",
    "KeyInfo",
    "X509Data",
    "X509Certificate",
)

NFE_ROOTS = {"nfeProc", "NFe", "procEventoNFe", "evento", "procCancNFe"}
CTE_ROOTS = {
    "cteProc",
    "CTe",
    "GTVeProc",
    "GTVe",
    "cteSimpProc",
    "CTeSimp",
    "procEventoCTe",
    "eventoCTe",
    "procCancCTe",
}
EFIN_ROOTS = {"eFinanceira"}

RUST_TEMPLATE = r"""//! # Validador de Integridade de Esquemas XML
//!
//! Código autogerado pelo script `scripts/gerar_validacao.py`. Não edite este arquivo manualmente.

use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::events::Event;
use rayon::prelude::*;
use serde_aux::prelude::serde_introspect;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::XmlPathExt;
use crate::xml_structs::{
    agente, aut_xml, cancelamento, cancelamento_cte, cancelamento_nfe, cobranca, cte,
    cte_detalhamento, cte_evento, efinanceira, endereco, entrega, impostos, integrated_dev_env,
    nfe, nfe_detalhamento, nfe_evento, pagamento, ret_evento,
};

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de NF-e.
fn get_nfe_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
__NFE_CODE__
    adicionar_tags_comuns(&mut expected);
    expected
}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de CT-e.
fn get_cte_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
__CTE_CODE__
    adicionar_tags_comuns(&mut expected);
    expected
}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de e-Financeira.
fn get_efin_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
__EFIN_CODE__
    expected
}

/// Adiciona as tags comuns de estruturas transversais compartilhadas.
fn adicionar_tags_comuns(expected: &mut HashSet<&'static str>) {
__COMUM_CODE__
    // Adiciona cancelamento e cobrança comuns
    expected.extend(serde_introspect::<cancelamento::Cancelamento>());
    expected.extend(serde_introspect::<cancelamento::Retencao>());
    expected.extend(serde_introspect::<cancelamento::InfoCancelamento>());
    expected.extend(serde_introspect::<cobranca::Cobranca>());
    expected.extend(serde_introspect::<cobranca::Dup>());
    expected.extend(serde_introspect::<cobranca::Fat>());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum XmlDomain {
    Nfe,
    Cte,
    EFinanceira,
    Desconhecido,
}

#[derive(Debug)]
enum XmlCheckOutcome {
    Analisado {
        domain: XmlDomain,
        root_tag: String,
        total_tags: usize,
        unmapped: BTreeSet<String>,
    },
    ErroLeitura,
}

static NFE_EXPECTED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(get_nfe_expected_tags);
static CTE_EXPECTED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(get_cte_expected_tags);
static EFIN_EXPECTED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(get_efin_expected_tags);

#[inline]
fn is_root_or_xmldsig_tag(tag: &str) -> bool {
    matches!(
        tag,
        "nfeProc"
            | "NFe"
            | "cteProc"
            | "CTe"
            | "eFinanceira"
            | "procEventoCTe"
            | "eventoCTe"
            | "procEventoNFe"
            | "evento"
            | "procCancCTe"
            | "procCancNFe"
            | "GTVeProc"
            | "GTVe"
            | "cteSimpProc"
            | "CTeSimp"
    ) || tag.starts_with("Signature")
        || tag.starts_with("SignedInfo")
        || tag.starts_with("CanonicalizationMethod")
        || tag.starts_with("SignatureMethod")
        || tag.starts_with("Reference")
        || tag.starts_with("Transforms")
        || tag.starts_with("Transform")
        || tag.starts_with("DigestMethod")
        || tag.starts_with("DigestValue")
        || tag.starts_with("SignatureValue")
        || tag.starts_with("KeyInfo")
        || tag.starts_with("X509Data")
        || tag.starts_with("X509Certificate")
}

/// Realiza varredura SAX em passada única (detectando a tag raiz no próprio stream),
/// contabilizando total de tags lidas e tags não mapeadas.
fn inspecionar_xml(xml_path: &Path) -> XmlCheckOutcome {
    let Ok(mut reader) = xml_path.open_xml_reader_with_capacity(64 * 1024) else {
        return XmlCheckOutcome::ErroLeitura;
    };

    let mut buf = Vec::with_capacity(512);
    let mut domain: Option<XmlDomain> = None;
    let mut root_tag_name = String::new();
    let mut expected_tags: Option<&HashSet<&'static str>> = None;
    let mut total_tags = 0usize;
    let mut unmapped = BTreeSet::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let raw_name = e.name().into_inner();
                let clean_tag = raw_name.split(':').next_back().unwrap_or(raw_name);
                total_tags += 1;

                if domain.is_none() {
                    root_tag_name = clean_tag.to_string();
                    let (dom, exp) = match clean_tag {
                        "nfeProc" | "NFe" | "procEventoNFe" | "evento" | "procCancNFe" => {
                            (XmlDomain::Nfe, Some(&*NFE_EXPECTED_TAGS))
                        }
                        "cteProc" | "CTe" | "GTVeProc" | "GTVe" | "cteSimpProc" | "CTeSimp" | "procEventoCTe" | "eventoCTe" | "procCancCTe" => {
                            (XmlDomain::Cte, Some(&*CTE_EXPECTED_TAGS))
                        }
                        "eFinanceira" => (XmlDomain::EFinanceira, Some(&*EFIN_EXPECTED_TAGS)),
                        _ => (XmlDomain::Desconhecido, None),
                    };
                    domain = Some(dom);
                    expected_tags = exp;
                    if exp.is_none() {
                        break;
                    }
                } else if let Some(expected) = expected_tags
                    && !expected.contains(clean_tag)
                    && !is_root_or_xmldsig_tag(clean_tag)
                {
                    unmapped.insert(clean_tag.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return XmlCheckOutcome::ErroLeitura,
            _ => {}
        }
        buf.clear();
    }

    match domain {
        Some(dom) => XmlCheckOutcome::Analisado {
            domain: dom,
            root_tag: root_tag_name,
            total_tags,
            unmapped,
        },
        None => XmlCheckOutcome::ErroLeitura,
    }
}

/// Executa a varredura SAX concorrente em lote sobre todos os XMLs, exibe as
/// inconsistências agrupadas de forma determinística e imprime um resumo completo.
pub fn verificar_inconsistencias_de_xml(xml_paths: &[PathBuf]) {
    let total = xml_paths.len();
    let num_char = total.to_string().chars().count();

    let template_bar =
        "{prefix:<10.bold.dim} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>nchar}/{len:>nchar} ({eta})"
        .replace("nchar", &num_char.to_string());

    let style = match ProgressStyle::default_bar().template(&template_bar) {
        Ok(s) => s.progress_chars("#>-"),
        Err(_) => ProgressStyle::default_bar(),
    };

    let pb = ProgressBar::new(total as u64);
    pb.set_style(style);
    pb.set_prefix("check xml");

    let resultados: Vec<(&PathBuf, XmlCheckOutcome)> = xml_paths
        .par_iter()
        .map(|path| {
            pb.inc(1);
            (path, inspecionar_xml(path))
        })
        .collect();

    pb.finish_and_clear();

    let mut total_tags_lidas = 0usize;
    let mut nfe_count = 0usize;
    let mut cte_count = 0usize;
    let mut efin_count = 0usize;
    let mut desconhecidos_count = 0usize;
    let mut erros_count = 0usize;
    let mut conformes_count = 0usize;
    let mut inconsistentes_count = 0usize;

    let mut por_root_tag: BTreeMap<String, usize> = BTreeMap::new();
    let mut unknown_roots: BTreeMap<String, usize> = BTreeMap::new();
    let mut erros_amostra: Vec<&PathBuf> = Vec::new();
    let mut tags_unicas_nao_mapeadas: BTreeSet<String> = BTreeSet::new();
    let mut agrupado: HashMap<BTreeSet<String>, Vec<PathBuf>> = HashMap::new();

    for (path, outcome) in resultados {
        match outcome {
            XmlCheckOutcome::ErroLeitura => {
                erros_count += 1;
                if erros_amostra.len() < 5 {
                    erros_amostra.push(path);
                }
            }
            XmlCheckOutcome::Analisado {
                domain,
                root_tag,
                total_tags,
                unmapped,
            } => {
                total_tags_lidas += total_tags;
                match domain {
                    XmlDomain::Nfe => nfe_count += 1,
                    XmlDomain::Cte => cte_count += 1,
                    XmlDomain::EFinanceira => efin_count += 1,
                    XmlDomain::Desconhecido => {
                        desconhecidos_count += 1;
                        *unknown_roots.entry(root_tag).or_default() += 1;
                        continue;
                    }
                }

                *por_root_tag.entry(root_tag).or_default() += 1;

                if unmapped.is_empty() {
                    conformes_count += 1;
                } else {
                    inconsistentes_count += 1;
                    tags_unicas_nao_mapeadas.extend(unmapped.iter().cloned());
                    agrupado.entry(unmapped).or_default().push(path.clone());
                }
            }
        }
    }

    let mut agrupado_vec: Vec<(BTreeSet<String>, Vec<PathBuf>)> = agrupado.into_iter().collect();
    agrupado_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
    let num_grupos = agrupado_vec.len();

    for (tags, mut sorted_paths) in agrupado_vec {
        sorted_paths.sort();

        println!(
            "Aviso: Tags verificadas em arquivos XML não mapeadas em Structs: {:?}",
            tags
        );
        println!(
            "Total de {} arquivo(s) com esta inconsistência:",
            sorted_paths.len()
        );
        for path in sorted_paths {
            println!("  - {:?}", path);
        }
        println!();
    }

    let elegiveis = nfe_count + cte_count + efin_count;
    let pct = |n: usize| -> f64 {
        if elegiveis == 0 {
            0.0
        } else {
            (n as f64 * 100.0) / elegiveis as f64
        }
    };

    println!("=== Resumo da Verificação de Consistência (-k) ===");
    println!("Arquivos XML encontrados   : {total}");
    println!("Arquivos XML elegíveis     : {elegiveis}");
    println!("Total de tags inspecionadas: {total_tags_lidas}");
    println!(
        "Conformes (sem pendências) : {conformes_count} ({:.2}%)",
        pct(conformes_count)
    );
    println!(
        "Com tags não mapeadas      : {inconsistentes_count} ({:.2}%)",
        pct(inconsistentes_count)
    );
    println!("Ignorados (raiz desconhec.): {desconhecidos_count}");
    println!("Erros de leitura / XML     : {erros_count}");

    println!("\nDistribuição por domínio:");
    println!("  NF-e        : {nfe_count}");
    println!("  CT-e        : {cte_count}");
    println!("  e-Financeira: {efin_count}");

    if !por_root_tag.is_empty() {
        let mut roots_sorted: Vec<(&String, &usize)> = por_root_tag.iter().collect();
        roots_sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        println!("\nDetalhamento por tag raiz:");
        for (tag, qtd) in roots_sorted {
            println!("  {tag:<14}: {qtd}");
        }
    }

    if !unknown_roots.is_empty() {
        println!("\nTags raiz ignoradas:");
        for (tag, qtd) in &unknown_roots {
            println!("  {tag:<14}: {qtd}");
        }
    }

    if !erros_amostra.is_empty() {
        println!("\nAmostra de arquivos com erro de leitura/XML:");
        for p in erros_amostra {
            println!("  - {:?}", p);
        }
    }

    if tags_unicas_nao_mapeadas.is_empty() {
        println!(
            "\nStatus: 100% dos XMLs elegíveis estão em conformidade com as structs mapeadas."
        );
    } else {
        println!("\nGrupos de inconsistência   : {num_grupos}");
        println!(
            "Tags únicas não mapeadas ({}): {:?}",
            tags_unicas_nao_mapeadas.len(),
            tags_unicas_nao_mapeadas
        );
    }
    println!();
}

/// Varre o XML físico e retorna o conjunto de tags não mapeadas nas structs (ou `None` se conforme).
pub fn obter_tags_nao_mapeadas(xml_path: &Path) -> Option<HashSet<String>> {
    match inspecionar_xml(xml_path) {
        XmlCheckOutcome::Analisado { unmapped, .. } if !unmapped.is_empty() => {
            Some(unmapped.into_iter().collect())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_xml(filename: &str, content: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(filename);
        fs::write(&path, content).expect("Falha ao criar arquivo XML temporário de teste");
        path
    }

    #[test]
    fn test_xml_validation_complete() {
        let xml_valido_content = r#"<?xml version="1.0" encoding="utf-8"?>
        <nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe>
                <infNFe Id="NFe3522" versao="4.00">
                    <ide><cUF>35</cUF></ide>
                    <emit><CNPJ>00000000000100</CNPJ></emit>
                    <total><ICMSTot><vProd>150.00</vProd></ICMSTot></total>
                </infNFe>
            </NFe>
        </nfeProc>"#;

        let xml_incompleto_content = r#"<?xml version="1.0" encoding="utf-8"?>
        <nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe>
                <infNFe Id="NFe3522" versao="4.00">
                    <ide><cUF>35</cUF></ide>
                    <tagInexistenteDeTeste>valor_teste</tagInexistenteDeTeste>
                    <outroElementoFicticio>outro_valor</outroElementoFicticio>
                </infNFe>
            </NFe>
        </nfeProc>"#;

        let xml_desconhecido_content = r#"<?xml version="1.0" encoding="utf-8"?>
        <OutroDocumento><campo>valor</campo></OutroDocumento>"#;

        let path_valido = create_temp_xml("nfe_valido_teste.xml", xml_valido_content);
        let path_incompleto = create_temp_xml("nfe_incompleto_teste.xml", xml_incompleto_content);
        let path_desconhecido =
            create_temp_xml("desconhecido_teste.xml", xml_desconhecido_content);

        assert!(obter_tags_nao_mapeadas(&path_valido).is_none());
        assert!(obter_tags_nao_mapeadas(&path_desconhecido).is_none());

        let res_incompleto = obter_tags_nao_mapeadas(&path_incompleto);
        assert!(res_incompleto.is_some());

        let unmapped_tags = res_incompleto.unwrap();
        assert_eq!(unmapped_tags.len(), 2);
        assert!(unmapped_tags.contains("tagInexistenteDeTeste"));
        assert!(unmapped_tags.contains("outroElementoFicticio"));

        let _ = fs::remove_file(path_valido);
        let _ = fs::remove_file(path_incompleto);
        let _ = fs::remove_file(path_desconhecido);
    }
}
"""


def obter_structs_do_modulo(root: Path, modulo: str) -> list[str]:
    filepath = root / XML_STRUCTS_DIR / f"{modulo}.rs"
    if not filepath.exists():
        return []

    structs = []
    pattern = re.compile(r"^\s*pub\s+struct\s+(\w+)")
    with filepath.open("r", encoding="utf-8") as f:
        for line in f:
            if "#[cfg(test)]" in line:
                break
            m = pattern.match(line)
            if m:
                structs.append(m.group(1))
    return sorted(structs)


def obter_tags_mapeadas_do_modulo(root: Path, modulo: str) -> set[str]:
    """Extrai nomes de tags XML mapeadas via #[serde(rename/alias = "...")] e nomes de campos."""
    filepath = root / XML_STRUCTS_DIR / f"{modulo}.rs"
    if not filepath.exists():
        return set()

    text = filepath.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
    tags: set[str] = set()
    for m in re.finditer(r'(?:rename|alias)\s*=\s*"([^"]+)"', text):
        tag = m.group(1)
        if not tag.startswith("@") and tag != "$text":
            tags.add(tag)

    for m in re.finditer(r"^\s*pub\s+([a-zA-Z_]\w*)\s*:", text, flags=re.MULTILINE):
        tags.add(m.group(1))
    return tags


def gerar_codigo_introspeccao(root: Path, dominio_chaves: list[str]) -> str:
    linhas = []
    for modulo in dominio_chaves:
        for s in obter_structs_do_modulo(root, modulo):
            linhas.append(f"    expected.extend(serde_introspect::<{modulo}::{s}>());")
    return "\n".join(linhas)


def render_xml_validation_rs(root: Path) -> str:
    return (
        RUST_TEMPLATE.replace("__NFE_CODE__", gerar_codigo_introspeccao(root, DOMINIOS["nfe"]))
        .replace("__CTE_CODE__", gerar_codigo_introspeccao(root, DOMINIOS["cte"]))
        .replace("__EFIN_CODE__", gerar_codigo_introspeccao(root, DOMINIOS["efin"]))
        .replace("__COMUM_CODE__", gerar_codigo_introspeccao(root, DOMINIOS["comum"]))
    )


_WORKER_TAGS: dict[str, set[str]] = {}


def _init_worker(domain_tags: dict[str, set[str]]) -> None:
    global _WORKER_TAGS
    _WORKER_TAGS = domain_tags


def _decode_xml_bytes(raw: bytes) -> str:
    if raw.startswith((b"\xff\xfe", b"\xfe\xff")):
        return raw.decode("utf-16", errors="replace")
    if raw.startswith(b"\xef\xbb\xbf"):
        return raw[3:].decode("utf-8", errors="replace")

    head = raw[:512].decode("ascii", errors="ignore")
    m = re.search(r'encoding=["\']([^"\']+)["\']', head, flags=re.IGNORECASE)
    if m:
        enc = m.group(1)
        try:
            return raw.decode(enc, errors="replace")
        except LookupError:
            pass
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("windows-1252", errors="replace")


def _scan_xml_batch(paths: list[str]) -> list[tuple[str, str, str, bool, str]]:
    """Retorna lista de (domínio, parent_tag, tag_ausente, is_container, caminho)."""
    findings: list[tuple[str, str, str, bool, str]] = []

    for path_str in paths:
        try:
            raw = Path(path_str).read_bytes()
        except OSError:
            continue

        text = re.sub(r"^\s*<\?xml[^>]*\?>", "", _decode_xml_bytes(raw), count=1)

        stack: list[tuple[str, bool]] = []
        root_tag: str | None = None
        expected: set[str] | None = None
        domain = ""
        file_findings: list[tuple[str, str, str, bool]] = []

        parser = xml.parsers.expat.ParserCreate()

        def start_elem(name: str, _attrs: dict) -> None:
            nonlocal root_tag, expected, domain
            clean = name.rsplit(":", 1)[-1]
            if stack:
                stack[-1] = (stack[-1][0], True)
            parent = stack[-1][0] if stack else "<root>"
            stack.append((clean, False))

            if root_tag is None:
                root_tag = clean
                if clean in NFE_ROOTS:
                    domain = "NFe"
                    expected = _WORKER_TAGS["nfe"]
                elif clean in CTE_ROOTS:
                    domain = "CTe"
                    expected = _WORKER_TAGS["cte"]
                elif clean in EFIN_ROOTS:
                    domain = "eFinanceira"
                    expected = _WORKER_TAGS["efin"]
                return

            if (
                expected is not None
                and clean not in expected
                and clean not in ROOT_TAGS
                and not clean.startswith(XMLDSIG_PREFIXES)
            ):
                file_findings.append((domain, parent, clean, False))

        def end_elem(_name: str) -> None:
            tag, had_children = stack.pop()
            if had_children and file_findings:
                for idx in range(len(file_findings) - 1, -1, -1):
                    d, p, t, _ = file_findings[idx]
                    if t == tag:
                        file_findings[idx] = (d, p, t, True)

        parser.StartElementHandler = start_elem
        parser.EndElementHandler = end_elem
        try:
            parser.Parse(text, True)
        except xml.parsers.expat.ExpatError:
            continue

        if expected is not None and file_findings:
            seen_local: set[tuple[str, str, str]] = set()
            for d, p, t, is_cont in file_findings:
                key = (d, p, t)
                if key not in seen_local:
                    seen_local.add(key)
                    findings.append((d, p, t, is_cont, path_str))

    return findings


def scan_directories(root: Path, dirs: list[Path]) -> int:
    comum = set().union(*(obter_tags_mapeadas_do_modulo(root, m) for m in DOMINIOS["comum"]))
    comum |= set().union(
        *(obter_tags_mapeadas_do_modulo(root, m) for m in ("cancelamento", "cobranca"))
    )
    domain_tags = {
        "nfe": comum.union(*(obter_tags_mapeadas_do_modulo(root, m) for m in DOMINIOS["nfe"])),
        "cte": comum.union(*(obter_tags_mapeadas_do_modulo(root, m) for m in DOMINIOS["cte"])),
        "efin": obter_tags_mapeadas_do_modulo(root, "efinanceira"),
    }

    xml_files: list[str] = []
    for d in dirs:
        if not d.exists():
            print(f"Aviso: diretório não encontrado: {d}", file=sys.stderr)
            continue
        for dirpath, _, filenames in os.walk(d):
            for fn in filenames:
                if fn.lower().endswith(".xml"):
                    xml_files.append(os.path.join(dirpath, fn))

    if not xml_files:
        print("Nenhum arquivo .xml encontrado nos diretórios informados.")
        return 0

    print(f"Inspecionando {len(xml_files)} arquivo(s) XML em {len(dirs)} diretório(s)...")
    batch_size = max(100, min(2000, len(xml_files) // (os.cpu_count() or 4) + 1))
    batches = [xml_files[i : i + batch_size] for i in range(0, len(xml_files), batch_size)]

    grouped: dict[tuple[str, str, str], dict] = defaultdict(
        lambda: {"count": 0, "container": False, "samples": []}
    )

    with ProcessPoolExecutor(initializer=_init_worker, initargs=(domain_tags,)) as pool:
        for batch_findings in pool.map(_scan_xml_batch, batches):
            for domain, parent, tag, is_cont, sample in batch_findings:
                entry = grouped[(domain, parent, tag)]
                entry["count"] += 1
                entry["container"] |= is_cont
                if len(entry["samples"]) < 2:
                    entry["samples"].append(sample)

    if not grouped:
        print("Excelente: 0 tags não mapeadas encontradas nos diretórios inspecionados!")
        return 0

    print(f"\nEncontradas {len(grouped)} ocorrência(s) de (domínio, pai, tag) não mapeadas:\n")
    for (domain, parent, tag), info in sorted(
        grouped.items(), key=lambda kv: (-kv[1]["count"], kv[0])
    ):
        kind = "struct/container" if info["container"] else "Option<String>"
        snake = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", tag).lower()
        print(
            f"  [{domain}] pai <{parent}> -> tag <{tag}> ({kind}, {info['count']} arquivos)"
        )
        print(f'      Sugestão Rust: #[serde(rename = "{tag}", default)] pub {snake}: {kind},')
        for s in info["samples"]:
            print(f"      Exemplo: {s}")
    return 2


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Gera/atualiza src/xml_validation.rs e opcionalmente inspeciona diretórios com XMLs."
    )
    parser.add_argument(
        "-p",
        "--path",
        action="append",
        type=Path,
        help="Diretório com arquivos XML (NFe/CTe/eFinanceira) para auditar tags não mapeadas",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Apenas verifica se src/xml_validation.rs está atualizado (não sobrescreve)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    if not (root / "Cargo.toml").exists() or not (root / XML_STRUCTS_DIR).exists():
        print("Erro: não foi possível localizar a raiz do projeto read_xml.", file=sys.stderr)
        return 1

    rendered = render_xml_validation_rs(root)
    out_path = root / OUTPUT_FILE
    current = out_path.read_text(encoding="utf-8") if out_path.exists() else ""

    if args.check:
        if current != rendered:
            print(f"Desatualizado: {out_path} difere do gerado pelas structs.", file=sys.stderr)
            return 1
        print(f"OK: {out_path} está sincronizado com {XML_STRUCTS_DIR}.")
    else:
        if current != rendered:
            out_path.write_text(rendered, encoding="utf-8")
            print(f"Atualizado: {out_path} a partir de {XML_STRUCTS_DIR}.")
        else:
            print(f"OK: {out_path} já está atualizado.")

    if args.path:
        return scan_directories(root, args.path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
