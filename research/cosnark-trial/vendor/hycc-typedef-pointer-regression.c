/* Public input regression for canonical typedef/tag resolution only. */
#include <stdint.h>
typedef struct { int16_t coefficients[4]; unsigned offset; } InputState;
typedef struct { int16_t coefficients[4]; unsigned offset; } OutputState;
static void typed_module(const InputState *input, OutputState *OUTPUT_result) {
  unsigned i;
  for (i = 0; i < 4; ++i) OUTPUT_result->coefficients[i] = input->coefficients[i];
  OUTPUT_result->offset = input->offset + 1;
}
void mpc_main(void) {
  uint16_t INPUT_A_value;
  unsigned INPUT_A_offset;
  InputState input = {{(int16_t)INPUT_A_value, -7, 123, 0}, INPUT_A_offset};
  OutputState output;
  typed_module(&input, &output);
  uint16_t OUTPUT_value = (uint16_t)output.coefficients[0];
  unsigned OUTPUT_offset = output.offset;
}
