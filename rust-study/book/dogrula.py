#!/usr/bin/env python3
"""Kitap doğrulama aracı.

Kullanım:
    python3 dogrula.py                      # bütün kitap, bütün denetimler
    python3 dogrula.py src/temel/03-*.md    # yalnız verilen sayfalar
    python3 dogrula.py --yalniz kod,cikti src/temel/03-ownership.md
    python3 dogrula.py --kapsam src/temel/03-ownership.md src/temel/03-ownership-derinlesme.md

Denetimler:
    yapi    SUMMARY ile dosyalar, başlıklar, dipnotlar, yasak ifadeler, biçim kuralları
    kod     Her Rust bloğu rustdoc ile derlenir ve çalıştırılır (hata kodları dâhil)
    cikti   "Çıktı:" ile verilen metin, programın gerçek çıktısıyla karşılaştırılır;
            compile_fail bloğundan sonraki derleyici mesajı gerçek mesajla karşılaştırılır
    link    Resmî belge linkleri ve çapaları yerel resmî belgeden, rustc kaynak linkleri
            sürüm etiketinden, ötekiler internetten denetlenir
    kapsam  (yalnız istenirse) eski sayfada ve notlarda geçip yeni sayfada geçmeyen terimler
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import os
import re
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

KOK = Path(__file__).resolve().parent
SRC = KOK / "src"
NOTLAR = KOK.parent / "learn"
ONBELLEK = KOK / ".onbellek"
ILK_HAL = KOK / "yedek" / "kitap-ilk-hal-2026-11-23.tar.gz"
EDITION = "2024"
RUSTC_ETIKET = "1.99.0"

SYSROOT = Path(subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True).stdout.strip())
BELGELER = SYSROOT / "share/doc/rust/html"
STD_KAYNAK = SYSROOT / "lib/rustlib/src/rust"

# Kitap bölümü -> not dosyaları (kapsam denetimi için)
NOT_HARITASI = {
    "01-temel-kavramlar": ["basic-concepts"],
    "02-yaygin-kavramlar": ["common-concepts"],
    "03-ownership": ["ownership"],
    "04-structs": ["structs"],
    "05-enums": ["enums"],
    "06-modules": ["modules"],
    "07-collections": ["collections"],
    "08-error-handling": ["error-handling"],
    "09-generics-traits-lifetimes": ["generics-traits-lifetimes"],
    "10-writing-tests": ["writing-tests"],
    "11-minigrep": ["minigrep"],
    "12-closures-iterators": ["closures-iterators"],
    "13-cargo-and-crates": ["cargo-and-crates"],
    "14-smart-pointers": ["smart-pointers"],
    "15-concurrency": ["concurrency"],
    "16-async-await": ["async-await"],
    "17-oop": ["oop"],
    "18-patterns": ["patterns"],
    "19-advanced-features": ["advanced-features"],
    "20-macros": ["macros"],
    "21-variance": ["advanced-concepts/ileri-variance"],
    "22-sized-dst": ["advanced-concepts/ileri-sized-dst"],
    "23-coherence": ["advanced-concepts/ileri-coherence"],
    "24-deref-metot": ["advanced-concepts/ileri-deref-metot"],
    "25-send-sync": ["advanced-concepts/ileri-send-sync"],
    "26-dropck": ["advanced-concepts/ileri-dropck"],
    "27-lifetime-hrtb-gat": ["advanced-concepts/ileri-lifetime-hrtb-gat"],
    "28-future": ["advanced-concepts/ileri-future"],
    "29-ileri-eszamanlilik-async": ["advanced-concepts/ileri-eszamanlilik-async/src/main.rs"],
    "29-threadler": ["advanced-concepts/ileri-eszamanlilik-async/src/threadler.rs"],
    "29-calisma-zamani": ["advanced-concepts/ileri-eszamanlilik-async/src/calisma_zamani.rs"],
    "29-istemci": ["advanced-concepts/ileri-eszamanlilik-async/src/istemci.rs"],
}

YASAK = [
    (r"↳", "eski kaynak satırı; dipnot kullan"),
    (r"(?i)5 yaşındaki|aptala anlat", "eski başlık; 'Günlük hayattan bir benzetme' kullan"),
    (r"Konu anlatımı \(madde madde\)|Kodda nasıl görünür\?", "eski bölüm iskeleti"),
    (r"SADE ANLATIM|MİNİ SÖZLÜK|DERLEYİCİ PERSPEKTİFİ", "not başlığı; kitap başlığına çevir"),
    (r"(?i)çalışma anı", "'çalışma zamanı' kullan"),
    (r"(?i)derleme anı", "'derleme zamanı' kullan"),
    (r"1\.99\.1", "doğrulama sürümü 1.99.0"),
    (r"rcs/(arc|rc)\.rs", "1.99.0'da yol alloc/src/sync.rs ve alloc/src/rc.rs"),
    (r"\b(hala|kagit|imkan|hal)\b(?!')", "şapka: hâlâ, kâğıt, imkân, hâl"),
    (r"[✗✓✅❌]", "düzyazıda işaret yerine sözcük kullan"),
]
BAGIRMA_IZIN = {
    "HRTB", "RAII", "GAT", "GATS", "DST", "ZST", "FFI", "AST", "HIR", "MIR", "THIR", "LLVM", "NLL", "UTF",
    "TCP", "HTTP", "JSON", "TOML", "ASCII", "CPU", "API", "ABI", "IDE", "RFC", "SIMD", "ERROR", "SAFETY",
    "POSIX", "TOCTOU", "LIFO", "FIFO", "RAM", "UB", "DNS", "URL", "HTML", "CSS", "SQL", "IO", "STD", "GC",
    "RwLock", "README", "SUMMARY", "VTABLE", "YAML", "SOLID", "CRTP", "PATH", "HOME", "CARGO", "RUST",
    "NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION", "IGNORE_CASE", "RUST_BACKTRACE", "RUST_TEST_THREADS", "CI", "MSRV", "SPDX", "MIT", "OOP", "ADT",
}


# --------------------------------------------------------------------------- yardımcılar
class Rapor:
    def __init__(self):
        self.hata, self.uyari = [], []

    def h(self, yer, mesaj):
        self.hata.append(f"HATA   {yer}: {mesaj}")

    def u(self, yer, mesaj):
        self.uyari.append(f"UYARI  {yer}: {mesaj}")


def goreli(p: Path) -> str:
    try:
        return str(p.relative_to(KOK))
    except ValueError:
        return str(p)


def bloklar(metin: str):
    """[(satir_no, dil, nitelikler, içerik, bitis_satiri)]"""
    satirlar = metin.splitlines()
    i, sonuc = 0, []
    while i < len(satirlar):
        m = re.match(r"^(\s*)(`{3,})(.*)$", satirlar[i])
        if not m:
            i += 1
            continue
        girinti, tik, bilgi = m.group(1), m.group(2), m.group(3).strip()
        j, govde = i + 1, []
        while j < len(satirlar) and not re.match(rf"^\s*{tik}\s*$", satirlar[j]):
            s = satirlar[j]
            govde.append(s[len(girinti):] if s.startswith(girinti) else s)
            j += 1
        parcalar = [x.strip() for x in re.split(r"[,\s]+", bilgi) if x.strip()]
        sonuc.append((i + 1, parcalar[0] if parcalar else "", parcalar[1:], "\n".join(govde), j + 1))
        i = j + 1
    return sonuc


def duzyazi(metin: str) -> list[tuple[int, str]]:
    """Kod bloklarını ve satır içi kodu çıkarılmış satırlar."""
    icinde, sonuc = False, []
    for n, s in enumerate(metin.splitlines(), 1):
        if re.match(r"^\s*`{3,}", s):
            icinde = not icinde
            continue
        if not icinde:
            sonuc.append((n, re.sub(r"`+[^`]*`+", "", s)))
    return sonuc


def rustdoc_kaynagi(kod: str) -> str:
    """rustdoc'un gizli satır kuralını uygular ve gerekirse main ile sarar."""
    satirlar, nitelik = [], []
    for s in kod.splitlines():
        t = s.lstrip()
        if t == "#":
            s = ""
        elif t.startswith("# "):
            s = s[: len(s) - len(t)] + t[2:]
        elif t.startswith("##"):
            s = s[: len(s) - len(t)] + t[1:]
        if s.startswith("#!["):
            nitelik.append(s)
        else:
            satirlar.append(s)
    govde = "\n".join(satirlar)
    if not re.search(r"\bfn\s+main\s*\(", govde):
        govde = "fn main() {\n" + govde + "\n}"
    return "\n".join(["#![allow(unused)]", *nitelik, govde])


def derle(kod: str, dizin: Path, ad: str, sinirla: bool = True):
    """sinirla=False iken deny düzeyindeki lint'ler de hata sayılır (arithmetic_overflow gibi)."""
    f = dizin / f"{ad}.rs"
    f.write_text(rustdoc_kaynagi(kod))
    ikili = dizin / ad
    r = subprocess.run(
        ["rustc", "--edition", EDITION, *(["--cap-lints", "warn"] if sinirla else []),
         "-o", str(ikili), str(f)],
        capture_output=True, text=True, cwd=dizin)
    return r.returncode == 0, r.stderr, ikili


# --------------------------------------------------------------------------- yapı
def denet_yapi(dosyalar, rapor: Rapor, tam: bool):
    ozet = (SRC / "SUMMARY.md").read_text()
    etiketler = {p: ad for ad, p in re.findall(r"\[((?:[^\[\]]|\\\])+)\]\(([^)]+\.md)\)", ozet)}
    if tam:
        for p in etiketler:
            if not (SRC / p).exists():
                rapor.h("SUMMARY.md", f"eksik dosya {p}")
        for p in SRC.rglob("*.md"):
            g = str(p.relative_to(SRC))
            if g != "SUMMARY.md" and g not in etiketler:
                rapor.h("SUMMARY.md", f"listede olmayan dosya {g}")
    for f in dosyalar:
        metin, yer = f.read_text(), goreli(f)
        g = str(f.relative_to(SRC)) if f.is_relative_to(SRC) else f.name
        ilk = next((s for s in metin.splitlines() if s.startswith("# ")), "")
        if g in etiketler and ilk[2:] != etiketler[g]:
            rapor.h(yer, f"başlık SUMMARY ile farklı: '{ilk[2:]}' != '{etiketler[g]}'")
        satirlar = duzyazi(metin)
        if sum(1 for _, s in satirlar if s.startswith("# ")) != 1:
            rapor.h(yer, "tam bir tane H1 olmalı")
        onceki = 1
        for n, s in satirlar:
            m = re.match(r"^(#{2,6}) ", s)
            if m:
                d = len(m.group(1))
                if d > onceki + 1:
                    rapor.h(f"{yer}:{n}", f"başlık düzeyi atlıyor: {s}")
                onceki = d
        # dipnotlar
        duz = "\n".join(s for _, s in satirlar)
        tanim = set(re.findall(r"^\[\^([^\]]+)\]:", duz, re.M))
        kullanim = set(re.findall(r"\[\^([^\]]+)\](?!:)", duz))
        for x in sorted(kullanim - tanim):
            rapor.h(yer, f"tanımsız dipnot [^{x}]")
        for x in sorted(tanim - kullanim):
            rapor.h(yer, f"kullanılmayan dipnot [^{x}]")
        if g not in ("index.md", "giris.md", "SUMMARY.md", "kaynaklar.md", "terimler.md") and len(tanim) < 4:
            rapor.u(yer, f"yalnız {len(tanim)} kaynak dipnotu var")
        # yasak ifadeler ve biçim
        for n, s in satirlar:
            for desen, neden in YASAK:
                m = re.search(desen, s)
                if m:
                    rapor.u(f"{yer}:{n}", f"'{m.group(0)}' -> {neden}")
            for m in re.finditer(r"<([A-Za-z][A-Za-z0-9_]*)>", s):
                if m.group(1) not in {"details", "summary", "br", "kbd", "sub", "sup", "p", "div", "span"}:
                    rapor.h(f"{yer}:{n}", f"ters tırnaksız <{m.group(1)}> HTML etiketi sanılır")
            for m in re.finditer(r"\b[A-ZÇĞİÖŞÜ]{4,}\b", re.sub(r"\]\([^)]*\)", "", s)):
                if m.group(0) not in BAGIRMA_IZIN and not re.match(r"^E\d", m.group(0)):
                    rapor.u(f"{yer}:{n}", f"büyük harfle vurgu: {m.group(0)}")
            if "→" in s and not s.startswith("[^") and "|" not in s:
                rapor.u(f"{yer}:{n}", "düzyazıda ok işareti; cümle kur")
        # kod blokları
        for n, dil, nit, govde, _ in bloklar(metin):
            if not dil:
                rapor.h(f"{yer}:{n}", "dili belirtilmemiş kod bloğu (text kullan)")
            if dil == "rust":
                if "ignore" in nit:
                    rapor.u(f"{yer}:{n}", "ignore edilmiş Rust bloğu (gerekçesi metinde olmalı)")
                if "compile_fail" in nit and not any(re.match(r"E\d{4}", x) for x in nit):
                    rapor.u(f"{yer}:{n}", "compile_fail bloğunda hata kodu yok (kodsuz hata değilse ekle)")
                gorunen = [s for s in govde.splitlines() if not re.match(r"^\s*#( |$)", s)]
                if len(gorunen) > 60:
                    rapor.u(f"{yer}:{n}", f"uzun kod bloğu: {len(gorunen)} görünen satır; böl")
                en = max((len(s) for s in gorunen), default=0)
                if en > 100:
                    rapor.u(f"{yer}:{n}", f"çok geniş kod satırı: {en} sütun")
        # iç linkler
        for n, s in satirlar:
            for hedef in re.findall(r"\]\(([^)\s]+)\)", s):
                if re.match(r"^(https?:|#|mailto:)", hedef):
                    continue
                yol = hedef.split("#")[0]
                if yol and not (f.parent / yol).exists():
                    rapor.h(f"{yer}:{n}", f"kırık iç link {hedef}")


# --------------------------------------------------------------------------- kod
def denet_kod(dosyalar, rapor: Rapor):
    env = dict(os.environ, RUSTC_BOOTSTRAP="1")  # rustdoc'un hata kodlarını da denetlemesi için

    def bir(f: Path):
        with tempfile.TemporaryDirectory() as d:
            r = subprocess.run(["rustdoc", "--test", "--edition", EDITION, str(f)],
                               capture_output=True, text=True, env=env, cwd=d)
        return f, r

    with cf.ThreadPoolExecutor(max_workers=max(2, (os.cpu_count() or 4) // 2)) as ex:
        for f, r in ex.map(bir, dosyalar):
            m = re.findall(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", r.stdout)
            if m:
                g, k, t = (sum(int(x[i]) for x in m) for i in range(3))
                print(f"  kod    {goreli(f)}: {g} geçti, {k} kaldı, {t} atlandı")
            if r.returncode != 0:
                ayrinti = re.findall(r"---- .*? stdout ----\n(?:.*\n)*?(?=\n\n---- |\nfailures:)", r.stdout)
                rapor.h(goreli(f), "rustdoc testi başarısız\n" + "\n".join(a[:1500] for a in ayrinti)[:6000]
                        if ayrinti else "rustdoc testi başarısız\n" + (r.stdout + r.stderr)[-3000:])


def sade(s: str) -> str:
    return re.sub(r"\s+", " ", re.sub(r"^\s*\d*\s*\|", "", s)).strip()


def denet_cikti(dosyalar, rapor: Rapor):
    isler = []
    for f in dosyalar:
        metin = f.read_text()
        satirlar, bl = metin.splitlines(), bloklar(metin)
        for a, b in zip(bl, bl[1:]):
            if a[1] != "rust" or b[1] not in ("text", "console"):
                continue
            ara = " ".join(satirlar[a[4]: b[0] - 1])
            if len(ara) > 600 or re.search(r"^#+ ", "\n".join(satirlar[a[4]: b[0] - 1]), re.M):
                continue
            isler.append((f, a, b, ara))

    def bir(is_):
        f, a, b, ara = is_
        yer = f"{goreli(f)}:{b[0]}"
        with tempfile.TemporaryDirectory() as d:
            if "compile_fail" in a[2]:
                ok, err, _ = derle(a[3], Path(d), "blok", sinirla=False)
                if ok:
                    return ("h", yer, "compile_fail bloğu derlendi")
                gercek = {sade(s) for s in err.splitlines()}
                bulunan = 0
                for s in b[3].splitlines():
                    t = sade(s)
                    if re.match(r"^(error|warning)(\[E\d+\])?:|^= (note|help):|^(note|help):", t):
                        bulunan += 1
                        if t not in gercek:
                            return ("h", yer, f"derleyici mesajı gerçek çıktıda yok: {t}\n--- gerçek ---\n{err[:1200]}")
                if not bulunan and re.search(r"[Dd]erleyici", ara):
                    return ("u", yer, "derleyici mesajı olarak verilen blokta error satırı yok")
                return None
            if any(x in a[2] for x in ("ignore", "no_run")) or not re.search(r"[Çç]ıktı", ara):
                return None
            ok, err, ikili = derle(a[3], Path(d), "blok")
            if not ok:
                return ("h", yer, f"çıktısı denetlenecek blok derlenmedi:\n{err[:1200]}")
            try:
                r = subprocess.run([str(ikili)], capture_output=True, text=True, timeout=30, cwd=d)
            except subprocess.TimeoutExpired:
                return ("h", yer, "program 30 saniyede bitmedi")
            bek = [s.rstrip() for s in b[3].strip("\n").splitlines()]
            ger = [s.rstrip() for s in r.stdout.strip("\n").splitlines()]
            if "should_panic" in a[2]:
                ger_err = r.stderr
                for s in bek:
                    if s and s not in ger and s.strip() not in ger_err:
                        return ("h", yer, f"panik çıktısında yok: {s}\n--- gerçek ---\n{ger_err[:800]}")
                return None
            if bek != ger:
                return ("h", yer, "çıktı farklı\n--- yazan ---\n" + "\n".join(bek) + "\n--- gerçek ---\n" + "\n".join(ger))
        return None

    with cf.ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as ex:
        for s in ex.map(bir, isler):
            if s:
                (rapor.h if s[0] == "h" else rapor.u)(s[1], s[2])
    print(f"  çıktı  {len(isler)} kod/çıktı çifti incelendi")


# --------------------------------------------------------------------------- linkler
def getir(url: str) -> tuple[int, str]:
    ONBELLEK.mkdir(exist_ok=True)
    f = ONBELLEK / (hashlib.sha1(url.encode()).hexdigest())
    if f.exists():
        return 200, f.read_text(errors="replace")
    r = subprocess.run(["curl", "-sSL", "--max-time", "40", "--retry", "2", "-o", str(f), "-w", "%{http_code}", url],
                       capture_output=True, text=True)
    kod = int(r.stdout.strip() or 0) if r.stdout.strip().isdigit() else 0
    if kod != 200:
        f.unlink(missing_ok=True)
        return kod, ""
    return 200, f.read_text(errors="replace")


def denet_link(dosyalar, rapor: Rapor):
    linkler = {}
    for f in dosyalar:
        for n, s in enumerate(f.read_text().splitlines(), 1):
            for m in re.finditer(r"\[((?:[^\[\]]|\[[^\]]*\])*)\]\((https?://[^)\s]+)\)", s):
                linkler.setdefault(m.group(2), []).append((f"{goreli(f)}:{n}", m.group(1)))
            for m in re.finditer(r"(?<![\(\w])<(https?://[^>\s]+)>", s):
                linkler.setdefault(m.group(1), []).append((f"{goreli(f)}:{n}", ""))

    def bir(oge):
        url, yerler = oge
        taban, _, capa = url.partition("#")
        sorun = []
        m = re.match(r"https://doc\.rust-lang\.org/(?:stable/)?(.*)$", taban)
        if m and not m.group(1).startswith(("nightly/", "beta/", "1.")):
            yol = m.group(1) or "index.html"
            p = BELGELER / (yol + "index.html" if yol.endswith("/") else yol)
            if not p.exists():
                if yol.startswith(("unstable-book", "rustc/", "rustdoc/", "clippy/", "style-guide", "edition-guide")):
                    kod, t = getir(taban)
                    if kod != 200:
                        sorun.append(f"HTTP {kod}")
                else:
                    sorun.append("resmî belgede böyle bir sayfa yok")
                t = "" if sorun else (t if not p.exists() else "")
            else:
                t = p.read_text(errors="replace")
                if re.search(r'http-equiv="refresh"', t, re.I):
                    sorun.append("sayfa taşınmış (yönlendirme)")
            if capa and not sorun and f'id="{capa}"' not in t:
                sorun.append(f"çapa yok: #{capa}")
        elif taban.startswith("https://github.com/rust-lang/rust/"):
            m = re.match(r"https://github\.com/rust-lang/rust/blob/([^/]+)/(.+)$", taban)
            if m and m.group(1) == RUSTC_ETIKET:
                yol, t = m.group(2), None
                if yol.startswith("library/") and (STD_KAYNAK / yol).exists():
                    t = (STD_KAYNAK / yol).read_text(errors="replace")
                else:
                    kod, t = getir(f"https://raw.githubusercontent.com/rust-lang/rust/{RUSTC_ETIKET}/{yol}")
                    if kod != 200:
                        sorun.append(f"kaynak dosya {RUSTC_ETIKET} etiketinde yok (HTTP {kod})")
                if t and not sorun:
                    sat = t.splitlines()
                    for yer, etiket in yerler:
                        semboller = [x for x in re.findall(r"`([^`]+)`", etiket) if not x.endswith(".rs")]
                        aralik = re.findall(r"L(\d+)", capa)
                        if aralik:
                            a, b = int(aralik[0]), int(aralik[-1])
                            if b > len(sat):
                                sorun.append(f"{yer}: satır {b} yok (dosya {len(sat)} satır)")
                                continue
                            pencere = "\n".join(sat[max(0, a - 4): b + 4])
                        else:
                            pencere = t
                        for s in semboller:
                            if s not in pencere:
                                sorun.append(f"{yer}: `{s}` belirtilen yerde bulunamadı")
            elif m:
                sorun.append(f"kaynak linki {RUSTC_ETIKET} etiketini göstermeli")
        else:
            kod, t = getir(taban)
            if kod != 200:
                sorun.append(f"HTTP {kod}")
            elif capa and "github.com" not in taban and f'id="{capa}"' not in t and f"id={capa}" not in t \
                    and f'name="{capa}"' not in t:
                sorun.append(f"çapa yok: #{capa}")
        return url, yerler, sorun

    with cf.ThreadPoolExecutor(max_workers=6) as ex:
        for url, yerler, sorun in ex.map(bir, sorted(linkler.items())):
            for s in sorun:
                rapor.h(yerler[0][0], f"{url} -> {s}")
    print(f"  link   {len(linkler)} ayrı dış link denetlendi")


# --------------------------------------------------------------------------- kapsam
def denet_kapsam(dosyalar):
    """Not kaybını görmek için: eski sayfa ve notlardaki terimlerden yeni sayfada geçmeyenler."""
    yeni = "\n".join(f.read_text() for f in dosyalar)
    eski, notlar = "", ""
    with tarfile.open(ILK_HAL) as t:
        for f in dosyalar:
            g = "src/" + str(f.relative_to(SRC))
            try:
                eski += t.extractfile(g).read().decode() + "\n"
            except KeyError:
                pass
    for f in dosyalar:
        k = f.stem.removesuffix("-derinlesme")
        for yol in NOT_HARITASI.get(k, []):
            p = NOTLAR / yol
            for q in ([p] if p.suffix == ".rs" else sorted(p.rglob("*.rs")) + [p / "ACIKLAMA.md"]):
                if q.exists() and q.read_text() not in notlar:
                    notlar += q.read_text() + "\n"

    def kodlar(t):
        return set(re.findall(r"\bE0\d{3}\b", t))

    def ticks(t):
        return {x.strip() for x in re.findall(r"`([^`\n]{2,60})`", t)}

    def fnler(t):
        return set(re.findall(r"^\s*(?:pub )?(?:async )?(?:unsafe )?(?:fn|struct|enum|trait) (\w+)", t, re.M))

    def ev(baslik, kume):
        eksik = sorted(x for x in kume if x not in yeni)
        print(f"\n{baslik} ({len(eksik)}/{len(kume)} eksik):")
        for x in eksik:
            print("   ", x)

    ev("Notlarda ve eski sayfada olup yeni sayfada olmayan hata kodları", kodlar(eski + notlar))
    ev("Notlardaki fn/struct/enum/trait adları", {x for x in fnler(notlar) if x not in ("main", "calistir")})
    ev("Eski sayfadaki satır içi kodlar", ticks(re.sub(r"(?s)```.*?```", "", eski)))
    ev("Notlardaki satır içi kodlar", ticks(notlar))


# --------------------------------------------------------------------------- ana
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dosya", nargs="*")
    ap.add_argument("--yalniz", default="yapi,kod,cikti,link")
    ap.add_argument("--kapsam", action="store_true")
    a = ap.parse_args()
    tam = not a.dosya
    dosyalar = [Path(x).resolve() for x in a.dosya] or sorted(p for p in SRC.rglob("*.md") if p.name != "SUMMARY.md")
    if a.kapsam:
        denet_kapsam(dosyalar)
        return
    r, sec = Rapor(), set(a.yalniz.split(","))
    if "yapi" in sec:
        denet_yapi(dosyalar, r, tam)
    if "kod" in sec:
        denet_kod(dosyalar, r)
    if "cikti" in sec:
        denet_cikti(dosyalar, r)
    if "link" in sec:
        denet_link(dosyalar, r)
    for x in r.uyari + r.hata:
        print(x)
    print(f"\nSONUÇ: {len(dosyalar)} sayfa, {len(r.hata)} hata, {len(r.uyari)} uyarı")
    sys.exit(1 if r.hata else 0)


if __name__ == "__main__":
    main()
