#include "ext_prop_shorthand.at.h"


int Point_sum(unknown self) {
    return self->x + self->y;
}

int main(void) {
    struct Point p = {.x = 3, .y = 4};
    unknown result = Point_sum(p);
    printf("%d\n", result);
    return 0;
}
