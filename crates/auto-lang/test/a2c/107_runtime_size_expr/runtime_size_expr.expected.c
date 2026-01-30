#include "runtime_size_expr.h"

int main(void) {
    int x = 5;
    int y = 3;
    int* arr = malloc(sizeof(int) * (x + y));

    arr[0] = 1;
    arr[1] = 2;
    arr[2] = 3;

    unknown len = arr.len();
    unknown elem = arr[0];

    printf("%s %d\n", "Length:", len);
    printf("%s %d\n", "First element:", elem);
    return 0;
}
