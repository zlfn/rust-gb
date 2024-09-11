mkdir -p ./out
rustc --emit=llvm-ir -C opt-level=3 -C embed-bitcode=no --target avr-unknown-gnu-atmega328\
	-Cpanic=abort -L dependency=/home/zlfn/rust-z80/target/avr-unknown-gnu-atmega328/release/deps\
	--extern 'noprelude:compiler_builtins=/home/zlfn/rust-z80/target/avr-unknown-gnu-atmega328/release/deps/libcompiler_builtins-7c8cb6a88df6298c.rlib'\
	--extern 'noprelude:core=/home/zlfn/rust-z80/target/avr-unknown-gnu-atmega328/release/deps/libcore-19ea3eec74d36e50.rlib'\
	-Z unstable-options ./src/main.rs -o ./out/main.ll

echo "Rust compile completed"

llvm-cbe --cbe-declare-locals-late ./out/main.ll -o ./out/main.c

echo "LLVM-CBE compile completed"

sed 's/static __forceinline/inline/g' -i ./out/main.c

sed 's/uint8_t\* memset(uint8_t\*, uint32_t, uint16_t);/inline uint8_t\* memset(uint8_t\* dst, uint8_t c, uint16_t sz) {uint8_t \*p = dst; while (sz--) *p++ = c; return dst; }/g' -i ./out/main.c

sed '/__noreturn void rust_begin_unwind(struct l_struct_core_KD__KD_panic_KD__KD_PanicInfo\* llvm_cbe_info)/{:a;N;/__builtin_unreachable/{N;N;d};/  }/b;ba}' -i ./out/main.c

sed 's/volatile//g' -i ./out/main.c

sdcc -S -Iinc -Isrc -Itests \
    --std-sdcc11 -mz80 --out-fmt-ihx \
    --disable-warning 110 --disable-warning 126 \
    --max-allocs-per-node 2000 --allow-unsafe-read --opt-code-speed \
    --no-std-crt0 --nostdlib --no-xinit-opt \
    -D__HIDDEN__= -D__attribute__\(a\)=  -D__builtin_unreachable\(\)=while\(1\)\; \
	-D__PORT_z80 -D__TARGET_gb \
    ./out/main.c -o ./out/main.asm

echo "SDCC compile completed"

./bin/lcc -o ./out/main.gb ./out/main.asm

echo "Gameboy ROM compile completed"
