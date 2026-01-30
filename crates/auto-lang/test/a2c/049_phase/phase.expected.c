#include "phase.h"


bool is_first(struct Phase m) {
    switch (m.tag) {
    case PHASE_START:
        {
            return true;
        }
        break;
    case PHASE_END:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Phase s = {.tag = PHASE_START, .as.Start = 0};
    return is_first(s);
}
