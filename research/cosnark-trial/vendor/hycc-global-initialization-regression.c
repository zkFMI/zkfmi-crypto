/* Deterministic compiler regression: module calls must read initialized global
 * constants, not unconstrained inputs. Exhaustively compare its circuit with
 * this C reference using HyCC's existing create-verifier/CBMC workflow. */
static const unsigned int TABLE[4] = {7, 41, 93, 251};
typedef unsigned int InputA;
typedef unsigned int InputB;
typedef unsigned int Output;

unsigned int read_table(unsigned int x) {
  return TABLE[x & 3] ^ x;
}

Output mpc_main(InputA INPUT_A, InputB INPUT_B) {
  return read_table(INPUT_A) + INPUT_B;
}
