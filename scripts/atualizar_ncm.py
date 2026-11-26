#!/usr/bin/env python3
"""
atualizar_ncm.py — Baixa a tabela NCM oficial do Portal Único Siscomex (Classif/RFB)
e gera `src/export/ncm.rs` com descrições hierárquicas concatenadas no padrão Receita Federal.

Uso:
    ./scripts/atualizar_ncm.py
    ./scripts/atualizar_ncm.py --json caminho/local/Tabela_NCM.json
"""

import argparse
from datetime import datetime
import json
from pathlib import Path
import re
import sys
from urllib.request import Request, urlopen

SISCOMEX_NCM_URL = (
    "https://portalunico.siscomex.gov.br/classif/api/publico/nomenclatura/download/json"
)


def clean_desc(raw: str) -> str:
    text = re.sub(r"<[^>]+>", "", raw)
    text = re.sub(r"^\s*-+\s*", "", text)
    text = " ".join(text.split()).strip()
    if text and not text.endswith((".", "!", "?")):
        text += "."
    return text


def build_hierarchical_table(data: dict) -> list[tuple[int, str]]:
    items = data.get("Nomenclaturas", [])
    if not items:
        raise RuntimeError("Campo 'Nomenclaturas' vazio ou ausente no JSON.")

    desc_by_digits: dict[str, str] = {}
    items_8: list[str] = []

    for entry in items:
        cod_raw = str(entry.get("Codigo", "")).strip()
        digits = re.sub(r"\D", "", cod_raw)
        desc = clean_desc(str(entry.get("Descricao", "")))
        if not digits or not desc:
            continue
        desc_by_digits[digits] = desc
        if len(digits) == 8:
            items_8.append(digits)

    out: list[tuple[int, str]] = []
    for digits in items_8:
        parts: list[str] = []
        # Hierarquia Receita Federal: Posição (4) + Subposição (5/6) + Item (7) + Subitem (8)
        for prefix_len in (4, 5, 6, 7, 8):
            p = digits[:prefix_len]
            d = desc_by_digits.get(p)
            if d and (not parts or parts[-1] != d):
                parts.append(d)
        out.append((int(digits), " ".join(parts)))

    dedup: dict[int, str] = {}
    for code, desc in out:
        dedup[code] = desc
    return sorted(dedup.items(), key=lambda x: x[0])


def render_rust(rows: list[tuple[int, str]], meta: str) -> str:
    lines = [
        "//! # Tabela Oficial NCM (Nomenclatura Comum do Mercosul)",
        "//!",
        f"//! Gerado automaticamente por `./scripts/atualizar_ncm.py` ({meta}).",
        "//! Fonte: Portal Único Siscomex / Receita Federal (Sistema Classif).",
        "",
        'const ND: &str = "<N/D>";',
        "",
        "/// Busca binária O(log N) na tabela NCM vigente.",
        "pub fn lookup_ncm_desc(ncm: &str) -> &'static str {",
        "    let digits: String = ncm.chars().filter(char::is_ascii_digit).collect();",
        "    let Ok(code) = digits.parse::<u32>() else {",
        "        return ND;",
        "    };",
        "",
        "    match NCM_TABLE.binary_search_by_key(&code, |&(k, _)| k) {",
        "        Ok(idx) => NCM_TABLE[idx].1,",
        "        Err(_) => ND,",
        "    }",
        "}",
        "",
        f"/// {len(rows)} códigos NCM de 8 dígitos ordenados para busca binária.",
        "static NCM_TABLE: &[(u32, &str)] = &[",
    ]
    for code, desc in rows:
        lines.append(f"    ({code}, {json.dumps(desc, ensure_ascii=False)}),")
    lines.extend(
        [
            "];",
            "",
            "#[cfg(test)]",
            "mod tests_ncm {",
            "    use super::*;",
            "",
            "    #[test]",
            "    fn test_ncm_table_is_strictly_sorted() {",
            "        assert!(",
            "            NCM_TABLE.windows(2).all(|w| w[0].0 < w[1].0),",
            '            "NCM_TABLE deve estar estritamente ordenada"',
            "        );",
            "    }",
            "",
            "    #[test]",
            "    fn test_lookup_ncm_invalid_returns_nd() {",
            '        assert_eq!(lookup_ncm_desc(""), "<N/D>");',
            '        assert_eq!(lookup_ncm_desc("abc"), "<N/D>");',
            "    }",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Atualiza src/export/ncm.rs via JSON oficial Siscomex.")
    parser.add_argument("--json", type=Path, help="Caminho para JSON local já baixado")
    args = parser.parse_args()

    if args.json:
        raw = args.json.read_text(encoding="utf-8")
    else:
        print(f"Baixando tabela NCM oficial de {SISCOMEX_NCM_URL} ...")
        req = Request(SISCOMEX_NCM_URL, headers={"User-Agent": "read_xml/1.0"})
        with urlopen(req, timeout=60) as resp:
            charset = resp.headers.get_content_charset() or "utf-8"
            raw = resp.read().decode(charset, errors="replace")

    data = json.loads(raw)
    rows = build_hierarchical_table(data)
    meta = data.get("Data_Ultima_Atualizacao_NCM", datetime.now().strftime("%Y-%m-%d"))

    out_path = Path(__file__).resolve().parent.parent / "src/export/ncm.rs"
    if not out_path.parent.exists():
        print("Erro: execute a partir da raiz do projeto read_xml.", file=sys.stderr)
        return 1

    out_path.write_text(render_rust(rows, meta), encoding="utf-8")
    print(f"Atualizado {out_path} com {len(rows)} códigos NCM (vigência/atualização: {meta}).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
