/* Public deterministic check of the typed integer boundary and its validity
 * ABI. Arithmetic modules with nontrivial properties must be rejected. */
#include <stdint.h>
static uint64_t pqc_mul64(uint64_t a, uint64_t b) {
#if defined(PQC_ARITHMETIC_NEGATIVE)
  __CPROVER_assert(a == 0, "nontrivial arithmetic property must not be dropped");
#endif
  return a * b;
}
void mpc_main(void) {
  uint64_t INPUT_A_left, INPUT_A_right;
  uint64_t OUTPUT_product = pqc_mul64(INPUT_A_left, INPUT_A_right);
}
