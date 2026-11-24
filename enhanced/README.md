# Enhanced C++
Independent optional enhancement layer on faithful C++ library. Currently contains an opt-in gallery loading/index consistency fix with a regression test. The primary new development target is enhanced-rust. No changes to faithful application semantics.

Build with the root CMake project and -DPRESSKIT_BUILD_PLAYER=ON. Separate executable: build/enhanced/presskit_player_enhanced. The shared host selects enhanced gallery semantics at compile time, not in the faithful target. Qt remains a reference C++ host dependency.
