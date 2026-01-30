#include "null_coalesce.h"

int test_coalesce_int(void) {
    int x = 10;
    unknown y = x != NULL ? x : 0;
    return y;
}

int test_coalesce_with_nil(void) {
    unknown x = NULL;
    unknown y = x != NULL ? x : 42;
    return y;
}

int main(void) {
    int a = test_coalesce_int();
    int b = test_coalesce_with_nil();
    return 0;
}
