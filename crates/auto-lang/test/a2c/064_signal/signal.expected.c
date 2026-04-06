#include "signal.h"


bool is_first(struct Signal m) {
    switch (m.tag) {
    case SIGNAL_HIGH:
        {
            return true;
        }
        break;
    case SIGNAL_LOW:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    unknown s = {.tag = SIGNAL_HIGH, .as.High = 0};
    return is_first(s);
}
