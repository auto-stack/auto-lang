#include "struct.h"



int main(void) {
    struct Point p = {.x = 1, .y = 2};
    p.x = 3;
    printf("%s %d %s %d\n", "P: ", p.x, ", ", p.y);

    struct Circle circle = {.radius = 5.0, .border = 1, .center = {.x = 50, .y = 50}};
    printf("%s %d %s %d %s %d\n", "C: ", circle.center.x, ", ", circle.center.y, ", ", circle.radius);
    return 0;
}
