#include "array_return.h"

int* get_numbers(int* out_size) {
        static int _static_get_numbers[] = {1, 2, 3, 4, 5};
    *out_size = 5;
    return _static_get_numbers;
}

int main(void) {
    int _size_nums;
    int* nums = get_numbers(&_size_nums);
    printf("%d\n", nums[0]);
    return 0;
}
