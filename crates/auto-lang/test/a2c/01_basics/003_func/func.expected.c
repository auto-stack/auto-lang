#include "func.h"

int add(int a, int b) {
    return a + b;
}

int main(void) {
    int result = add(5, 3);
    printf("%s %d\n", "The result is:", result);
    return 0;
}
