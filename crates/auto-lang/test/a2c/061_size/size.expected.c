#include "size.h"


bool is_first(struct Size m) {
    switch (m.tag) {
    case SIZE_BIG:
        {
            return true;
        }
        break;
    case SIZE_SMALL:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Size s = {.tag = SIZE_BIG, .as.Big = 0};
    return is_first(s);
}
