#include "binary.h"


bool is_first(struct Binary m) {
    switch (m.tag) {
    case BINARY_YES:
        {
            return true;
        }
        break;
    case BINARY_NO:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Binary s = {.tag = BINARY_YES, .as.Yes = 0};
    return is_first(s);
}
