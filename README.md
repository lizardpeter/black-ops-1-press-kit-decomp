# black-ops-1-press-kit-decomp

Evidence-driven reconstruction of the 2010 Black Ops press-kit application, distinct from the T5 game.

- Windows x86 and macOS PowerPC/x86 Adobe Flash projectors: native Ghidra analysis.
- Shared launcher and main press-kit SWFs: AVM1 bytecode analysis and faithful native reconstruction.
- Original media stays in R2; do not commit executables, media, SWFs, or Flash runtime binaries.

See [AGENTS.md](AGENTS.md), [target.toml](target.toml), [STATUS.md](STATUS.md), and [initial evidence](docs/originals.md).

Build and test: `cmake -S . -B build && cmake --build build && ctest --test-dir build --output-on-failure`.

Native Ghidra analysis and SWF analysis run independently in Actions. The reconstructed library is not yet a playable application.
