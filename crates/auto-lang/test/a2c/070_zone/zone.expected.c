#include "zone.h"


bool is_first(struct Zone m) {
    switch (m.tag) {
    case ZONE_A:
        {
            return true;
        }
        break;
    case ZONE_B:
        {
            return false;
        }
        break;
    case ZONE_C:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Zone s = {.tag = ZONE_A, .as.A = 0};
    return is_first(s);
}
