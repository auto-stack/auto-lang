#include "mode2.h"


bool is_first(struct Mode2 m) {
    switch (m.tag) {
    case MODE2_AUTO:
        {
            return true;
        }
        break;
    case MODE2_MANUAL:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Mode2 s = {.tag = MODE2_AUTO, .as.Auto = 0};
    return is_first(s);
}
