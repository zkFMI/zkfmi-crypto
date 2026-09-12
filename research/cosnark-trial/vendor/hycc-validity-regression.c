/* Development regression for exported execution validity, not a proof circuit.
 * Compile with --unwind 4 --emit-validity. A legal short loop, a false assertion,
 * a false assumption and an exhausted loop bound must remain distinguishable. */
void mpc_main(void) {
  unsigned char INPUT_A_count;
  unsigned char OUTPUT_count = 0;
  unsigned char i;
  for (i = 0; i < INPUT_A_count; ++i) ++OUTPUT_count;
  __CPROVER_assert(INPUT_A_count != 1, "explicit assertion regression");
  __CPROVER_assume(INPUT_A_count != 2);
}
