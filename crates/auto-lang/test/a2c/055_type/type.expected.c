#include "type.h"


bool is_first(struct Type m) {
    switch (m.tag) {
    case TYPE_A:
        {
            return true;
        }
        break;
    case TYPE_B:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Type s = {.tag = TYPE_A, .as.A = 0};
    return is_first(s);
}
