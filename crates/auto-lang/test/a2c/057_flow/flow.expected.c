#include "flow.h"


bool is_first(struct Flow m) {
    switch (m.tag) {
    case FLOW_IN:
        {
            return true;
        }
        break;
    case FLOW_OUT:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Flow s = {.tag = FLOW_IN, .as.In = 0};
    return is_first(s);
}
