#include "array_return.at.h"

int* get_numbers(int* out_size) {
        static int _static_get_numbers[] = {1, 2, 3, 4, 5};
    *out_size = 5;
    return _static_get_numbers;
}

int main(void) {
    int nums[0] = get_numbers(&_size_nums);
    printf("%d\n", nums[0]);
    return 0;
}
