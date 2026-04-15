#include "bool.h"

bool test_bool_return(void) {
    return 1 == 1;
}

int main(void) {
    unknown result = test_bool_return();
    return 0;
}
