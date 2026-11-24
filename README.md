# black-ops-1-press-kit-decomp

Evidence-driven reconstruction of the 2010 Black Ops press-kit application, distinct from the T5 game.

- Windows and macOS Adobe projector binaries: historical reference only, out of reconstruction scope.
- Shared launcher and main press-kit SWFs: AVM1 bytecode analysis and faithful native reconstruction.
- Original media stays in R2; do not commit executables, media, SWFs, or Flash runtime binaries.

See [AGENTS.md](AGENTS.md), [target.toml](target.toml), [STATUS.md](STATUS.md), and [initial evidence](docs/originals.md).

Build and test: `cmake -S . -B build && cmake --build build && ctest --test-dir build --output-on-failure`.

Adobe projector analysis is disabled by owner decision. SWF application analysis runs independently in Actions. The SWF workflow exports AVM1 timelines, bytecode and FFDec ActionScript into a restricted-retention artifact. The reconstructed library is not yet a playable application. The payload workflow downloads and hashes the full recorded R2 inventory without committing the original files.
