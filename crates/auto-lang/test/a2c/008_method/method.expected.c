#include "method.h"

int modulus(struct Point *s) {
    return s->x * s->x + s->y * s->y;
}

int main(void) {
    struct Point p = {.x = 3, .y = 4};
    int m = modulus(&p);
    printf("%s %d\n", "Modulus:", m);
    return 0;
}
