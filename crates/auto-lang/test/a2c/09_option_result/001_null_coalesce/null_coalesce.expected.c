#include "null_coalesce.h"

int test_coalesce_int(void) {
    int x = 10;
    int y = x != NULL ? x : 0;
    return y;
}

int test_coalesce_with_nil(void) {
    unknown x = NULL;
    int y = x != NULL ? x : 42;
    return y;
}

int main(void) {
    unknown a = test_coalesce_int();
    unknown b = test_coalesce_with_nil();
    return 0;
}
