#include "tag_types.h"


float area(struct Shape s) {
    switch (s.tag) {
    case SHAPE_CIRCLE:
        {
            return 3.14 * s.as.Circle * s.as.Circle;
        }
        break;
    case SHAPE_RECT:
        {
            return s.as.Rect * s.as.Rect;
        }
        break;
    }
    return 0.0;
}

int main(void) {
    struct Shape c = {.tag = SHAPE_CIRCLE, .as.Circle = 2.0};
    unknown ca = area(c);
    printf("%s %d\n", "Circle area:", ca);

    struct Shape r = {.tag = SHAPE_RECT, .as.Rect = 3.0};
    unknown ra = area(r);
    printf("%s %d\n", "Rect area:", ra);
    return 0;
}
