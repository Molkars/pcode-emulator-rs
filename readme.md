
# Molkars' PCode Emulator

a simplistic-ish PCode emulator

## Quick Start

### Prerequisites
- [cargo (via rustup)](https://rustup.rs)
- a generic c-compiler: `cc`
- [objdump](https://man7.org/linux/man-pages/man1/objdump.1.html) & [nm](https://man7.org/linux/man-pages/man1/nm.1.html) on the current user's `PATH`
- a binary program ([we'll compile our own](example.c))
    ```console
    $ sudo apt install gcc-multilib
    $ cc -static -m32 -target i386-pc-linux-gnu your-main.c -o out.bin
    ```

### Usage
```console
$ cargo run -- emulate ./out.bin [...args]
```

## Reference
- [lifting-bits/sleigh C++ API Docs](https://grant-h.github.io/docs/ghidra/decompiler/sleighAPIbasic.html)
- [angr/pypcode](https://github.com/angr/pypcode), a pcode parsing & emulation library for python
- [black-binary/sleigh](https://github.com/black-binary/sleigh), a rust-crate that I forked as a starting-point for disassembly & translation
- [black-binary/sleigh-sys](https://github.com/black-binary/sleigh-sys), a rust-crate that I forked as a starting-point for interop with Sleigh C++
- [Lucas Ritzdorf](https://github.com/LRitzdorf), a friend of mine who answered some of my questions about processor architectures & assembly formats

<iframe src="https://github.com/LRitzdorf" title="Lucas">
</iframe>