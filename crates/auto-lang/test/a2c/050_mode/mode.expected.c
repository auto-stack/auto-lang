#include "mode.h"


bool is_first(struct Mode m) {
    switch (m.tag) {
    case MODE_READ:
        {
            return true;
        }
        break;
    case MODE_WRITE:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Mode s = {.tag = MODE_READ, .as.Read = 0};
    return is_first(s);
}
