# Black Ops 2010 press-kit reconstruction

Three separate targets, sharing original evidence and offline asset tools. **Primary development: enhanced-rust (Rust, wgpu, Slang).**

- `faithful/`: C++ application reconstruction, reference Qt host, original behavior.
- `enhanced/`: separate optional C++ enhancements linked to faithful library.
- `enhanced-rust/`: independent Rust application, no C++/Qt/Adobe runtime dependency.
- `analysis/`, `records/`, `tools/`, `shared/`: original evidence, ledger, extraction and shared fixtures.

C++: `cmake -S . -B build && cmake --build build && ctest --test-dir build --output-on-failure`.
Rust: see [enhanced-rust/README.md](enhanced-rust/README.md). Original assets stay in R2, not Git. Adobe projector and library decompilation are out of scope.
