#!/usr/bin/env python3
"""
atualizar_cfop.py — Atualiza `src/export/cfop.rs` a partir de fonte oficial de CFOP:
  - CONFAZ (Anexo II Convênio s/nº 1970 vigente — HTML oficial)
  - Portal Nacional da NF-e (`--xlsx IT_2023.002_Tabela_CFOP.xlsx`)
  - Arquivo CSV local (`--csv tabela_cfop.csv`)

Sem dependências externas (usa apenas biblioteca padrão do Python).
"""

import argparse
import csv
from datetime import datetime
from html import unescape
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sys
from urllib.request import Request, urlopen
import xml.etree.ElementTree as ET
import zipfile

CONFAZ_CFOP_URL = (
    "https://www.confaz.fazenda.gov.br/legislacao/ajustes/sinief/cfop_cvsn_70_vigente"
)


class _TextExtractor(HTMLParser):
    BLOCK_TAGS = {"p", "div", "tr", "li", "br", "h1", "h2", "h3", "h4", "h5", "h6"}

    def __init__(self) -> None:
        super().__init__()
        self.chunks: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        t = tag.lower()
        if t in self.BLOCK_TAGS:
            self.chunks.append("\n")
        elif t in {"td", "th"}:
            self.chunks.append(" ")

    def handle_endtag(self, tag: str) -> None:
        if tag.lower() in self.BLOCK_TAGS:
            self.chunks.append("\n")

    def handle_data(self, data: str) -> None:
        self.chunks.append(data)


def clean_desc(raw: str) -> str:
    text = " ".join(raw.split()).strip()
    return text.rstrip(".") if text.endswith("..") else text


def parse_confaz_html(html_text: str) -> dict[int, str]:
    parser = _TextExtractor()
    parser.feed(html_text)
    full_text = unescape("".join(parser.chunks))

    rows: dict[int, str] = {}
    # Aceita formatos "1.101 - Compra para..." ou "1101 - Compra para..."
    pattern = re.compile(r"(?m)^\s*([1-7]\.?[0-9]{3})\s*[-–]\s*(.+?)\s*$")
    for m in pattern.finditer(full_text):
        code = int(m.group(1).replace(".", ""))
        desc = clean_desc(m.group(2))
        if desc and code not in rows:
            rows[code] = desc
    return rows


def parse_xlsx_stdlib(xlsx_path: Path) -> dict[int, str]:
    """Lê planilha XLSX oficial (IT 2023.002 Portal NF-e) usando apenas zipfile + ElementTree."""
    ns = {"s": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
    with zipfile.ZipFile(xlsx_path) as zf:
        shared: list[str] = []
        if "xl/sharedStrings.xml" in zf.namelist():
            root_ss = ET.fromstring(zf.read("xl/sharedStrings.xml"))
            for si in root_ss.findall("s:si", ns):
                shared.append("".join(t.text or "" for t in si.iterfind(".//s:t", ns)))

        sheet_name = next(
            n for n in sorted(zf.namelist()) if n.startswith("xl/worksheets/sheet") and n.endswith(".xml")
        )
        root_ws = ET.fromstring(zf.read(sheet_name))

        def cell_val(c: ET.Element) -> str:
            t = c.attrib.get("t", "")
            v = c.find("s:v", ns)
            if v is None or v.text is None:
                is_el = c.find("s:is", ns)
                return "".join(x.text or "" for x in is_el.iterfind(".//s:t", ns)) if is_el is not None else ""
            if t == "s":
                return shared[int(v.text)]
            return v.text

        def col_index_from_ref(ref: str) -> int | None:
            letters = "".join(ch for ch in ref if ch.isalpha()).upper()
            if not letters:
                return None
            idx = 0
            for ch in letters:
                idx = idx * 26 + (ord(ch) - ord("A") + 1)
            return idx - 1

        table_rows: list[list[str]] = []
        for row in root_ws.findall(".//s:sheetData/s:row", ns):
            row_vals: list[str] = []
            for pos, c in enumerate(row.findall("s:c", ns)):
                col_idx = col_index_from_ref(c.attrib.get("r", ""))
                if col_idx is None:
                    col_idx = pos
                while len(row_vals) <= col_idx:
                    row_vals.append("")
                row_vals[col_idx] = cell_val(c).strip()
            if any(row_vals):
                table_rows.append(row_vals)

    if not table_rows:
        raise RuntimeError(f"Nenhuma linha encontrada em {xlsx_path}.")

    # Localiza linha de cabeçalho contendo CFOP e DESCRIÇÃO/Título
    header_idx = 0
    cfop_col = 0
    desc_col = 1
    found_header = False
    for idx, r in enumerate(table_rows[:15]):
        upper = [c.upper() for c in r]
        for ci, col in enumerate(upper):
            if "CFOP" in col and "GRUPO" not in col:
                header_idx = idx
                cfop_col = ci
                for di, dcol in enumerate(upper):
                    if di != ci and ("TÍTULO" in dcol or "TITULO" in dcol or "DESCRI" in dcol):
                        desc_col = di
                        break
                found_header = True
                break
        if found_header:
            break

    rows: dict[int, str] = {}
    for r in table_rows[header_idx + 1 :]:
        if max(cfop_col, desc_col) >= len(r):
            continue
        digits = re.sub(r"\D", "", r[cfop_col])
        desc = clean_desc(r[desc_col])
        if len(digits) == 4 and desc:
            rows[int(digits)] = desc
    return rows


def parse_csv_file(csv_path: Path) -> dict[int, str]:
    raw = csv_path.read_text(encoding="utf-8-sig")
    delim = ";" if raw.count(";") >= raw.count(",") else ","
    rows: dict[int, str] = {}
    for r in csv.reader(raw.splitlines(), delimiter=delim):
        if len(r) < 2:
            continue
        digits = re.sub(r"\D", "", r[0])
        desc = clean_desc(r[1])
        if len(digits) == 4 and desc:
            rows[int(digits)] = desc
    return rows


def render_rust(rows: dict[int, str], source_note: str) -> str:
    sorted_items = sorted(rows.items(), key=lambda x: x[0])
    lines = [
        "//! # Tabela Oficial CONFAZ de CFOP (Código Fiscal de Operações e Prestações)",
        "//!",
        f"//! Gerado automaticamente por `./scripts/atualizar_cfop.py` ({source_note}).",
        "",
        'const ND: &str = "<N/D>";',
        "",
        "/// Busca binária O(log N) na tabela oficial CONFAZ de descrições de CFOP.",
        "pub fn lookup_cfop_desc(cfop: &str) -> &'static str {",
        "    let Ok(code) = cfop.trim().parse::<u16>() else {",
        "        return ND;",
        "    };",
        "",
        "    match CFOP_TABLE.binary_search_by_key(&code, |&(k, _)| k) {",
        "        Ok(idx) => CFOP_TABLE[idx].1,",
        "        Err(_) => ND,",
        "    }",
        "}",
        "",
        f"/// {len(sorted_items)} códigos CFOP ordenados para busca binária.",
        "static CFOP_TABLE: &[(u16, &str)] = &[",
    ]
    for code, desc in sorted_items:
        lines.append(f"    ({code}, {json.dumps(desc, ensure_ascii=False)}),")
    lines.extend(
        [
            "];",
            "",
            "#[cfg(test)]",
            "mod tests_cfop {",
            "    use super::*;",
            "",
            "    #[test]",
            "    fn test_cfop_table_is_strictly_sorted() {",
            "        assert!(",
            "            CFOP_TABLE.windows(2).all(|w| w[0].0 < w[1].0),",
            '            "CFOP_TABLE deve estar estritamente ordenada"',
            "        );",
            "    }",
            "",
            "    #[test]",
            "    fn test_lookup_cfop_desc() {",
            "        assert_eq!(",
            '            lookup_cfop_desc("5352"),',
            '            "Prestação de serviço de transporte a estabelecimento industrial"',
            "        );",
            "        assert_eq!(",
            '            lookup_cfop_desc("7949"),',
            '            "Outra saída de mercadoria ou prestação de serviço não especificado"',
            "        );",
            '        assert_eq!(lookup_cfop_desc("9999"), "<N/D>");',
            "    }",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Atualiza src/export/cfop.rs a partir de fonte oficial.")
    parser.add_argument("--xlsx", type=Path, help="Planilha XLSX oficial (Portal NF-e IT 2023.002)")
    parser.add_argument("--csv", type=Path, help="Arquivo CSV local (codigo;descricao)")
    parser.add_argument("--html", type=Path, help="HTML local baixado da página do CONFAZ")
    args = parser.parse_args()

    if args.xlsx:
        rows = parse_xlsx_stdlib(args.xlsx)
        note = f"XLSX {args.xlsx.name} em {datetime.now().strftime('%Y-%m-%d')}"
    elif args.csv:
        rows = parse_csv_file(args.csv)
        note = f"CSV {args.csv.name} em {datetime.now().strftime('%Y-%m-%d')}"
    elif args.html:
        rows = parse_confaz_html(args.html.read_text(encoding="utf-8", errors="replace"))
        note = f"HTML {args.html.name} em {datetime.now().strftime('%Y-%m-%d')}"
    else:
        print(f"Baixando tabela CFOP oficial de {CONFAZ_CFOP_URL} ...")
        req = Request(CONFAZ_CFOP_URL, headers={"User-Agent": "read_xml/1.0"})
        with urlopen(req, timeout=60) as resp:
            html_text = resp.read().decode("utf-8", errors="replace")
        rows = parse_confaz_html(html_text)
        note = f"CONFAZ em {datetime.now().strftime('%Y-%m-%d')}"

    if len(rows) < 100:
        print(
            f"Erro: apenas {len(rows)} códigos CFOP extraídos; verifique o formato da fonte.",
            file=sys.stderr,
        )
        return 1

    out_path = Path(__file__).resolve().parent.parent / "src/export/cfop.rs"
    if not out_path.parent.exists():
        print("Erro: execute a partir da raiz do projeto read_xml.", file=sys.stderr)
        return 1

    out_path.write_text(render_rust(rows, note), encoding="utf-8")
    print(f"Atualizado {out_path} com {len(rows)} códigos CFOP ({note}).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
