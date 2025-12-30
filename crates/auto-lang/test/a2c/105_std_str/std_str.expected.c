#include "std_str.h"

int main(void) {
    struct sstr s1 = {.size = 5, .data = "Hello"};
    sstr_Print(s1);
    return 0;
}
