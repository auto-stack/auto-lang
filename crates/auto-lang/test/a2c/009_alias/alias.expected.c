#include <stdio.h>

int my_add(int a, int b) {
    return a + b;
}

#define my_add add

int main(void) {
    int s = add(3, 5);
    printf("%s %d\n", "Sum:", s);
    return 0;
}
