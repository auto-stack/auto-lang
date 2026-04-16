#include "ext_static_method.h"

int int_default(void) {
    return 42;
}

int main(void) {
    unknown x = int_default(int);
    printf("%d\n", x);
    return 0;
}
