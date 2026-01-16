#include "ext_static_method.at.h"

int int_default(void) {
    return 42;
}

int main(void) {
    unknown x = int.default();
    printf("%d\n", x);
    return 0;
}
