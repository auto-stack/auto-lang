#include "link.h"


bool is_first(struct Connection m) {
    switch (m.tag) {
    case CONNECTION_CONNECTED:
        {
            return true;
        }
        break;
    case CONNECTION_DISCONNECTED:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Connection s = {.tag = CONNECTION_CONNECTED, .as.Connected = 0};
    return is_first(s);
}
