#include "level.h"


bool is_first(struct Level m) {
    switch (m.tag) {
    case LEVEL_HIGH:
        {
            return true;
        }
        break;
    case LEVEL_LOW:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Level s = {.tag = LEVEL_HIGH, .as.High = 0};
    return is_first(s);
}
