#!/usr/bin/env python3
"""
auditar-valor-proporcional.py — Seleciona até K NF-es e K CT-es reais com a maior
variedade de tags fiscais/acessórias (vDesc, vFrete, vSeg, vOutro, vIPI, vICMSST,
vFCPST, vICMSDeson, vII, vIPIDevol, PISST, COFINSST, vICMSMonoReten, ICMS60,
ICMSOutraUF, toma4, det/vItem) e compara lado a lado:
  1. XML original (fonte da verdade + cálculo legal W16-10 / VB01-10);
  2. ContÁgil / Receita Federal;
  3. `read-xml`.
"""

import argparse
from collections import defaultdict
import csv
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

csv.field_size_limit(min(sys.maxsize, 2**31 - 1))

EXCLUDED_KEYS = {
    "12345678901234567890123456789012345678901234",
}

KEY_COL = "Chave da Nota Fiscal Eletrônica : NF Item (Todos)"
ITEM_COL = "Número do Item : NF Item (Todos)"
RE_KEY_QUOTE = re.compile(r"^'([0-9A-Za-z]{44})'$")

TOMA_MAP = {
    "0": "Remetente",
    "1": "Expedidor",
    "2": "Recebedor",
    "3": "Destinatário",
    "4": "Outros",
}


def local(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def find_child(el: ET.Element | None, name: str) -> ET.Element | None:
    if el is None:
        return None
    for ch in el:
        if local(ch.tag) == name:
            return ch
    return None


def find_children(el: ET.Element | None, name: str) -> list[ET.Element]:
    if el is None:
        return []
    return [ch for ch in el if local(ch.tag) == name]


def text_of(el: ET.Element | None, name: str) -> str:
    ch = find_child(el, name)
    return (ch.text or "").strip() if ch is not None and ch.text else ""


def float_of(el: ET.Element | None, name: str) -> float:
    t = text_of(el, name)
    if not t:
        return 0.0
    try:
        return float(t.replace(",", "."))
    except ValueError:
        return 0.0


def decode_xml(raw: bytes) -> str:
    if raw.startswith(b"\xef\xbb\xbf"):
        raw = raw[3:]
    head = raw[:256].decode("ascii", errors="ignore")
    m = re.search(r'encoding=["\']([^"\']+)["\']', head, flags=re.I)
    enc = m.group(1) if m else "utf-8"
    try:
        return raw.decode(enc)
    except (LookupError, UnicodeDecodeError):
        return raw.decode("cp1252", errors="replace")


def decode_csv_text(path: Path) -> tuple[str, str]:
    raw = path.read_bytes()
    try:
        text = raw.decode("utf-8-sig")
    except UnicodeDecodeError:
        text = raw.decode("cp1252", errors="replace")
    first = text.splitlines()[0] if text else ""
    delim = ";" if first.count(";") >= first.count(",") else ","
    return text, delim


def load_keys_from_csv(path: Path) -> set[str]:
    text, delim = decode_csv_text(path)
    keys: set[str] = set()
    for row in csv.DictReader(text.splitlines(), delimiter=delim):
        rk = (row.get(KEY_COL) or "").strip()
        m = RE_KEY_QUOTE.match(rk)
        k = (m.group(1) if m else rk).upper()
        if len(k) == 44:
            keys.add(k)
    return keys


def load_csv_by_keys(
    path: Path, target_keys: set[str]
) -> dict[str, dict[str, dict[str, str]]]:
    text, delim = decode_csv_text(path)
    out: dict[str, dict[str, dict[str, str]]] = defaultdict(dict)
    for row in csv.DictReader(text.splitlines(), delimiter=delim):
        rk = (row.get(KEY_COL) or "").strip()
        m = RE_KEY_QUOTE.match(rk)
        k = (m.group(1) if m else rk).upper()
        if k in target_keys:
            it = (row.get(ITEM_COL) or "").strip()
            out[k][it] = {c: (v or "").strip() for c, v in row.items()}
    return out


def br(val: float, decimals: int = 2) -> str:
    s = f"{val:,.{decimals}f}"
    return s.replace(",", "_").replace(".", ",").replace("_", ".")


def parse_nfe_xml(root: ET.Element) -> dict | None:
    nfe = root if local(root.tag) == "NFe" else find_child(root, "NFe")
    inf = find_child(nfe, "infNFe")
    if inf is None:
        return None

    prot = find_child(root, "protNFe")
    inf_prot = find_child(prot, "infProt")
    chave = (
        text_of(inf_prot, "chNFe") or inf.attrib.get("Id", "").removeprefix("NFe")
    ).strip().upper()
    if len(chave) != 44:
        return None

    emit = find_child(inf, "emit")
    emit_cnpj = text_of(emit, "CNPJ") or text_of(emit, "CPF")
    emit_nome = text_of(emit, "xNome")
    total = find_child(inf, "total")
    icms_tot = find_child(total, "ICMSTot")
    v_nf = float_of(icms_tot, "vNF")

    dets = find_children(inf, "det")
    if len(dets) <= 1:
        return None

    tags_present: set[str] = set()
    items = []
    for det in dets:
        n_item = det.attrib.get("nItem", "").strip()
        prod = find_child(det, "prod")
        imp = find_child(det, "imposto")
        icms = find_child(imp, "ICMS")
        icms_sub = next(iter(icms), None) if icms is not None else None
        ipi = find_child(imp, "IPI")
        ipi_trib = find_child(ipi, "IPITrib")
        ii = find_child(imp, "II")
        pisst = find_child(imp, "PISST")
        cofinsst = find_child(imp, "COFINSST")
        imp_devol = find_child(det, "impostoDevol")
        ipi_devol = find_child(imp_devol, "IPI")

        v_prod = float_of(prod, "vProd")
        v_desc = float_of(prod, "vDesc")
        v_frete = float_of(prod, "vFrete")
        v_seg = float_of(prod, "vSeg")
        v_outro = float_of(prod, "vOutro")
        v_ipi = float_of(ipi_trib, "vIPI")
        v_ii = float_of(ii, "vII")
        v_ipi_devol = float_of(ipi_devol, "vIPIDevol")

        v_icms = float_of(icms_sub, "vICMS")
        v_icms_st = float_of(icms_sub, "vICMSST")
        v_fcp_st = float_of(icms_sub, "vFCPST")
        v_icms_mono_reten = float_of(icms_sub, "vICMSMonoReten")
        v_icms_mono_ret = float_of(icms_sub, "vICMSMonoRet")
        v_icms_deson_raw = float_of(icms_sub, "vICMSDeson")
        v_icms_deson_ded = (
            v_icms_deson_raw if text_of(icms_sub, "indDeduzDeson") == "1" else 0.0
        )

        v_pis_st = (
            float_of(pisst, "vPIS") if text_of(pisst, "indSomaPISST") == "1" else 0.0
        )
        v_cofins_st = (
            float_of(cofinsst, "vCOFINS")
            if text_of(cofinsst, "indSomaCOFINSST") == "1"
            else 0.0
        )
        v_item_xml = text_of(det, "vItem")

        for label, val in [
            ("vDesc", v_desc),
            ("vFrete", v_frete),
            ("vSeg", v_seg),
            ("vOutro", v_outro),
            ("vIPI", v_ipi),
            ("vICMSST", v_icms_st),
            ("vFCPST", v_fcp_st),
            ("vICMSDeson(deduz=1)", v_icms_deson_ded),
            ("vII", v_ii),
            ("vIPIDevol", v_ipi_devol),
            ("vICMSMonoReten", v_icms_mono_reten),
            ("vICMSMonoRet(ICMS61)", v_icms_mono_ret),
            ("PISST(indSoma=1)", v_pis_st),
            ("COFINSST(indSoma=1)", v_cofins_st),
        ]:
            if val > 0.0:
                tags_present.add(label)
        if v_item_xml:
            tags_present.add("det/vItem")

        v_vb01 = round(
            v_prod
            - v_desc
            - v_icms_deson_ded
            + v_icms_st
            + v_icms_mono_reten
            + v_fcp_st
            + v_frete
            + v_seg
            + v_outro
            + v_ii
            + v_ipi
            + v_ipi_devol
            + v_pis_st
            + v_cofins_st,
            2,
        )
        v_legal = float(v_item_xml.replace(",", ".")) if v_item_xml else v_vb01

        items.append(
            {
                "nItem": n_item,
                "vProd": v_prod,
                "vDesc": v_desc,
                "vFrete": v_frete,
                "vSeg": v_seg,
                "vOutro": v_outro,
                "vIPI": v_ipi,
                "vICMS": v_icms,
                "vICMSST": v_icms_st + v_icms_mono_reten,
                "vLegal": v_legal,
            }
        )

    if len(tags_present) < 2:
        return None

    return {
        "chave": chave,
        "emit_cnpj": emit_cnpj,
        "emit_nome": emit_nome,
        "vNF": v_nf,
        "tags": sorted(tags_present),
        "items": items,
    }


def parse_cte_xml(root: ET.Element) -> dict | None:
    root_name = local(root.tag)
    cte_el = None
    for tag_name in ("CTe", "GTVe", "CTeSimp", "CTeOS"):
        cte_el = root if root_name == tag_name else find_child(root, tag_name)
        if cte_el is not None:
            break
    inf = find_child(cte_el, "infCte")
    if inf is None:
        return None

    prot = find_child(root, "protCTe")
    inf_prot = find_child(prot, "infProt")
    chave = (
        text_of(inf_prot, "chCTe") or inf.attrib.get("Id", "").removeprefix("CTe")
    ).strip().upper()
    if len(chave) != 44:
        return None

    ide = find_child(inf, "ide")
    emit = find_child(inf, "emit")
    emit_cnpj = text_of(emit, "CNPJ") or text_of(emit, "CPF")
    emit_nome = text_of(emit, "xNome")
    rem = find_child(inf, "rem")
    dest = find_child(inf, "dest")
    compl = find_child(inf, "compl")
    v_prest = find_child(inf, "vPrest")
    total_el = find_child(inf, "total")
    v_tprest = float_of(v_prest, "vTPrest") or float_of(total_el, "vTPrest")

    imp = find_child(inf, "imp")
    icms = find_child(imp, "ICMS")
    icms_sub = next(iter(icms), None) if icms is not None else None
    icms_tag = local(icms_sub.tag) if icms_sub is not None else ""

    v_bc = float_of(icms_sub, "vBC") or float_of(icms_sub, "vBCOutraUF")
    p_icms = (float_of(icms_sub, "pICMS") or float_of(icms_sub, "pICMSOutraUF")) / 100.0
    v_icms = float_of(icms_sub, "vICMS") or float_of(icms_sub, "vICMSOutraUF")
    v_icms_st = float_of(icms_sub, "vICMSSTRet")
    v_cred = float_of(icms_sub, "vCred")

    toma_code = text_of(ide, "toma")
    toma_el = None
    for tname in ("toma3", "toma03", "toma4", "toma04", "tomaTerceiro"):
        toma_el = find_child(ide, tname)
        if toma_el is not None:
            toma_code = text_of(toma_el, "toma") or toma_code
            break
    outro_toma = ""
    if toma_code == "4" and toma_el is not None:
        outro_toma = text_of(toma_el, "CNPJ") or text_of(toma_el, "CPF")

    inf_norm = find_child(inf, "infCTeNorm")
    inf_doc = find_child(inf_norm, "infDoc")
    nfe_count = len(find_children(inf_doc, "infNFe"))
    x_obs = text_of(compl, "xObs")

    tags_present: set[str] = set()
    if icms_tag:
        tags_present.add(icms_tag)
    if v_icms > 0:
        tags_present.add("vICMS>0")
    if v_icms_st > 0:
        tags_present.add("ICMS60/vICMSSTRet")
    if v_cred > 0:
        tags_present.add("vCred")
    if toma_code == "4":
        tags_present.add("toma4(Outros)")
    if nfe_count == 0:
        tags_present.add("sem_infNFe")
    elif nfe_count > 1:
        tags_present.add(f"multi_NFe({nfe_count})")
    if x_obs:
        tags_present.add("xObs")
    if rem is not None and text_of(rem, "xNome"):
        tags_present.add("rem")
    if dest is not None and text_of(dest, "xNome"):
        tags_present.add("dest")

    # Prioriza CT-es com tags mais ricas/interessantes
    richness = (
        (4 if v_icms_st > 0 else 0)
        + (3 if icms_tag == "ICMSOutraUF" else 0)
        + (3 if toma_code == "4" else 0)
        + (2 if v_icms > 0 else 0)
        + (2 if nfe_count == 0 else 1)
        + (1 if x_obs else 0)
    )

    return {
        "chave": chave,
        "emit_cnpj": emit_cnpj,
        "emit_nome": emit_nome,
        "vTPrest": v_tprest,
        "icms_tag": icms_tag or "<N/D>",
        "vBC": v_bc,
        "pICMS": p_icms,
        "vICMS": v_icms,
        "vICMSST": v_icms_st,
        "papel": TOMA_MAP.get(toma_code, "<N/D>"),
        "outro_toma": outro_toma or "<N/D>",
        "rem_nome": text_of(rem, "xNome") or "<N/D>",
        "dest_nome": text_of(dest, "xNome") or "<N/D>",
        "uf_ini": text_of(ide, "UFIni") or "<N/D>",
        "uf_fim": text_of(ide, "UFFim") or "<N/D>",
        "nfe_count": nfe_count,
        "richness": richness,
        "tags": sorted(tags_present),
    }


def pick_diverse(candidates: list[dict], k: int, sort_key) -> list[dict]:
    candidates.sort(key=sort_key)
    selected: list[dict] = []
    seen_emits: set[str] = set()
    for doc in candidates:
        if doc["emit_cnpj"] not in seen_emits:
            selected.append(doc)
            seen_emits.add(doc["emit_cnpj"])
            if len(selected) == k:
                return selected
    for doc in candidates:
        if len(selected) >= k:
            break
        if doc not in selected:
            selected.append(doc)
    return selected


def audit_nfes(
    zip_files: list[Path],
    ref_csv: Path,
    gen_csv: Path,
    k: int,
    max_scan: int,
) -> None:
    print("\n" + "#" * 100)
    print(f"# AUDITORIA NF-e: {ref_csv.name} vs {gen_csv.name}")
    print("#" * 100)

    ref_keys = load_keys_from_csv(ref_csv)
    gen_keys = load_keys_from_csv(gen_csv)
    valid_keys = ref_keys & gen_keys

    candidates: list[dict] = []
    seen: set[str] = set()
    scanned = 0

    for zp in zip_files:
        with zipfile.ZipFile(zp) as zf:
            for info in zf.infolist():
                if info.is_dir() or not info.filename.lower().endswith(".xml"):
                    continue
                low = info.filename.lower()
                if "evento" in low:
                    continue
                scanned += 1
                if max_scan > 0 and scanned > max_scan and len(candidates) >= k * 10:
                    break
                try:
                    root = ET.fromstring(decode_xml(zf.read(info)))
                except ET.ParseError:
                    continue
                if local(root.tag) not in {"nfeProc", "NFe"}:
                    continue
                doc = parse_nfe_xml(root)
                if (
                    doc is None
                    or doc["chave"] in EXCLUDED_KEYS
                    or doc["chave"] in seen
                    or doc["chave"] not in valid_keys
                ):
                    continue
                sum_legal = round(sum(it["vLegal"] for it in doc["items"]), 2)
                sum_prod = round(sum(it["vProd"] for it in doc["items"]), 2)
                if abs(sum_legal - doc["vNF"]) <= 0.02 and abs(sum_legal - sum_prod) > 0.05:
                    seen.add(doc["chave"])
                    candidates.append(doc)
            if max_scan > 0 and scanned > max_scan and len(candidates) >= k * 10:
                break

    selected = pick_diverse(
        candidates, k, lambda d: (-len(d["tags"]), -len(d["items"]), d["chave"])
    )
    if not selected:
        print("Nenhuma NF-e elegível encontrada.")
        return

    target_keys = {d["chave"] for d in selected}
    ref_map = load_csv_by_keys(ref_csv, target_keys)
    gen_map = load_csv_by_keys(gen_csv, target_keys)
    col_prop = "Valor da Nota Proporcional : NF Item (Todos) SOMA"
    col_tot = "Valor Total : NF (Todos) SOMA"

    for idx, doc in enumerate(selected, 1):
        ch = doc["chave"]
        print("=" * 100)
        print(f"[NFe {idx}] Chave: {ch}")
        print(f"    Emitente: {doc['emit_cnpj']} - {doc['emit_nome']}")
        print(f"    vNF XML : R$ {br(doc['vNF'])} | Tags ativas: {', '.join(doc['tags'])}")
        print(
            f"    {'Item':>4} | {'vProd':>10} {'vDesc':>8} {'vFrete':>8} {'vSeg':>7} {'vOutro':>8} "
            f"{'vIPI':>8} {'vST':>8} | {'XML(VB01)':>10} | {'ContÁgil':>10} | {'read-xml':>10}"
        )
        print("-" * 100)
        sum_xml = 0.0
        for it in doc["items"]:
            n = it["nItem"]
            sum_xml += it["vLegal"]
            r_prop = ref_map.get(ch, {}).get(n, {}).get(col_prop, "?")
            g_prop = gen_map.get(ch, {}).get(n, {}).get(col_prop, "?")
            mark = "OK" if br(it["vLegal"]) == g_prop else "DIFF"
            print(
                f"    {n:>4} | {br(it['vProd']):>10} {br(it['vDesc']):>8} {br(it['vFrete']):>8} "
                f"{br(it['vSeg']):>7} {br(it['vOutro']):>8} {br(it['vIPI']):>8} {br(it['vICMSST']):>8} "
                f"| {br(it['vLegal']):>10} | {r_prop:>10} | {g_prop:>10}  [{mark}]"
            )
        r0 = ref_map.get(ch, {}).get("0", {})
        g0 = gen_map.get(ch, {}).get("0", {})
        print("-" * 100)
        print(
            f"    Soma XML VB01-10: {br(sum_xml)} (vNF XML: {br(doc['vNF'])}) | "
            f"Item 0 ContÁgil: {r0.get(col_tot, '?')} | read-xml: {g0.get(col_tot, '?')}"
        )


def audit_ctes(
    zip_files: list[Path],
    ref_csv: Path,
    gen_csv: Path,
    k: int,
    max_scan: int,
) -> None:
    print("\n" + "#" * 100)
    print(f"# AUDITORIA CT-e: {ref_csv.name} vs {gen_csv.name}")
    print("#" * 100)

    ref_keys = load_keys_from_csv(ref_csv)
    gen_keys = load_keys_from_csv(gen_csv)
    valid_keys = ref_keys & gen_keys

    candidates: list[dict] = []
    seen: set[str] = set()
    scanned = 0

    for zp in zip_files:
        with zipfile.ZipFile(zp) as zf:
            for info in zf.infolist():
                if info.is_dir() or not info.filename.lower().endswith(".xml"):
                    continue
                low = info.filename.lower()
                if "evento" in low:
                    continue
                scanned += 1
                if max_scan > 0 and scanned > max_scan and len(candidates) >= k * 20:
                    break
                try:
                    root = ET.fromstring(decode_xml(zf.read(info)))
                except ET.ParseError:
                    continue
                if local(root.tag) not in {
                    "cteProc",
                    "CTe",
                    "GTVeProc",
                    "GTVe",
                    "cteSimpProc",
                    "CTeSimp",
                    "cteOSProc",
                    "CTeOS",
                }:
                    continue
                doc = parse_cte_xml(root)
                if doc is None or doc["chave"] in seen or doc["chave"] not in valid_keys:
                    continue
                seen.add(doc["chave"])
                candidates.append(doc)
            if max_scan > 0 and scanned > max_scan and len(candidates) >= k * 20:
                break

    selected = pick_diverse(
        candidates, k, lambda d: (-d["richness"], -len(d["tags"]), d["chave"])
    )
    if not selected:
        print("Nenhum CT-e elegível encontrado.")
        return

    target_keys = {d["chave"] for d in selected}
    ref_map = load_csv_by_keys(ref_csv, target_keys)
    gen_map = load_csv_by_keys(gen_csv, target_keys)

    cols = [
        ("Valor Total", "Valor Total : NF (Todos) SOMA", lambda d: br(d["vTPrest"])),
        (
            "Valor Prop.",
            "Valor da Nota Proporcional : NF Item (Todos) SOMA",
            lambda d: br(d["vTPrest"]),
        ),
        ("ICMS BC", "ICMS: Base de Cálculo : NF Item (Todos) SOMA", lambda d: br(d["vBC"])),
        (
            "ICMS Alíq.",
            "ICMS: Alíquota : NF Item (Todos) NOISE OR",
            lambda d: br(d["pICMS"], 4),
        ),
        ("ICMS Valor", "ICMS: Valor do Tributo : NF Item (Todos) SOMA", lambda d: br(d["vICMS"])),
        (
            "ICMS ST",
            "ICMS por Substituição: Valor do Tributo : NF Item (Todos) SOMA",
            lambda d: br(d["vICMSST"]),
        ),
        (
            "Papel Tom.",
            "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoInformacaoNFe",
            lambda d: d["papel"],
        ),
        (
            "Outro Tom.",
            "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe",
            lambda d: d["outro_toma"],
        ),
        ("UF Início", "CTe - UF do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes", lambda d: d["uf_ini"]),
        ("UF Término", "CTe - UF do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes", lambda d: d["uf_fim"]),
    ]

    for idx, doc in enumerate(selected, 1):
        ch = doc["chave"]
        r = ref_map.get(ch, {}).get("1", {})
        g = gen_map.get(ch, {}).get("1", {})
        print("=" * 100)
        print(f"[CTe {idx}] Chave: {ch}")
        print(f"    Emitente : {doc['emit_cnpj']} - {doc['emit_nome']}")
        print(f"    Remetente: {doc['rem_nome']} | Destinatário: {doc['dest_nome']}")
        print(f"    Grupo ICMS XML: {doc['icms_tag']} | Tags: {', '.join(doc['tags'])}")
        print(f"    {'Campo':<12} | {'XML':>14} | {'ContÁgil':>14} | {'read-xml':>14} | Status")
        print("-" * 100)
        for label, col, xml_fn in cols:
            xv = xml_fn(doc)
            rv = r.get(col, "?")
            gv = g.get(col, "?")
            ok_xml = "OK" if xv == gv else "DIFF-XML"
            print(f"    {label:<12} | {xv:>14} | {rv:>14} | {gv:>14} | {ok_xml}")
        obs_r = r.get("CTe - Observações Gerais de Conhecimento : ConhecimentoInformacaoNFe", "?")
        obs_g = g.get("CTe - Observações Gerais de Conhecimento : ConhecimentoInformacaoNFe", "?")
        print(f"    Obs ContÁgil: {obs_r[:90]}")
        print(f"    Obs read-xml: {obs_g[:90]}")
    print("=" * 100)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audita K NF-es e K CT-es reais: XML original vs ContÁgil vs read-xml."
    )
    parser.add_argument(
        "--zip-dir", type=Path, default=Path("."), help="Diretório raiz dos .zip (recursivo)"
    )
    parser.add_argument("--nfe-receita", type=Path, help="CSV ContÁgil de NF-e")
    parser.add_argument("--nfe-gerado", type=Path, help="CSV read-xml de NF-e")
    parser.add_argument("--cte-receita", type=Path, help="CSV ContÁgil de CT-e")
    parser.add_argument("--cte-gerado", type=Path, help="CSV read-xml de CT-e")
    parser.add_argument("-k", "--qtd", type=int, default=5, help="Chaves por tipo (padrão: 5)")
    parser.add_argument(
        "--max-scan",
        type=int,
        default=50000,
        help="Máximo de XMLs varridos por tipo quando já houver candidatos (0 = tudo)",
    )
    args = parser.parse_args()

    zip_files = sorted(args.zip_dir.rglob("*.zip"))
    if not zip_files:
        print(f"Erro: nenhum .zip encontrado em {args.zip_dir}", file=sys.stderr)
        return 1

    ran = False
    if args.nfe_receita and args.nfe_gerado:
        audit_nfes(zip_files, args.nfe_receita, args.nfe_gerado, args.qtd, args.max_scan)
        ran = True
    if args.cte_receita and args.cte_gerado:
        audit_ctes(zip_files, args.cte_receita, args.cte_gerado, args.qtd, args.max_scan)
        ran = True

    if not ran:
        print(
            "Informe --nfe-receita/--nfe-gerado e/ou --cte-receita/--cte-gerado.",
            file=sys.stderr,
        )
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
