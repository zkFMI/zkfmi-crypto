INPUT_A_value := 0;
INPUT_A_offset := 0;
OUTPUT_value == 0;
OUTPUT_offset == 1;
OUTPUT_assertions == 1;
---
INPUT_A_value := 1;
INPUT_A_offset := 4294967295;
OUTPUT_value == 1;
OUTPUT_offset == 0;
OUTPUT_assertions == 1;
---
INPUT_A_value := 32767;
INPUT_A_offset := 2147483647;
OUTPUT_value == 32767;
OUTPUT_offset == 2147483648;
OUTPUT_assertions == 1;
---
INPUT_A_value := 32768;
INPUT_A_offset := 1;
OUTPUT_value == 32768;
OUTPUT_offset == 2;
OUTPUT_assertions == 1;
---
INPUT_A_value := 65535;
INPUT_A_offset := 4294967295;
OUTPUT_value == 65535;
OUTPUT_offset == 0;
OUTPUT_assertions == 1;
