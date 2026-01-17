#include "side.h"


bool is_first(struct Side m) {
    switch (m.tag) {
    case SIDE_LEFT:
        {
            return true;
        }
        break;
    case SIDE_RIGHT:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Side s = {.tag = SIDE_LEFT, .as.Left = 0};
    return is_first(s);
}
