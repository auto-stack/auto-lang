#include "status.h"


bool is_first(struct Status m) {
    switch (m.tag) {
    case STATUS_ACTIVE:
        {
            return true;
        }
        break;
    case STATUS_INACTIVE:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Status s = {.tag = STATUS_ACTIVE, .as.Active = 0};
    return is_first(s);
}
