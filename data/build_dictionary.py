#!/usr/bin/env python3
"""Build the bundled Sinhala word-frequency list from a Wikipedia dump.

Usage (run with -I, point at the dump by path; do not run from the dump's folder):
    python3 -I data/build_dictionary.py <siwiki-pages-articles.xml.bz2> core/data/si-words.tsv [min_count]

Output: UTF-8 TSV, `word<TAB>count`, sorted by word in Unicode code point order
(which equals UTF-8 byte order, as the Rust binary search requires).

Source text: Sinhala Wikipedia, CC BY-SA 4.0 / GFDL. See data/README.md.
"""
import bz2
import collections
import re
import sys
import xml.etree.ElementTree as ET

# Sinhala block plus ZWJ (rakaransaya/yansaya). A word is a maximal run of these.
WORD = re.compile(r"[඀-෿‍]+")
# Crude wikitext stripping: enough to avoid counting markup, not a full parser.
TEMPLATE = re.compile(r"\{\{[^{}]*\}\}")
TAG = re.compile(r"<[^>]+>")
URL = re.compile(r"https?://\S+")
LINK = re.compile(r"\[\[(?:[^\]|]*\|)?([^\]]*)\]\]")
SINHALA_LETTER = re.compile(r"[අ-ෆ]")  # independent vowels + consonants


def clean(text: str) -> str:
    for _ in range(3):  # nested templates
        text = TEMPLATE.sub(" ", text)
    text = TAG.sub(" ", text)
    text = URL.sub(" ", text)
    return LINK.sub(r"\1", text)


def valid(word: str) -> bool:
    # Drop stray mark-only and ZWJ runs; single letters are kept so that typing "ka" can offer ක.
    word = word.strip("‍")
    return bool(SINHALA_LETTER.search(word)) and word[0] != "‍"


def main() -> None:
    src, dst = sys.argv[1], sys.argv[2]
    min_count = int(sys.argv[3]) if len(sys.argv) > 3 else 3
    counts: collections.Counter = collections.Counter()
    pages = 0
    with bz2.open(src, "rb") as f:
        for _, elem in ET.iterparse(f):
            if elem.tag.endswith("}text") and elem.text:
                counts.update(w for w in WORD.findall(clean(elem.text)) if valid(w))
                pages += 1
            elem.clear()
    kept = sorted((w, c) for w, c in counts.items() if c >= min_count)
    with open(dst, "w", encoding="utf-8", newline="\n") as out:
        for w, c in kept:
            out.write(f"{w}\t{c}\n")
    print(f"pages={pages} distinct={len(counts)} kept(>={min_count})={len(kept)}")


if __name__ == "__main__":
    main()
