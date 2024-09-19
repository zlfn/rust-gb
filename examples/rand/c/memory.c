#include <stdint.h>

void* memset(void* s, uint32_t value, uint16_t size) {
	unsigned char *p = (unsigned char *)s;

	for(int i = 0; i < size; i++) {
		*p = value;
		p++;
	}

	return s;
}
