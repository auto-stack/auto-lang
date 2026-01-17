#include "error_propagate.h"

int test_propagate(void) {
    int x = 10;
    unknown y = x;
    return y;
}

int main(void) {
    int result = test_propagate();
    return 0;
}
