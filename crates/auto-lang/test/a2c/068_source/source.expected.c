#include "source.h"


bool is_first(struct Source m) {
    switch (m.tag) {
    case SOURCE_INTERNAL:
        {
            return true;
        }
        break;
    case SOURCE_EXTERNAL:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Source s = {.tag = SOURCE_INTERNAL, .as.Internal = 0};
    return is_first(s);
}
