#include "tristate.h"


bool is_first(struct TriState m) {
    switch (m.tag) {
    case TRISTATE_ON:
        {
            return true;
        }
        break;
    case TRISTATE_OFF:
        {
            return false;
        }
        break;
    case TRISTATE_UNKNOWN:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct TriState s = {.tag = TRISTATE_ON, .as.On = 0};
    return is_first(s);
}
