# Working in black-ops-1-press-kit-decomp

Follow the sister decomp repositories: analysis/<build>/ is generated, faithful/src/ and faithful/include/ are faithful authored code, faithful/tests/ holds C++ validation, records/ holds authored evidence/status, docs/ holds technical notes, enhanced/ holds optional changes only, tools/ holds reproducible pipelines. Never edit generated exports by hand.

Use build + address as identity. Native addresses are virtual addresses; AVM1 addresses are decimal offsets in the uncompressed SWF including its eight-byte header. Never conflate projector/runtime code with authored press-kit code or the retail T5 game. No speculative functions, no fabricated names, no fake byte-match claims. Every reconstruction must cite original instructions/offsets; use the ladder located -> named -> typed -> decompiled -> compiles -> matches -> integrated. Synthetic unit tests alone do not establish matches. Preserve original behavior and bugs in src; record fixes in records/bugs.tsv and implement enhancements separately. Original binaries/assets remain outside git. Do not execute untrusted legacy binaries on CI. Graph re_t5_press_kit is reserved for derived data, not yet synchronized.

## Scope (owner decision 2026-11-24)
Reconstruct only the press-kit application and its assets. Adobe Flash Player and bundled mx.video components are explicitly out of scope; do not run projector Ghidra/decompiler workflows or implement a generic Flash VM. Existing native analysis is historical reference only. Use a native media/rendering backend and offline extraction of original artwork; no Adobe runtime dependency. Track authored AVM1 functions independently of third-party code.

## Three targets
Primary development target is enhanced-rust/ (Rust 2024, wgpu, Slang offline to WGSL). faithful/ is the C++ reference, enhanced/ is the optional C++ enhancement layer. Keep their semantics and builds independent. Shared original evidence/asset preparation remains at repository root. Never use public runners to compile or expose private source; use R2 handoff only for tools and public dependencies, local compilation where available. No Adobe decompilation.
