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
void main(void) __ATTRIBUTELIST__((nothrow));
uint32_t add(uint32_t llvm_cbe_left, uint32_t llvm_cbe_right) __ATTRIBUTELIST__((nothrow));
__noreturn void rust_begin_unwind(void* llvm_cbe_info) __ATTRIBUTELIST__((nothrow)) __HIDDEN__;
void color(uint8_t _1, uint8_t _2, uint8_t _3) __ATTRIBUTELIST__((nothrow));
void circle(uint8_t _4, uint8_t _5, uint8_t _6, uint8_t _7) __ATTRIBUTELIST__((nothrow));


/* LLVM Intrinsic Builtin Function Bodies */
static __forceinline uint8_t llvm_add_u8(uint8_t a, uint8_t b) {
  uint8_t r = a + b;
  return r;
}
static __forceinline uint32_t llvm_mul_u32(uint32_t a, uint32_t b) {
  uint32_t r = a * b;
  return r;
}


/* Function Bodies */

void main(void) {
  uint8_t llvm_cbe_x_2e_sroa_2e_0_2e_06__PHI_TEMPORARY;

   /*tail*/ color(3, 0, 0);
  llvm_cbe_x_2e_sroa_2e_0_2e_06__PHI_TEMPORARY = 20;   /* for PHI node */
  goto llvm_cbe_bb3_2e_preheader;

  do {     /* Syntactic loop 'bb3.preheader' to make GCC happy */
llvm_cbe_bb3_2e_preheader:;
  uint8_t llvm_cbe_x_2e_sroa_2e_0_2e_06 = llvm_cbe_x_2e_sroa_2e_0_2e_06__PHI_TEMPORARY;
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 20, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 30, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 40, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 50, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 60, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 70u, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 80u, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 90u, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 100u, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 110u, 15, 0);
   /*tail*/ circle(llvm_cbe_x_2e_sroa_2e_0_2e_06, 120u, 15, 0);
  if ((((uint8_t)llvm_cbe_x_2e_sroa_2e_0_2e_06) < ((uint8_t)((uint8_t)-121)))) {
    llvm_cbe_x_2e_sroa_2e_0_2e_06__PHI_TEMPORARY = (llvm_add_u8(llvm_cbe_x_2e_sroa_2e_0_2e_06, 10));   /* for PHI node */
    goto llvm_cbe_bb3_2e_preheader;
  } else {
    goto llvm_cbe_bb7;
  }

  } while (1); /* end of syntactic loop 'bb3.preheader' */
llvm_cbe_bb7:;
  return;
}


uint32_t add(uint32_t llvm_cbe_left, uint32_t llvm_cbe_right) {
  return (llvm_mul_u32(llvm_cbe_right, llvm_cbe_left));
}


__noreturn void rust_begin_unwind(void* llvm_cbe_info) {
   /*tail*/ rust_begin_unwind(llvm_cbe_info);
  __builtin_unreachable();

}

