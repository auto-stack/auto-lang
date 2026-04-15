#include "error_propagate.h"

int test_propagate(void) {
    int x = 10;
    int y = x;
    return y;
}

int main(void) {
    unknown result = test_propagate();
    return 0;
}
