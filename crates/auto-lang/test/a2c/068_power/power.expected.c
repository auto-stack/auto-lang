#include "power.h"


bool is_first(struct Power m) {
    switch (m.tag) {
    case POWER_ON:
        {
            return true;
        }
        break;
    case POWER_OFF:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Power s = {.tag = POWER_ON, .as.On = 0};
    return is_first(s);
}
