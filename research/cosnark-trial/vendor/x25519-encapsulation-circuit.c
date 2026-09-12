/* Domain/byte-order adapter only. Unmodified BearSSL m31 X25519 core at the
 * commit and MIT license recorded in encryption-reference-PROVENANCE.json.
 * Circuit shared-secret output and all inputs remain private proof wires.
 */
#include "inner.h"
/* Optional typed multiplication boundary for the same portable MUL31 core.
 * Both operands are converted to uint64_t exactly as in upstream MUL31. The
 * compiler can retain this single integer product instead of bit-blasting it.
 * It is not a new elliptic-curve or cryptographic implementation. */
#if defined(PQC_TYPED_MULTIPLY)
static uint64_t pqc_mul64(uint64_t a, uint64_t b) { return a * b; }
#undef MUL31
#define MUL31(x, y) pqc_mul64((uint64_t)(x), (uint64_t)(y))
#endif

/* HyCC's documented module ABI recognizes pointer writes only on parameters
 * named OUTPUT* or INOUT*. The selected f255 helpers assign their destination d
 * before reading it. Token-renaming d retains all original core expressions.
 * reduce_final_f255 reads its initial destination and MUST remain inline, not
 * be selected as a module with this output-only annotation.
 * The same annotation is used in native cross-implementation known answers.
 */
#define d OUTPUT_d
#include "ec_c25519_m31.c"
#undef d
#include "../codec/ccopy.c"

static int encapsulate(const unsigned char seed[32],
                       const unsigned char recipient[32],
                       unsigned char ephemeral[32],
                       unsigned char shared[32]) {
  unsigned char scalar_be[32];
  unsigned char nonzero = 0;
  int i;
  /* BearSSL's EC API accepts a big-endian scalar, unlike RFC7748 seed bytes.
   * Its upstream core applies the RFC7748 clamping itself. */
  for (i = 0; i < 32; ++i) scalar_be[i] = seed[31-i];
  api_mulgen(ephemeral, scalar_be, 32, BR_EC_curve25519);
  memcpy(shared, recipient, 32);
  if (api_mul(shared, 32, scalar_be, 32, BR_EC_curve25519) != 1) return 0;
  for (i = 0; i < 32; ++i) nonzero |= shared[i];
  return nonzero != 0;
}

#if defined(PQC_REFERENCE_KAT)
#include <stdio.h>
int main(void) {
  unsigned char seed[32], peer_seed[32], recipient[32], ephemeral[32], shared[32];
  int i;
  /* Nonuniform bytes make a mistaken scalar-endianness adapter observable. */
  for (i = 0; i < 32; ++i) { seed[i] = i+1; peer_seed[i] = 199-i; }
  api_mulgen(recipient, peer_seed, 32, BR_EC_curve25519);
  if (!encapsulate(seed, recipient, ephemeral, shared)) return 1;
  /* Public deterministic cross-implementation vector, not runtime keys. */
  if (fwrite(seed, 1, 32, stdout) != 32 || fwrite(recipient, 1, 32, stdout) != 32 ||
      fwrite(ephemeral, 1, 32, stdout) != 32 || fwrite(shared, 1, 32, stdout) != 32) return 2;
  return 0;
}
#else
void mpc_main(void) {
  unsigned char INPUT_A_seed[32];
  unsigned char INPUT_A_recipient[32];
  unsigned char OUTPUT_ephemeral[32] = {0};
  unsigned char OUTPUT_shared[32] = {0};
  int OUTPUT_valid = encapsulate(INPUT_A_seed, INPUT_A_recipient,
                                OUTPUT_ephemeral, OUTPUT_shared);
}
#endif
