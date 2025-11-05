#include <stdio.h>

struct Point {
    int x;
    int y;
};

int modulus(struct Point *p) {
    return p->x * p->x + p->y * p->y;
}

int main(void) {
    struct Point p = {.x = 3, .y = 4};
    printf("%s %d\n", "Modulus:", modulus(&p));
}
