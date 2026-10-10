# Data

Word list sources and the build script that produces the bundled dictionary (Phase 2).
Each source's licence must be recorded here before it is added.

## Sources and licences (verified 2026-10-10)

| Source | Status | Notes |
|---|---|---|
| Sinhala Wikipedia dump | **Approved** | dumps.wikimedia.org/legal.html: text under GFDL and CC BY-SA 4.0. Ship attribution (credit Wikipedia contributors, link CC BY-SA 4.0) in the installer/About. Derived frequency list is published under CC BY-SA 4.0. |
| LIRNEasia Facebook corpora | **Rejected** | README says CC BY 4.0, but Corpus-Alpha and Corpus-Sinhala-Redux are marked deprecated and removed from public access; licence over underlying Facebook posts is unclear. |
| FLORES+ Sinhala | Test set only | CC BY-SA 4.0 (per secondary source); ~2k sentences. Re-check the official repo before use. |
| CC-100 / OSCAR (Sinhala) | Deferred | Common Crawl terms; may contain personal data. Revisit only if Wikipedia alone is too thin. |
| hunspell-si | **Rejected** | GPLv2+; copyleft would apply to the dictionary. |
| Fernando & Dias, ICON 2021 list | Unknown | No licence found; ask the authors if needed. |

Decision: Phase 2 builds the frequency list from Sinhala Wikipedia.
Open risk: Wikipedia text is formal, so colloquial words may rank poorly; evaluate in Phase 2.
