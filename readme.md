
# Molkars' PCode Emulator

a simplistic-ish PCode emulator

## Quick Start

### Prerequisites
- [cargo (via rustup)](https://rustup.rs)
- a generic c-compiler: `cc`
- a binary program [we'll compile our own](example.c)

```console
$ sudo apt install gcc-multilib
$ cc -static -m32 -target i386-pc-linux-gnu your-main.c -o out.bin
$ cargo run -- ./out.bin (...args)
```
