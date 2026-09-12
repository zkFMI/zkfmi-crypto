/* Byte/domain adapter for the exact zkfmi-crypto envelope. Algorithm cores
 * retain the pinned BearSSL algorithm expressions (MIT). The optional
 * bearssl-fixed-envelope-dispatch.patch specializes virtual call targets to
 * this adapter's fixed SHA-256/AES-CTR/GHASH suite; it changes no arithmetic.
 * KEM components
 * are proved by their own circuits and joined by equality of PRIVATE wires.
 * This file alone is not a proof of correct encapsulation or note validity.
 */
/* HyCC by-reference module ABI: both helpers read AND write their complete
 * state. Token annotations do not change the pinned arithmetic expressions. */
#if defined(PQC_ENVELOPE_MODULES)
#define val INOUT_val
#endif
#include "hash/sha2small.c"
#if defined(PQC_ENVELOPE_MODULES)
#undef val
#endif
#include "codec/dec32be.c"
#include "codec/enc32be.c"
#include "mac/hmac.c"
#include "kdf/hkdf.c"
#if defined(PQC_ENVELOPE_MODULES)
#define q INOUT_q
#endif
#include "symcipher/aes_ct.c"
#if defined(PQC_ENVELOPE_MODULES)
#undef q
#endif
#include "symcipher/aes_ct_enc.c"
#include "symcipher/aes_ct_ctr.c"
#include "hash/ghash_ctmul32.c"
#include "aead/gcm.c"

static int encrypt_envelope(
    const unsigned char x_shared[32], const unsigned char pq_shared[32],
    const unsigned char kem_ciphertext[1120], const unsigned char nonce[12],
    const unsigned char recipient[1216], const unsigned char context[32],
    const unsigned char plaintext[232], unsigned char key[32],
    unsigned char ciphertext[232], unsigned char tag[16]) {
  static const unsigned char kem_domain[] = "ZKFMI:HYBRID-KEM:v1";
  static const unsigned char kem_info[] = "ZKFMI:HYBRID-KEM:SESSION:v1";
  static const unsigned char domain[] = "ZKFMI:RECIPIENT-ENVELOPE:v1";
  static const unsigned char suite[4] = {2, 1, 0, 1};
  static const unsigned char version[2] = {0, 1};
  static const unsigned char purpose[2] = {0, 1}; /* NoteOpening */
  unsigned char aad[sizeof(domain)-1+2+4+2+32+1216];
  size_t at = 0;
  br_hkdf_context hkdf;
  br_aes_ct_ctr_keys aes;
  br_gcm_context gcm;
  br_hkdf_init(&hkdf, &br_sha256_vtable, kem_domain, sizeof(kem_domain)-1);
  br_hkdf_inject(&hkdf, x_shared, 32);
  br_hkdf_inject(&hkdf, pq_shared, 32);
  br_hkdf_inject(&hkdf, kem_ciphertext, 1120);
  br_hkdf_inject(&hkdf, suite, 4);
  br_hkdf_flip(&hkdf);
  if (br_hkdf_produce(&hkdf, kem_info, sizeof(kem_info)-1, key, 32) != 32) return 0;
  memcpy(aad+at, domain, sizeof(domain)-1); at += sizeof(domain)-1;
  memcpy(aad+at, version, 2); at += 2;
  memcpy(aad+at, suite, 4); at += 4;
  memcpy(aad+at, purpose, 2); at += 2;
  memcpy(aad+at, context, 32); at += 32;
  memcpy(aad+at, recipient, 1216);
  memcpy(ciphertext, plaintext, 232);
  br_aes_ct_ctr_init(&aes, key, 32);
  br_gcm_init(&gcm, &aes.vtable, br_ghash_ctmul32);
  br_gcm_reset(&gcm, nonce, 12);
  br_gcm_aad_inject(&gcm, aad, sizeof(aad));
  br_gcm_flip(&gcm);
  br_gcm_run(&gcm, 1, ciphertext, 232);
  br_gcm_get_tag(&gcm, tag);
  return 1;
}

#if defined(PQC_REFERENCE_KAT)
#include <stdio.h>
int main(void) {
  unsigned char x[32], pq[32], ct[1120], nonce[12], recipient[1216], context[32];
  unsigned char plaintext[232], key[32], ciphertext[232], tag[16];
  memset(x, 11, sizeof(x)); memset(pq, 22, sizeof(pq));
  memset(ct, 33, sizeof(ct)); memset(nonce, 44, sizeof(nonce));
  memset(recipient, 55, sizeof(recipient)); memset(context, 66, sizeof(context));
  memset(plaintext, 77, sizeof(plaintext));
  if (!encrypt_envelope(x, pq, ct, nonce, recipient, context, plaintext, key, ciphertext, tag)) return 1;
  /* Public constant arithmetic vector only. ct/recipient are not valid KEM
   * objects; this tests the exact combiner/AAD/AEAD boundary, not full KEM. */
  if (fwrite(key, 1, 32, stdout) != 32 || fwrite(ciphertext, 1, 232, stdout) != 232 ||
      fwrite(tag, 1, 16, stdout) != 16) return 2;
  return 0;
}
#else
void mpc_main(void) {
  unsigned char INPUT_A_x[32], INPUT_A_pq[32], INPUT_A_ct[1120];
  unsigned char INPUT_A_nonce[12], INPUT_A_recipient[1216], INPUT_A_context[32];
  unsigned char INPUT_A_plaintext[232];
  unsigned char OUTPUT_key[32] = {0}, OUTPUT_ciphertext[232] = {0}, OUTPUT_tag[16] = {0};
  unsigned char OUTPUT_valid[1];
  OUTPUT_valid[0] = encrypt_envelope(INPUT_A_x, INPUT_A_pq, INPUT_A_ct, INPUT_A_nonce,
      INPUT_A_recipient, INPUT_A_context, INPUT_A_plaintext, OUTPUT_key, OUTPUT_ciphertext, OUTPUT_tag);
}
#endif
