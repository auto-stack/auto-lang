#include "union.h"


int main(void) {
    union MyUnion my_union = {.i = 42};
    printf("%s %d\n", "int value:", my_union.i);
    return 0;
}
