INPUT_A_left := 0;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 1;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 2;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 4294967295;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 4294967296;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 0;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 1;
OUTPUT_product == 1;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 2;
OUTPUT_product == 2;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 4294967295;
OUTPUT_product == 4294967295;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 4294967296;
OUTPUT_product == 4294967296;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 9223372036854775807;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 1;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 18446744073709551615;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 1;
OUTPUT_product == 2;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 2;
OUTPUT_product == 4;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 4294967295;
OUTPUT_product == 8589934590;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 4294967296;
OUTPUT_product == 8589934592;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 18446744073709551614;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 2;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 18446744073709551614;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 1;
OUTPUT_product == 4294967295;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 2;
OUTPUT_product == 8589934590;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 4294967295;
OUTPUT_product == 18446744065119617025;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 4294967296;
OUTPUT_product == 18446744069414584320;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 9223372032559808513;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967295;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 18446744069414584321;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 1;
OUTPUT_product == 4294967296;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 2;
OUTPUT_product == 8589934592;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 4294967295;
OUTPUT_product == 18446744069414584320;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 4294967296;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 18446744069414584320;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 4294967296;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 18446744069414584320;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 1;
OUTPUT_product == 9223372036854775807;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 2;
OUTPUT_product == 18446744073709551614;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 4294967295;
OUTPUT_product == 9223372032559808513;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 4294967296;
OUTPUT_product == 18446744069414584320;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 1;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775807;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 9223372036854775809;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 1;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 2;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 4294967295;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 4294967296;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 9223372036854775808;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 0;
OUTPUT_product == 0;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 1;
OUTPUT_product == 18446744073709551615;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 2;
OUTPUT_product == 18446744073709551614;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 4294967295;
OUTPUT_product == 18446744069414584321;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 4294967296;
OUTPUT_product == 18446744069414584320;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 9223372036854775807;
OUTPUT_product == 9223372036854775809;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 9223372036854775808;
OUTPUT_product == 9223372036854775808;
OUTPUT_assertions == 1;
---
INPUT_A_left := 18446744073709551615;
INPUT_A_right := 18446744073709551615;
OUTPUT_product == 1;
OUTPUT_assertions == 1;
