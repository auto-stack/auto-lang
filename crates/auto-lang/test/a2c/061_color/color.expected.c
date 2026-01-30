#include "color.h"


bool is_first(struct Color m) {
    switch (m.tag) {
    case COLOR_RED:
        {
            return true;
        }
        break;
    case COLOR_BLUE:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Color s = {.tag = COLOR_RED, .as.Red = 0};
    return is_first(s);
}
