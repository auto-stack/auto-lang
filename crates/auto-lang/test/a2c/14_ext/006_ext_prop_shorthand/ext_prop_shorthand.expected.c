#include "ext_prop_shorthand.h"

int Point_Sum(struct Point *self) {
    return self->x + self->y;
}

int main(void) {
    unknown p = {.x = 3, .y = 4};
    unknown result = p.sum();
    printf("%d\n", result);
    return 0;
}
