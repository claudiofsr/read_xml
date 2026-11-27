#!/usr/bin/env python3
r"""
verificar-chave.py — XML (fonte da verdade) vs ContÁgil vs read-xml.

CT-e: audita as 56 colunas do layout Receita. NF-e: campos por item (VB01-10).
Classes: xml (literal) | deriv (regra/tabela oficial) | nome/mun (normalização)
         escala (percentual vs fração) | info (eventos/grafo: não contabiliza)

Uso:
  verificar-chave.py -n 10 --seed 1973 --receita R.csv --gerado G.csv
  verificar-chave.py CHAVE1 CHAVE2 --receita R.csv --gerado G.csv

# Reauditar com Descrição CFOP agora verificada (57ª checagem efetiva)
cd ~/Documents/Derat_SPO/Magazine\ Luiza\ SA/Info\ da\ Receita

~/Documents/Rust/projects/read_xml/scripts/verificar-chave.py --receita DadosAdicionais-CTe.csv --gerado documentos_fiscais-ctes.csv -n 30 --seed 1973 --so-divergentes
~/Documents/Rust/projects/read_xml/scripts/verificar-chave.py --receita DadosAdicionais-NFe.csv --gerado documentos_fiscais-nfes.csv -n 30 --seed 1973 --so-divergentes

~/Documents/Rust/projects/read_xml/scripts/verificar-chave.py --receita DadosAdicionais-CTe.csv --gerado documentos_fiscais-ctes.csv --zip-dir . -n 10 --seed 1973

"""

import argparse, csv, random, re, sys, zipfile
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path

csv.field_size_limit(min(sys.maxsize, 2**31 - 1))
KEY = "Chave da Nota Fiscal Eletrônica : NF Item (Todos)"
ITEM = "Número do Item : NF Item (Todos)"
COMP = " : ConhecimentoValoresPrestacaoServico-Componentes"
INFO = " : ConhecimentoInformacaoNFe"
ND = "<N/D>"
RE44 = re.compile(r"([0-9A-Za-z]{44})")
PAPEL = {"0": "Remetente", "1": "Expedidor", "2": "Recebedor", "3": "Destinatário", "4": "Outros"}
MODELO = {"CTe": "Conhecimento de Transporte Eletrônico CT-e",
          "GTVe": "Guia de Transporte de Valores Eletrônica GTV-e",
          "CTeSimp": "Conhecimento de Transporte Eletrônico Simplificado CT-e",
          "CTeOS": "Conhecimento de Transporte Eletrônico para Outros Serviços CT-e OS"}


def local(t): return t.rsplit("}", 1)[-1]
def child(el, n):
    if el is None: return None
    for c in el:
        if local(c.tag) == n: return c
    return None
def any_child(el, names):
    for n in names:
        c = child(el, n)
        if c is not None: return c
    return None
def kids(el, n): return [] if el is None else [c for c in el if local(c.tag) == n]
def text(el, n):
    c = child(el, n)
    return (c.text or "").strip() if c is not None and c.text else ""
def num(el, n):
    try: return float(text(el, n).replace(",", "."))
    except ValueError: return 0.0
def ws(s): return " ".join((s or "").split())
def to_num(s):
    s = (s or "").strip()
    if not s or s in (ND, "*DIVERSOS*"): return None
    if "," in s: s = s.replace(".", "").replace(",", ".")
    try: return float(s)
    except ValueError: return None
def cnpj(v):
    d = re.sub(r"\W", "", v or "").upper()
    return f"{d[:2]}.{d[2:5]}.{d[5:8]}/{d[8:12]}-{d[12:14]}" if len(d) == 14 else (v or ND)
def cpf(v):
    d = re.sub(r"\D", "", v or "")
    return f"{d[:3]}.{d[3:6]}.{d[6:9]}-{d[9:11]}" if len(d) == 11 else (v or ND)
def doc_of(el):
    c, f = text(el, "CNPJ"), text(el, "CPF")
    return cnpj(c) if c else (cpf(f) if f else ND)
def name_of(el): return ws(text(el, "xNome")) or ND
def first(*vals):
    for v in vals:
        if v and v != ND: return v
    return ND

def decode(raw):
    if raw.startswith(b"\xef\xbb\xbf"): raw = raw[3:]
    m = re.search(r'encoding=["\']([^"\']+)["\']', raw[:256].decode("ascii", "ignore"), re.I)
    try: return raw.decode(m.group(1) if m else "utf-8")
    except (LookupError, UnicodeDecodeError): return raw.decode("cp1252", "replace")

def own_key(root):
    for p, ch in (("protCTe", "chCTe"), ("protNFe", "chNFe")):
        k = text(child(child(root, p), "infProt"), ch)
        if k: return k.strip().upper()
    for d, i, p in (("CTe","infCte","CTe"), ("NFe","infNFe","NFe"), ("GTVe","infCte","CTe"),
                    ("CTeSimp","infCte","CTe"), ("CTeOS","infCte","CTe")):
        el = root if local(root.tag) == d else child(root, d)
        if el is not None:
            a = (child(el, i).attrib.get("Id", "") if child(el, i) is not None else "")
            if a.upper().startswith(p): return a[len(p):].upper()
    return ""

def build_index(root_dir):
    idx = {}
    for p in Path(root_dir).rglob("*.xml"):
        if "evento" in p.name.lower(): continue
        m = RE44.search(p.name)
        if m: idx.setdefault(m.group(1).upper(), (str(p), None, str(p)))
    for zp in Path(root_dir).rglob("*.zip"):
        try:
            with zipfile.ZipFile(zp) as z:
                for i in z.infolist():
                    if i.is_dir() or not i.filename.lower().endswith(".xml"): continue
                    if "evento" in i.filename.lower(): continue
                    m = RE44.search(i.filename)
                    if m: idx.setdefault(m.group(1).upper(), (f"{zp}::{i.filename}", zp, i.filename))
        except zipfile.BadZipFile: continue
    return idx

def read_entry(e):
    _, zp, n = e
    if zp is None: return Path(n).read_bytes()
    with zipfile.ZipFile(zp) as z: return z.read(n)

def load_csv(path):
    raw = Path(path).read_bytes()
    try: txt = raw.decode("utf-8-sig")
    except UnicodeDecodeError: txt = raw.decode("cp1252", "replace")
    lines = txt.splitlines()
    d = ";" if lines[0].count(";") >= lines[0].count(",") else ","
    out = {}
    for r in csv.DictReader(lines, delimiter=d):
        k = (r.get(KEY) or "").strip().strip("'").upper()
        if len(k) == 44: out.setdefault(k, {})[(r.get(ITEM) or "").strip()] = r
    return out

def is_nfe(k): return k[20:22] in ("55", "65")


# ---------------------------------------------------------------- CT-e: 56 colunas
def load_cfop_table(root: Path) -> dict:
    """Tabela oficial CONFAZ já vigente no projeto (fonte única: src/export/cfop.rs)."""
    p = root / "src" / "export" / "cfop.rs"
    if not p.exists():
        return {}
    txt = p.read_text(encoding="utf-8")
    out = {}
    pat = re.compile(r'\(\s*(\d{4})\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)', re.DOTALL)
    for m in pat.finditer(txt):
        out[m.group(1)] = re.sub(r"\\(.)", r"\1", m.group(2))
    return out


CFOP_TABLE = load_cfop_table(Path(__file__).resolve().parent.parent)
CFOP_TABLE_OK = len(CFOP_TABLE) >= 100


def cte_fields(root):
    tag = local(root.tag)
    cte = root if tag in MODELO else any_child(root, MODELO)
    tipo = local(cte.tag)
    inf = child(cte, "infCte")
    ide, emit = child(inf, "ide"), child(inf, "emit")
    rem, dest = child(inf, "rem"), child(inf, "dest")
    compl, imp = child(inf, "compl"), child(inf, "imp")
    end_rem = child(rem, "enderReme")
    icms = child(imp, "ICMS")
    sub = next(iter(icms), None) if icms is not None else None
    pis = any_child(child(imp, "PIS"), ("PISAliq", "PISQtde", "PISOutr", "PISNT"))
    cof = any_child(child(imp, "COFINS"), ("COFINSAliq", "COFINSQtde", "COFINSOutr", "COFINSNT"))
    iss, ipi = child(imp, "ISSQN"), child(child(imp, "IPI"), "IPITrib")
    vp = child(inf, "vPrest")
    vtot = num(vp, "vTPrest") or num(child(inf, "total"), "vTPrest")
    toma_el = any_child(ide, ("toma4", "toma04", "tomaTerceiro", "toma3", "toma03"))
    cod = text(toma_el, "toma") or text(ide, "toma")
    if tipo == "GTVe" and cod == "1" and toma_el is None: cod = "3"
    toma4 = any_child(ide, ("toma4", "toma04", "tomaTerceiro"))
    if toma4 is None:
        toma4 = child(inf, "toma")
    papel = PAPEL.get(cod, ND)
    outro = doc_of(toma4) if cod == "4" else ND
    agente = {"0": rem, "1": child(inf, "exped"), "2": child(inf, "receb"), "3": dest}.get(cod, toma4)
    bc = num(sub, "vBC") or num(sub, "vBCOutraUF")
    aliq = (num(sub, "pICMS") or num(sub, "pICMSOutraUF")) / 100.0
    vic = num(sub, "vICMS") or num(sub, "vICMSOutraUF") or num(sub, "vICMSMono")
    vst = num(sub, "vICMSSTRet") or num(sub, "vICMSST") or num(sub, "vICMSMonoReten")
    cfop = text(ide, "CFOP")
    entrada = "Entrada" if cfop[:1] in ("1", "2", "3") else "Saída"
    dh = text(ide, "dhEmi") or text(ide, "dEmi")
    data = f"{dh[8:10]}/{dh[5:7]}/{dh[0:4]}" if len(dh) >= 10 else ND
    loc = child(dest, "locEnt")
    if loc is None:
        loc = child(compl, "Entrega")
    nfe_keys = [text(n, "chave") or text(n, "chNFe")
                for n in kids(child(child(inf, "infCTeNorm"), "infDoc"), "infNFe")]
    nfe_keys = [k for k in nfe_keys if k]
    divers = ND if not nfe_keys else (nfe_keys[0] if len(nfe_keys) == 1 else "*DIVERSOS*")
    prot = text(child(child(root, "protCTe"), "infProt"), "cStat")
    F = [
        ("CNPJ Contrib", "CNPJ do Contribuinte : NF Item (Todos)", doc_of(emit), "xml"),
        ("Nome Contrib", "Nome do Contribuinte : NF Item (Todos)", name_of(emit), "nome"),
        ("Entrada/Saída", "Entrada/Saída : NF (Todos)", entrada, "deriv"),
        ("CNPJ Particip", "CPF/CNPJ do Participante : NF (Todos)", doc_of(agente), "xml"),
        ("Nome Particip", "Nome do Participante : NF (Todos)", name_of(agente), "nome"),
        ("CRT", "CRT : NF (Todos)", text(emit, "CRT") or ND, "xml"),
        ("Observações NF", "Observações : NF (Todos)", ND, "deriv"),
        ("Rem CNPJ comp", "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento" + COMP, doc_of(rem), "xml"),
        ("Rem CNPJ nfe", "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento" + INFO, doc_of(rem), "xml"),
        ("Rem Nome", "CTe - Remetente das mercadorias transportadas: Nome de Conhecimento" + INFO, name_of(rem), "nome"),
        ("Rem Município", "CTe - Remetente das mercadorias transportadas: Município de Conhecimento" + INFO, ws(text(end_rem, "xMun")) or ND, "mun"),
        ("Papel comp", "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento" + COMP, papel, "xml"),
        ("Papel nfe", "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento" + INFO, papel, "xml"),
        ("Outro Tom comp", "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento" + COMP, outro, "xml"),
        ("Outro Tom nfe", "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento" + INFO, outro, "xml"),
        ("UF início", "CTe - UF do início da prestação de Conhecimento" + COMP, first(text(ide, "UFIni"), text(child(child(inf, "exped"), "enderExped"), "UF"), text(end_rem, "UF")), "deriv"),
        ("Mun início", "CTe - Nome do Município do início da prestação de Conhecimento" + COMP, first(ws(text(ide, "xMunIni")), ws(text(child(child(inf, "exped"), "enderExped"), "xMun")), ws(text(end_rem, "xMun"))), "mun"),
        ("UF término", "CTe - UF do término da prestação de Conhecimento" + COMP, first(text(ide, "UFFim"), text(child(child(inf, "receb"), "enderReceb"), "UF"), text(child(dest, "enderDest"), "UF")), "deriv"),
        ("Mun término", "CTe - Nome do Município do término da prestação de Conhecimento" + COMP, first(ws(text(ide, "xMunFim")), ws(text(child(child(inf, "receb"), "enderReceb"), "xMun")), ws(text(child(dest, "enderDest"), "xMun"))), "mun"),
        ("Dest CNPJ", "CTe - Informações do Destinatário do CT-e: CNPJ/CPF de Conhecimento" + COMP, doc_of(dest), "xml"),
        ("Dest Nome", "CTe - Informações do Destinatário do CT-e: Nome de Conhecimento" + COMP, name_of(dest), "nome"),
        ("Local Entrega", "CTe - Local de Entrega constante na Nota Fiscal: Nome de Conhecimento" + COMP, name_of(loc), "xml"),
        ("Natureza Op", "Descrição da Natureza da Operação : NF Item (Todos)", ws(text(ide, "natOp")) or ND, "xml"),
        ("Cancelada", "Cancelada : NF (Todos)", "Sim" if prot in ("101", "151", "155") else "Não", "info"),
        ("Origem", "Registro de Origem do Item : NF Item (Todos)", tipo, "deriv"),
        ("Nat BC Créd", "Natureza da Base de Cálculo do Crédito Descrição : NF Item (Todos)", ND, "deriv"),
        ("Modelo", "Modelo - Descrição : NF Item (Todos)", MODELO.get(tipo, ND), "deriv"),
        ("Número Nota", "Número da Nota : NF Item (Todos)", text(ide, "nCT") or ND, "xml"),
        ("Chave", KEY, own_key(root), "xml"),
        ("Inf NFe chave", "Inf. NFe - Chave de acesso da NF-e" + INFO, divers, "info"),
        ("xObs", "CTe - Observações Gerais de Conhecimento" + INFO, ws(text(compl, "xObs")) or ND, "xml"),
        ("Dia Emissão", "Dia da Emissão : NF Item (Todos)", data, "xml"),
        ("Número DI", "Número da DI : NF Item (Todos)", ND, "deriv"),
        ("Número Item", ITEM, "1", "deriv"),
        ("CFOP", "Código CFOP : NF Item (Todos)", cfop or ND, "xml"),
        ("Desc CFOP", "Descrição CFOP : NF Item (Todos)", CFOP_TABLE.get(cfop, ND), "tabela"),
        ("Desc Mercad", "Descrição da Mercadoria/Serviço : NF Item (Todos)", ND, "deriv"),
        ("NCM", "Código NCM : NF Item (Todos)", "00000000", "deriv"),
        ("Desc NCM", "Descrição NCM : NF Item (Todos)", ND, "deriv"),
        ("COFINS Alíq", "COFINS: Alíquota ad valorem - Atributo : NF Item (Todos)", num(cof, "pCOFINS"), "escala"),
        ("PIS Alíq", "PIS: Alíquota ad valorem - Atributo : NF Item (Todos)", num(pis, "pPIS"), "escala"),
        ("CST COFINS", "CST COFINS Descrição : NF Item (Todos)", ND, "deriv"),
        ("CST PIS", "CST PIS Descrição : NF Item (Todos)", ND, "deriv"),
        ("Valor Total", "Valor Total : NF (Todos) SOMA", vtot, "num"),
        ("Valor Prop", "Valor da Nota Proporcional : NF Item (Todos) SOMA", vtot, "num"),
        ("Descontos", "Valor dos Descontos : NF Item (Todos) SOMA", 0.0, "num"),
        ("Seguro", "Valor Seguro : NF (Todos) SOMA", 0.0, "num"),
        ("COFINS Valor", "COFINS: Valor do Tributo : NF Item (Todos) SOMA", num(cof, "vCOFINS"), "num"),
        ("PIS Valor", "PIS: Valor do Tributo : NF Item (Todos) SOMA", num(pis, "vPIS"), "num"),
        ("IPI Valor", "IPI: Valor do Tributo : NF Item (Todos) SOMA", num(ipi, "vIPI"), "num"),
        ("ISS BC", "ISS: Base de Cálculo : NF Item (Todos) SOMA", num(iss, "vBC"), "num"),
        ("ISS Valor", "ISS: Valor do Tributo : NF Item (Todos) SOMA", num(iss, "vISSQN"), "num"),
        ("ICMS Alíq", "ICMS: Alíquota : NF Item (Todos) NOISE OR", aliq, "num"),
        ("ICMS BC", "ICMS: Base de Cálculo : NF Item (Todos) SOMA", bc, "num"),
        ("ICMS Valor", "ICMS: Valor do Tributo : NF Item (Todos) SOMA", vic, "num"),
        ("ICMS ST", "ICMS por Substituição: Valor do Tributo : NF Item (Todos) SOMA", vst, "num"),
    ]
    return tipo, (local(sub.tag) if sub is not None else "<ausente>"), name_of(emit), F


def verdict(kind, exp, got):
    """Retorna (ok|None, nota). None = informativo, não contabiliza."""
    g = (got or "").strip()
    if kind == "info": return None, "contexto"
    if kind == "tabela":
        if not CFOP_TABLE_OK:
            return None, "CONFAZ indisponível"
        return ws(str(exp)).rstrip(".").casefold() == ws(str(got)).rstrip(".").casefold(), "CONFAZ"
    if kind == "escala":
        v = to_num(g)
        if v is None: return None, "s/ valor"
        if abs(v - exp) <= 0.00005: return True, "percentual"
        if abs(v - exp / 100.0) <= 0.00005: return True, "fração"
        return False, f"esperado {exp} ou {exp/100.0}"
    if kind == "num":
        v = to_num(g)
        return (v is not None and abs(v - float(exp)) <= 0.005), ""
    e = ws(str(exp))
    if kind == "nome" and e == ND: return None, "fallback permitido"
    if kind == "mun": return e.casefold() == ws(g).casefold(), ""
    return e == ws(g), ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("keys", nargs="*")
    ap.add_argument("-n", "--qtd", type=int)
    ap.add_argument("--seed", type=int, default=1973)
    ap.add_argument("--min-itens", type=int, default=2)
    ap.add_argument("--receita", required=True)
    ap.add_argument("--gerado", required=True)
    ap.add_argument("--zip-dir", default=".")
    ap.add_argument("--so-divergentes", action="store_true", help="omite linhas OK")
    a = ap.parse_args()
    if not a.keys and not a.qtd: ap.error("informe chaves ou -n N")

    ref, gen = load_csv(a.receita), load_csv(a.gerado)
    common = sorted(set(ref) & set(gen))
    print("=== Índice (diretório central, sem descompressão) ===")
    idx = build_index(a.zip_dir)
    print(f"  {len(idx)} chaves indexadas")
    print(f"  Tabela CONFAZ: {len(CFOP_TABLE)} código(s)"
          + ("" if CFOP_TABLE_OK else " — INSUFICIENTE: Descrição CFOP não contabilizada"))

    if a.keys:
        keys = [k.upper() for k in a.keys]
    else:
        nfes = [k for k in common if is_nfe(k)]
        if nfes:
            multi = [k for k in nfes if len([i for i in gen[k] if i != "0"]) >= a.min_itens]
            pool = multi or nfes
            nota = f"NFe >= {a.min_itens} itens: {len(multi)}" if multi else "AVISO: nenhuma multi-item; usando 1 item"
        else:
            pool, nota = common, "CT-es"
        print(f"  Universo: {len(common)} | {nota}")
        keys = sorted(random.Random(a.seed).sample(pool, min(a.qtd, len(pool))))
        print(f"  Seed={a.seed} | sorteadas: {len(keys)}")

    falhas, div_contagil, auditados = 0, Counter(), 0
    for k in keys:
        e = idx.get(k)
        print("=" * 100); print(f"CHAVE: {k}")
        if e is None: print("  XML não localizado."); continue
        try: root = ET.fromstring(decode(read_entry(e)))
        except ET.ParseError as ex: print(f"  XML ilegível: {ex}"); continue
        if own_key(root) != k: print("  IGNORADO: chave própria divergente."); continue
        print(f"XML  : {e[0]}")
        if is_nfe(k):
            print("  (NF-e: use a versão anterior para rateio por item; aqui foco CT-e)")
            continue
        tipo, grupo, emit, fields = cte_fields(root)
        auditados += 1
        print(f"[{tipo}] ICMS=<{grupo}> | emit={emit!r}")
        print(f"  {'Campo':<15}{'XML/regra':>16}{'ContÁgil':>16}{'read-xml':>16}  rx  ct")
        r, g = ref.get(k, {}).get("1", {}), gen.get(k, {}).get("1", {})
        for lbl, col, exp, kind in fields:
            rv, gv = r.get(col, "?"), g.get(col, "?")
            ok, nota = verdict(kind, exp, gv)
            okc, _ = verdict(kind, exp, rv)
            if ok is False: falhas += 1
            if okc is False: div_contagil[col] += 1
            if a.so_divergentes and ok is not False and okc is not False: continue
            sx = f"{exp:.4f}" if isinstance(exp, float) else str(exp)[:15]
            mark = {True: "OK", False: "DIFF", None: "--"}[ok]
            markc = {True: "OK", False: "DIFF", None: "--"}[okc]
            print(f"  {lbl:<15}{sx:>16}{rv[:15]:>16}{gv[:15]:>16}  {mark:<5}{markc} {nota}")
            if (ok is False or okc is False) and kind not in ("num", "escala"):
                print(f"      XML/regra: {exp}")
                print(f"      ContÁgil : {rv}")
                print(f"      read-xml : {gv}")

    print("=" * 100)
    print(f"RESULTADO: {auditados} doc(s) | read-xml vs XML: {falhas} divergência(s)")
    if div_contagil:
        print(f"\nContÁgil vs XML — {len(div_contagil)} coluna(s) divergente(s):")
        for col, n in div_contagil.most_common():
            print(f"  {n:>4}  {col}")
    return 1 if falhas else 0


if __name__ == "__main__":
    raise SystemExit(main())
