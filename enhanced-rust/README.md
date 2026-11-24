# Enhanced Rust (primary target)

Rust 2024, pinned R2 toolchain 1.100.0. Independent of C++, Qt and Flash. Workspace: core (pure app state and original-derived hit regions), renderer (wgpu texture compositor), app (winit native development host).

`cargo test --workspace` and `cargo run -p presskit-app -- /path/to/prepared-assets`. Canonical Slang source compiled offline with Slang 2026.22.1 into checked-in generated WGSL. Regenerate: `python3 tools/compile_shaders.py --out shaders/generated`. The app embeds these generated modules; no shader compiler needed at runtime.

Local validation 2026-11-24: Rust 1.100.0, complete workspace cargo check and cargo test offline, 4 behavior tests and headless wgpu shader pixel readback test passed (Mesa llvmpipe GL software adapter, not discrete GPU). Public dependency/toolchain handoffs only; no application source on public runner. Video/audio, original controls, mobile and browser host remain unimplemented. Enter skips unavailable media in development host. Source assets remain external; see root tools/prepare_player_assets.py.
