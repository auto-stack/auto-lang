#include "alias.h"

int my_add(int a, int b) {
    return a + b;
}

#define add my_add

int main(void) {
    int s = add(3, 5);
    printf("%s %d\n", "Sum:", s);
    return 0;
}
