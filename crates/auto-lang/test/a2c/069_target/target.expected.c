#include "target.h"


bool is_first(struct Target m) {
    switch (m.tag) {
    case TARGET_NEAR:
        {
            return true;
        }
        break;
    case TARGET_FAR:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Target s = {.tag = TARGET_NEAR, .as.Near = 0};
    return is_first(s);
}
