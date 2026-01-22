#include "runtime_size_var.at.h"

int get_size(void) {
    return 10;
}

int main(void) {
    int* arr = malloc(sizeof(int) * get_size());
    arr[0] = 42;
    return arr[0];
}
