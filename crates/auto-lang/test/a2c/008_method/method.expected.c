#include "method.h"

int Point_Modulus(struct Point *self) {
    return self->x * self->x + self->y * self->y;
}

int main(void) {
    struct Point p = {.x = 3, .y = 4};
    int m = Point_Modulus(&p);
    printf("%s %d\n", "Modulus:", m);
    return 0;
}
