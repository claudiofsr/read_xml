#!/usr/bin/env python3
"""
comparar_receita.py — Compara CSVs da Receita Federal (2025/2026) contra os CSVs
gerados pelo `read-xml -r`, selecionando uma amostra de N chaves em comum (ou todas),
ordenando por (Chave, NumeroItem) e apontando divergências coluna a coluna.

Exemplos:
  ./comparar_receita.py 2026-CTe-Destinatario-mes01.csv documentos_fiscais-ctes.csv -n 100
  ./comparar_receita.py 2025-CTe-Destinatario.csv gerado-2025-CTe-Destinatario.csv -n 500
  ./comparar_receita.py 2025-NFe-Destinatario-Sem01.csv gerado-2025-NFe-Destinatario-Sem01.csv -n 500 --save-dir /tmp/cmp_nfe
  ./comparar_receita.py 2025-CTe-Destinatario.csv gerado-2025-CTe-Destinatario.csv --all
"""

import argparse
from collections import defaultdict
import csv
from pathlib import Path
import random
import re
import sys

csv.field_size_limit(min(sys.maxsize, 2**31 - 1))

KEY_COL_EXACT = "Chave da Nota Fiscal Eletrônica : NF Item (Todos)"
ITEM_COL_EXACT = "Número do Item : NF Item (Todos)"
RE_APOSTROPHE_KEY = re.compile(r"^'([0-9A-Za-z]{44})'$")


def read_csv_auto(path: Path) -> tuple[list[str], list[dict[str, str]], str]:
    raw_bytes = path.read_bytes()
    encoding_used = "utf-8-sig"
    try:
        text = raw_bytes.decode("utf-8-sig")
    except UnicodeDecodeError:
        encoding_used = "cp1252"
        text = raw_bytes.decode("cp1252", errors="replace")

    first_line = text.splitlines()[0] if text else ""
    delim = ";" if first_line.count(";") >= first_line.count(",") else ","

    reader = csv.DictReader(text.splitlines(), delimiter=delim)
    headers = list(reader.fieldnames or [])
    rows = [dict(r) for r in reader]
    return headers, rows, encoding_used


def clean_val(val: str | None) -> str:
    if val is None:
        return ""
    s = val.strip()
    m = RE_APOSTROPHE_KEY.match(s)
    if m:
        return m.group(1)
    return s


def find_col(headers: list[str], exact: str, prefix: str) -> str:
    if exact in headers:
        return exact
    for h in headers:
        if h.startswith(prefix):
            return h
    raise RuntimeError(f"Coluna '{exact}' não encontrada no cabeçalho.")


def item_sort_key(val: str) -> tuple[int, int, str]:
    s = val.strip()
    if s.isdigit():
        return (0, int(s), s)
    return (1, 0, s)


def index_by_key(
    rows: list[dict[str, str]], key_col: str, item_col: str
) -> dict[str, list[dict[str, str]]]:
    grouped: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        cleaned = {k: clean_val(v) for k, v in row.items()}
        chave = cleaned.get(key_col, "").upper()
        if chave:
            grouped[chave].append(cleaned)
    for chave in grouped:
        grouped[chave].sort(key=lambda r: item_sort_key(r.get(item_col, "")))
    return grouped


def write_filtered_csv(
    out_path: Path, headers: list[str], rows: list[dict[str, str]]
) -> None:
    with out_path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=headers, delimiter=";", lineterminator="\n")
        writer.writeheader()
        for r in rows:
            writer.writerow({h: r.get(h, "") for h in headers})


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Compara CSV oficial da Receita Federal contra CSV gerado pelo read-xml -r."
    )
    parser.add_argument("receita_csv", type=Path, help="CSV parâmetro da Receita Federal")
    parser.add_argument("gerado_csv", type=Path, help="CSV gerado pelo read-xml")
    parser.add_argument(
        "-n", "--sample", type=int, default=100, help="Quantidade de chaves na amostra (padrão: 100)"
    )
    parser.add_argument(
        "--all", action="store_true", help="Compara TODAS as chaves em comum (ignora -n)"
    )
    parser.add_argument("--seed", type=int, default=42, help="Seed da amostragem (padrão: 42)")
    parser.add_argument(
        "--first",
        action="store_true",
        help="Seleciona as primeiras N chaves ordenadas em vez de amostra aleatória",
    )
    parser.add_argument(
        "--max-exemplos",
        type=int,
        default=3,
        help="Máximo de exemplos exibidos por coluna divergente (padrão: 3)",
    )
    parser.add_argument(
        "--save-dir",
        type=Path,
        help="Salva ref_amostra.csv e gen_amostra.csv ordenados nesse diretório para diff -u",
    )
    args = parser.parse_args()

    ref_headers, ref_rows, ref_enc = read_csv_auto(args.receita_csv)
    gen_headers, gen_rows, gen_enc = read_csv_auto(args.gerado_csv)

    print("=== Metadados dos Arquivos ===")
    print(f"Receita  : {args.receita_csv} ({ref_enc}, {len(ref_rows)} linhas, {len(ref_headers)} cols)")
    print(f"Gerado   : {args.gerado_csv} ({gen_enc}, {len(gen_rows)} linhas, {len(gen_headers)} cols)")

    if ref_headers != gen_headers:
        print("\n[AVISO] Cabeçalhos diferem!")
        only_ref = [h for h in ref_headers if h not in gen_headers]
        only_gen = [h for h in gen_headers if h not in ref_headers]
        if only_ref:
            print(f"  Só na Receita: {only_ref}")
        if only_gen:
            print(f"  Só no Gerado : {only_gen}")
    else:
        print("Cabeçalhos: 100% idênticos (56 colunas na mesma ordem).")

    ref_key_col = find_col(ref_headers, KEY_COL_EXACT, "Chave da Nota Fiscal Eletrônica")
    gen_key_col = find_col(gen_headers, KEY_COL_EXACT, "Chave da Nota Fiscal Eletrônica")
    ref_item_col = find_col(ref_headers, ITEM_COL_EXACT, "Número do Item")
    gen_item_col = find_col(gen_headers, ITEM_COL_EXACT, "Número do Item")

    ref_by_key = index_by_key(ref_rows, ref_key_col, ref_item_col)
    gen_by_key = index_by_key(gen_rows, gen_key_col, gen_item_col)

    ref_keys = set(ref_by_key)
    gen_keys = set(gen_by_key)
    common_keys = sorted(ref_keys & gen_keys)

    print("\n=== Cobertura de Chaves Únicas ===")
    print(f"Chaves na Receita        : {len(ref_keys)}")
    print(f"Chaves no read-xml       : {len(gen_keys)}")
    print(f"Chaves em comum          : {len(common_keys)}")
    print(f"Só na Receita (ausentes) : {len(ref_keys - gen_keys)}")
    print(f"Só no read-xml (extras)  : {len(gen_keys - ref_keys)}")

    if not common_keys:
        print("\nErro: nenhuma chave em comum encontrada entre os dois arquivos.", file=sys.stderr)
        return 1

    if args.all or args.sample >= len(common_keys):
        selected_keys = common_keys
    elif args.first:
        selected_keys = common_keys[: args.sample]
    else:
        rng = random.Random(args.seed)
        selected_keys = sorted(rng.sample(common_keys, args.sample))

    print(f"\n=== Comparação da Amostra ({len(selected_keys)} chaves ordenadas) ===")

    ref_sample_rows: list[dict[str, str]] = []
    gen_sample_rows: list[dict[str, str]] = []
    row_count_mismatches: list[tuple[str, int, int]] = []
    col_diffs: dict[str, list[tuple[str, str, str, str]]] = defaultdict(list)

    common_headers = [h for h in ref_headers if h in gen_headers]
    matched_docs = 0

    for chave in selected_keys:
        r_list = ref_by_key[chave]
        g_list = gen_by_key[chave]
        ref_sample_rows.extend(r_list)
        gen_sample_rows.extend(g_list)

        if len(r_list) != len(g_list):
            row_count_mismatches.append((chave, len(r_list), len(g_list)))
            continue

        doc_exact = True
        for r_row, g_row in zip(r_list, g_list):
            item_no = r_row.get(ref_item_col, "?")
            for h in common_headers:
                rv = r_row.get(h, "")
                gv = g_row.get(h, "")
                if rv != gv:
                    doc_exact = False
                    col_diffs[h].append((chave, item_no, rv, gv))
        if doc_exact:
            matched_docs += 1

    if args.save_dir:
        args.save_dir.mkdir(parents=True, exist_ok=True)
        ref_out = args.save_dir / "ref_amostra.csv"
        gen_out = args.save_dir / "gen_amostra.csv"
        write_filtered_csv(ref_out, ref_headers, ref_sample_rows)
        write_filtered_csv(gen_out, gen_headers, gen_sample_rows)
        print(f"Amostras salvas em:\n  {ref_out}\n  {gen_out}")
        print(f"Para diff direto: diff -u {ref_out} {gen_out} | head -n 60\n")

    print(f"Documentos 100% idênticos: {matched_docs}/{len(selected_keys)}")
    print(f"Divergências de qtd itens: {len(row_count_mismatches)}")
    for chave, rc, gc in row_count_mismatches[: args.max_exemplos]:
        print(f"  - {chave}: Receita={rc} linhas vs Gerado={gc} linhas")

    if not col_diffs and not row_count_mismatches:
        print("\nRESULTADO: 100% IDÊNTICO em todas as 56 colunas da amostra!")
        return 0

    print(f"\nColunas com divergência ({len(col_diffs)}):")
    for col, diffs in sorted(col_diffs.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        print(f"  * {col}: {len(diffs)} divergência(s)")
        for chave, item_no, rv, gv in diffs[: args.max_exemplos]:
            print(f"      [{chave} | item {item_no}]")
            print(f"        Receita: {rv!r}")
            print(f"        Gerado : {gv!r}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
