#include <stdio.h>

union MyUnion {
    int i;
    float f;
    char c;
};

int main(void) {
    union MyUnion my_union = { .i = 42 };
    printf("%s %d\n", "int value:", my_union.i);
}
