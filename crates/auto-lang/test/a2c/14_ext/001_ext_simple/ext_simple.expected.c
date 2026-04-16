#include "ext_simple.h"

int int_get_default(void) {
    return 42;
}

int main(void) {
    unknown x = int_get_default(int);
    printf("%d\n", x);
    return 0;
}
