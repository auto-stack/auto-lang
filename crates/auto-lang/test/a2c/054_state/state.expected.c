#include "state.h"


bool is_first(struct State m) {
    switch (m.tag) {
    case STATE_OPEN:
        {
            return true;
        }
        break;
    case STATE_CLOSED:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct State s = {.tag = STATE_OPEN, .as.Open = 0};
    return is_first(s);
}
