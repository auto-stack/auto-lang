#include <stdio.h>

struct Point {
    int x;
    int y;
};

int main(void) {
    struct Point p = {.x = 1, .y = 2};
    p.x = 3;
    printf("%s %d %s %d\n", "P: ", p.x, ", ", p.y);
    return 0;
}
