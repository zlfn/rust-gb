sdcc -S -Iinc -Isrc -Itests \
    --std-sdcc11 -mz80 --out-fmt-ihx \
    --disable-warning 110 --disable-warning 126 \
    --max-allocs-per-node 2000 --allow-unsafe-read --opt-code-speed \
    --no-std-crt0 --nostdlib --no-xinit-opt \
    -D__HIDDEN__= -D__attribute__\(a\)=  -D__builtin_unreachable\(\)=while\(1\)\; \
	-D__PORT_z80 -D__TARGET_gb \
    ./out/main.c -o ./out/main.asm



