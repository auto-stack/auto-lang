#include "tagtest.h"

bool is_first(struct TagTest m) {
    switch (m.tag) {
    case TAGTEST_FIRST:
        {
            return true;
        }
        break;
    case TAGTEST_SECOND:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct TagTest t = {.tag = TAGTEST_FIRST, .as.First = 0};
    return is_first(t);
}
