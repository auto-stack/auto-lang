#include "may_nested.h"


int double_or_default(struct Option opt, int default) {
    switch (opt.tag) {
    case OPTION_SOME:
        {
            return opt.as.Some * 2;
        }
        break;
    case OPTION_NONE:
        {
            return default;
        }
        break;
    }
    return 0;
}

int get_value(struct Option opt) {
    switch (opt.tag) {
    case OPTION_SOME:
        {
            return opt.as.Some;
        }
        break;
    case OPTION_NONE:
        {
            return 0;
        }
        break;
    }
    return 0;
}

int main(void) {
    struct Option a = {.tag = OPTION_SOME, .as.Some = 10};
    struct Option b = {.tag = OPTION_NONE, .as.None = 0};

    int result1 = double_or_default(a, 0);
    int result2 = double_or_default(b, 100);
    int v1 = get_value(a);
    int v2 = get_value(b);
    int result3 = v1 + v2;

    return result3;
}
