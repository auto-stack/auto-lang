#include "ext_instance_method.at.h"

int int_double(int self) {
    return self + self;
}

int main(void) {
    int x = 21;
    unknown result = int_double(x);
    printf("%d\n", result);
    return 0;
}
