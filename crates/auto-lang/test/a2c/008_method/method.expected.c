#include "method.h"

int modulus(struct Point *s) {
    return s->x * s->x + s->y * s->y;
}

int main(void) {
    struct Point p = {.x = 3, .y = 4};
    printf("%s %d\n", "Modulus:", modulus(&p));
    return 0;
}
