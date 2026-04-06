#include "runtime_size_var.h"

int main(void) {
    int size = 10;
    int* arr = malloc(sizeof(int) * size);

    arr[0] = 42;
    arr[1] = 100;

    int first = arr[0];
    int second = arr[1];

    printf("%s %d\n", "First:", first);
    printf("%s %d\n", "Second:", second);
    return 0;
}
