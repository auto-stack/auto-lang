#include "result.h"


bool is_first(struct Result m) {
    switch (m.tag) {
    case RESULT_PASS:
        {
            return true;
        }
        break;
    case RESULT_FAIL:
        {
            return false;
        }
        break;
    }
    return false;
}

int main(void) {
    struct Result s = {.tag = RESULT_PASS, .as.Pass = 0};
    return is_first(s);
}
