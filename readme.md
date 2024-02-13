
# Molkars' PCode Emulator

a simplistic-ish PCode emulator

## Quick Start

### Prerequisites
- [cargo (via rustup)](https://rustup.rs)
- the llvm compiler: clang v. 17.0.0 or later
- [llvm-readobj](https://llvm.org/docs/CommandGuide/llvm-readobj.html)
- a x86-32 binary executable

### Usage
```console
$ cargo run -- emulate ./out.bin
```

## References
- [lifting-bits/sleigh C++ API Docs](https://grant-h.github.io/docs/ghidra/decompiler/sleighAPIbasic.html)
- [angr/pypcode](https://github.com/angr/pypcode), a pcode parsing & emulation library for python
- [black-binary/sleigh](https://github.com/black-binary/sleigh), a rust-crate that I forked as a starting-point for disassembly & translation
- [black-binary/sleigh-sys](https://github.com/black-binary/sleigh-sys), a rust-crate that I forked as a starting-point for interop with Sleigh C++
- [Lucas Ritzdorf](https://github.com/LRitzdorf), a friend of mine who answered some of my questions about processor architectures & assembly formats


## Project Directory

[sleigh](./sleigh) - a rust library for disassembly & translation
[sleigh-sys](./sleigh-sys) - a rust library for interop with Sleigh C++
[binaries](./binaries) - a collection of binary programs to test the emulator translation
[tests](./tests) - a collection of tests for the emulator
  - format: `./tests/<test-name>/`
  - `./tests/<test-name>/bin` is the compiled binary (x86-32)
  - `./tests/<test-name>/main.c` is the source code

## Future Work

- [ ] implement floating point support
- [ ] implement & test structure, union, and enum support
- [ ] implement syscall/function interrupt support
