/* Domain adapter only. The algorithm core is the pinned mlkem-native source
 * named in encryption-reference-PROVENANCE.json, under its MIT option.
 * Inputs and the shared-secret output are PRIVATE proof wires. Exporting a
 * circuit does not authorize publishing these values or accepting its result.
 * A bounded compiler run must retain/check unwinding assertions; a truncated
 * rejection sampler is not ML-KEM and must never be accepted as a circuit.
 */
#define MLK_CONFIG_PARAMETER_SET 768
#define MLK_CONFIG_NAMESPACE_PREFIX mlkem
#define MLK_CONFIG_NO_KEYPAIR_API
#define MLK_CONFIG_NO_DECAPS_API
#define MLK_CONFIG_NO_RANDOMIZED_API
#define MLK_CONFIG_NO_ASM
#define MLK_CONFIG_SERIAL_FIPS202_ONLY
/* The circuit compiler has no assembly barrier. Use the upstream-supported
 * zeroization hook; the portable native reference writes through volatile. */
#define MLK_CONFIG_CUSTOM_ZEROIZE
#include <stddef.h>
#include <stdint.h>
#if defined(PQC_SAMPLER_VALUE_MODULE)
typedef struct {
  int16_t coefficients[256];
  unsigned offset;
  unsigned length;
  uint8_t bytes[504];
} PqcSamplerInput;
typedef struct {
  int16_t coefficients[256];
  unsigned offset;
} PqcSamplerOutput;
static unsigned pqc_sampler_adapter(int16_t *r, unsigned target, unsigned offset,
                                    const uint8_t *buf, unsigned buflen);
#endif
static void mlk_zeroize(void *ptr, size_t len) {
  volatile unsigned char *p = (volatile unsigned char *)ptr;
  size_t i;
  for (i = 0; i < len; ++i) p[i] = 0;
}
/* HyCC module ABI annotations only: the selected Keccak permutation and NTT
 * routines read and overwrite their complete state/polynomial arguments.
 * Do not select output-only r routines under this INOUT annotation. */
#define state INOUT_state
#define r INOUT_r
#include "mlkem_native.c"
#undef r
#undef state

#if defined(PQC_SAMPLER_VALUE_MODULE)
/* Complete-struct pointer ABI: the original sampler below is unchanged.
 * Secret offsets are ordinary input bits, never C pointer specialization
 * constants. The unused coefficient tail is zero-filled because the original
 * sampler contract defines only the prefix before its returned offset. */
static void pqc_sampler_module(const PqcSamplerInput *input, PqcSamplerOutput *OUTPUT_result) {
  memcpy(OUTPUT_result->coefficients, input->coefficients, sizeof OUTPUT_result->coefficients);
#if !defined(PQC_REFERENCE_KAT)
  __CPROVER_assert(input->offset <= 256 && input->length <= 504 && input->length % 3 == 0,
                  "fixed ML-KEM sampler module bounds");
#endif
  OUTPUT_result->offset = mlk_rej_uniform_c(OUTPUT_result->coefficients, 256, input->offset,
                                          input->bytes, input->length);
}

static unsigned pqc_sampler_adapter(int16_t *r, unsigned target, unsigned offset,
                                    const uint8_t *buf, unsigned buflen) {
  PqcSamplerInput input = {0};
  PqcSamplerOutput output;
  unsigned i;
  /* Preserve the generic upstream behavior outside this adapter's exact
   * ML-KEM-768 matrix shape. The configured encapsulation uses target=256. */
  if (target != 256 || offset > 256 || buflen > 504)
    return mlk_rej_uniform_c(r, target, offset, buf, buflen);
  for (i = 0; i < 256; ++i) input.coefficients[i] = i < offset ? r[i] : 0;
  input.offset = offset;
  input.length = buflen;
  memcpy(input.bytes, buf, buflen);
  pqc_sampler_module(&input, &output);
  memcpy(r, output.coefficients, sizeof output.coefficients);
  return output.offset;
}
#endif

#if defined(PQC_REFERENCE_KAT)
#include <stdio.h>
#include "expected_test_vectors.h"
int main(void) {
  unsigned char ciphertext[1088], shared[32];
  if (mlkem_enc_derand(ciphertext, shared, test_vector_pk, test_vector_m) != 0 ||
      memcmp(ciphertext, test_vector_ct, sizeof(ciphertext)) != 0 ||
      memcmp(shared, test_vector_ss, sizeof(shared)) != 0) return 1;
  /* Public upstream known-answer material only, never runtime entropy. */
  if (fwrite(test_vector_m, 1, 32, stdout) != 32 ||
      fwrite(test_vector_pk, 1, 1184, stdout) != 1184 ||
      fwrite(shared, 1, 32, stdout) != 32 ||
      fwrite(ciphertext, 1, 1088, stdout) != 1088) return 2;
  return 0;
}
#else
void mpc_main(void) {
  /* The compiler's documented local INPUT_/OUTPUT_ convention avoids a
   * typedef-tag frontend limitation. These are circuit I/O, not a C runner. */
  unsigned char INPUT_A_recipient[1184];
  unsigned char INPUT_A_coins[32];
  unsigned char OUTPUT_ciphertext[1088] = {0};
  unsigned char OUTPUT_shared_secret[32] = {0};
  int OUTPUT_status = mlkem_enc_derand(OUTPUT_ciphertext, OUTPUT_shared_secret,
                                     INPUT_A_recipient, INPUT_A_coins);
}
#endif
