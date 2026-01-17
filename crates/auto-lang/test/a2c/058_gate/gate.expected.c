#include "gate.h"


bool is_first(struct Gate m) {
    switch (m.tag) {
    case GATE_OPEN:
        {
            return true;
        }
        break;
    case GATE_SHUT:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Gate s = {.tag = GATE_OPEN, .as.Open = 0};
    return is_first(s);
}
