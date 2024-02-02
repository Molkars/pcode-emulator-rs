
# Molkars' PCode Emulator

a simplistic-ish PCode emulator

## Quick Start

### Prerequisites
- [cargo (via rustup)](https://rustup.rs)
- a generic c-compiler: `cc`

<details>
<summary>Example Program</summary>
<code>

#include <stdio.h>

extern void exit(int);

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: example [arg]\n");
        exit(1);
    }
    fprintf(stdout, "hi!\n");
    return 0;
}
</code>
</details>

```console
$ sudo apt install gcc-multilib
$ cc -static -m32 -target i386-pc-linux-gnu your-main.c -o out.bin
$ cargo run -- ./out.bin (...args)
```
