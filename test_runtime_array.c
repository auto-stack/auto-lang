#include "test_runtime_array.at.h"

int main(void) {
    int size = 10;
    int* arr = malloc(sizeof(int) * size);

    arr[0] = 42;
    arr[1] = 100;

    unknown first = arr[0];
    unknown second = arr[1];

    printf("%s %d\n", "First:", first);
    printf("%s %d\n", "Second:", second);
    printf("%s %d\n", "Array size:", size);
    return 0;
}
