#include "link.h"


bool is_first(struct Link m) {
    switch (m.tag) {
    case LINK_CONNECTED:
        {
            return true;
        }
        break;
    case LINK_DISCONNECTED:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Link s = {.tag = LINK_CONNECTED, .as.Connected = 0};
    return is_first(s);
}
