
if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <test-name>"
    exit 1
fi

name="$1"

clang -static -m32 -target i386-pc-linux-gnu -o "./tests/$name/bin" "./tests/$name/main.c" || {
    echo "Compilation failed"
    exit 1
}
llvm-objdump --x86-asm-syntax=intel -d "./tests/$name/bin" > "./tests/$name/objdump.txt" || {
    echo "Disassembly failed"
    exit 1
}

cargo run -- emulate "./tests/$name/bin" > "./tests/$name/log.txt" || {
    echo "Emulation failed"
    exit 1
}


