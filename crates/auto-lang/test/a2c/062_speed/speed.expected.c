#include "speed.h"


bool is_first(struct Speed m) {
    switch (m.tag) {
    case SPEED_FAST:
        {
            return true;
        }
        break;
    case SPEED_SLOW:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Speed s = {.tag = SPEED_FAST, .as.Fast = 0};
    return is_first(s);
}
