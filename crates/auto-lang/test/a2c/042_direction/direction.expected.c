#include "direction.h"


bool is_first(struct Direction m) {
    switch (m.tag) {
    case DIRECTION_UP:
        {
            return true;
        }
        break;
    case DIRECTION_DOWN:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Direction s = {.tag = DIRECTION_UP, .as.Up = 0};
    return is_first(s);
}
