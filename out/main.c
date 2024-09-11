/* Provide Declarations */
#include <stdint.h>
#ifndef __cplusplus
typedef unsigned char bool;
#endif

#ifdef _MSC_VER
#define __builtin_unreachable() __assume(0)
#endif
#ifdef _MSC_VER
#define __noreturn __declspec(noreturn)
#else
#define __noreturn __attribute__((noreturn))
#endif
#ifndef _MSC_VER
#define __forceinline __attribute__((always_inline)) inline
#endif

#if defined(__GNUC__)
#define __HIDDEN__ __attribute__((visibility("hidden")))
#endif

#if defined(__GNUC__)
#define  __ATTRIBUTELIST__(x) __attribute__(x)
#else
#define  __ATTRIBUTELIST__(x)  
#endif

#ifdef _MSC_VER  /* Can only support "linkonce" vars with GCC */
#define __attribute__(X)
#endif



/* Global Declarations */

/* Types Declarations */

/* Function definitions */

/* Types Definitions */

/* Function Declarations */
int main(void) __ATTRIBUTELIST__((nothrow));
uint8_t add(uint8_t llvm_cbe_left, uint8_t llvm_cbe_right) __ATTRIBUTELIST__((nothrow));
__noreturn void rust_begin_unwind(void* llvm_cbe_info) __ATTRIBUTELIST__((nothrow)) __HIDDEN__;
void delay(uint16_t _1) __ATTRIBUTELIST__((nothrow));


/* LLVM Intrinsic Builtin Function Bodies */
inline uint8_t llvm_add_u8(uint8_t a, uint8_t b) {
  uint8_t r = a + b;
  return r;
}


/* Function Bodies */

int main(void) {
   /*tail*/ delay(10000);
  return 42;
}


uint8_t add(uint8_t llvm_cbe_left, uint8_t llvm_cbe_right) {
  return (llvm_add_u8(llvm_cbe_right, llvm_cbe_left));
}


__noreturn void rust_begin_unwind(void* llvm_cbe_info) {
   /*tail*/ rust_begin_unwind(llvm_cbe_info);
  __builtin_unreachable();

}

