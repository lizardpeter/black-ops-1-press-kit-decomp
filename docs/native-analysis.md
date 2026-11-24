# Native projector analysis

Original Ghidra run 45449260151 completed the three analyze jobs and produced saved project artifacts raw-windows_projector, raw-mac_x86_projector, raw-mac_ppc_projector. The decompile job was skipped due to a self-referencing condition accidentally introduced during bootstrap. The main workflow condition is corrected; ghidra-resume downloads the already analyzed projects and decompiles six shards per build. The resume publication is conditional on all shards completing. No original projector binaries are executed. Saved projects stay as CI artifacts rather than in git.

Native code is third-party Flash projector implementation, distinct from application-specific AVM1. Do not attribute its functions to Treyarch or to the T5 engine.
