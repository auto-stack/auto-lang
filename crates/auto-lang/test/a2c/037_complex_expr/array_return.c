#include "array_return.at.h"

int* get_numbers(int* out_size) {
    return {1, 2, 3, 4, 5};
}

int main(void) {
    int nums[0] = get_numbers();
    printf("%d\n", nums[0]);
    return 0;
}
