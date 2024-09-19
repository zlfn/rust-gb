#include <stdint.h>
void gprint(void* _1) __nonbanked;
void gotogxy(uint8_t _2, uint8_t _3) __sdcccall(0);
void color(uint8_t _4, uint8_t _5, uint8_t _6) __sdcccall(0);

//https://github.com/rust-lang/rust/blob/master/library/core/src/panicking.rs
void _ZN4core9panicking5panic17h03fc2a0e21329e27E(void* message, uint16_t _1, void* _2) {
	gotogxy(0, 0);
	color(0, 3, 0);
	gprint("PANIC occured");
	gotogxy(0, 1);
	color(1, 3, 0);
	gprint(message);
}

void _ZN4core9panicking18panic_bounds_check17h7c8da0e27d354709E(uint16_t index, uint16_t len, void* _1) {
	gotogxy(0, 0);
	color(0, 3, 0);
	gprint("PANIC occured");
	gotogxy(0, 1);
	color(1, 3, 0);
	gprint("index out of bounds");
}
