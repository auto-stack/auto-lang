#include "path.h"


bool is_first(struct Path m) {
    switch (m.tag) {
    case PATH_NORTH:
        {
            return true;
        }
        break;
    case PATH_SOUTH:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Path s = {.tag = PATH_NORTH, .as.North = 0};
    return is_first(s);
}
