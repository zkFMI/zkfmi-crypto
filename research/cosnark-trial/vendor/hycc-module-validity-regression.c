/* Compiler-only regression: validity must flow through two module calls and
 * respect the root call guard. Compile with --unwind 4 --emit-validity
 * --bool checked_count --bool nested_count --merge. */
unsigned char checked_count(unsigned char count) {
  unsigned char result = 0;
  unsigned char i;
  for (i = 0; i < count; ++i) ++result;
  __CPROVER_assert(count != 1, "module assertion regression");
  __CPROVER_assume(count != 2);
  return result;
}

unsigned char nested_count(unsigned char count) {
  return checked_count(count);
}

void mpc_main(void) {
  unsigned char INPUT_A_count;
  unsigned char OUTPUT_count = 255;
  if (INPUT_A_count != 255) OUTPUT_count = nested_count(INPUT_A_count);
}
