#include "test_operators.at.h"

int test_null_coalesce(void) {
    int x = 10;
    unknown y = x != NULL ? x : 0;
    return y;
}
