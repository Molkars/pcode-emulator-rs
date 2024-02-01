
CC=clang
OBJDUMP=llvm-objdump-15
NM=llvm-nm

PARITY=4

if [[ ! -d binaries/ ]]
then
  mkdir binaries/
fi

compile() {
  $CC -static -m32 -target "$1" example.c -o "binaries/$1.bin"
  $OBJDUMP --x86-asm-syntax=intel -d "binaries/$1.bin" > "binaries/$1.objdump"
  $NM --numeric-sort "binaries/$1.bin" > "binaries/$1.nm"
}

compile x86_64-pc-linux-gnu
compile i386-pc-linux-gnu