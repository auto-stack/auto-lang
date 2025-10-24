#include <stdio.h>

int main(void) {
    int x = 5;
    printf("%s %d\n", "x = ", x);

    int *y = &x;
    printf("%s %d\n", "y = ", *y);

    *y += 1;
    printf("%s %d\n", "now x is ", x);

    return 0;
}
