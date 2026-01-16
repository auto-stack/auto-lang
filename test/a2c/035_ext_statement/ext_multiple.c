#include "ext_multiple.at.h"

int int_double(int self) {
    return self + self;
}

int int_triple(int self) {
    return self + self + self;
}

int main(void) {
    int x = 5;
    unknown d = int_double(x);
    unknown t = int_triple(x);
    printf("%d\n", d);
    printf("%d\n", t);
    return 0;
}
