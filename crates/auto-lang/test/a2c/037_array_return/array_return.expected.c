#include "array_return.h"

struct slice_int get_numbers(void) {
    return {1, 2, 3, 4, 5};
}

int main(void) {
    struct slice_int nums = get_numbers();
    printf("%d\n", nums[0]);
    return 0;
}
