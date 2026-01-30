#include "may_bool.h"


bool is_nil(struct MayBool m) {
    switch (m.tag) {
    case MAYBOOL_NIL:
        {
            return true;
        }
        break;
    case MAYBOOL_VAL:
        {
            return false;
        }
        break;
    case MAYBOOL_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

bool is_some(struct MayBool m) {
    switch (m.tag) {
    case MAYBOOL_NIL:
        {
            return false;
        }
        break;
    case MAYBOOL_VAL:
        {
            return true;
        }
        break;
    case MAYBOOL_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

bool unwrap_or(struct MayBool m, bool default) {
    switch (m.tag) {
    case MAYBOOL_NIL:
        {
            return default;
        }
        break;
    case MAYBOOL_VAL:
        {
            return m.as.Val;
        }
        break;
    case MAYBOOL_ERR:
        {
            return default;
        }
        break;
    }
    return false;
}

int main(void) {
    struct MayBool x = {.tag = MAYBOOL_VAL, .as.Val = true};
    bool check1 = is_some(x);
    bool val1 = unwrap_or(x, false);

    struct MayBool y = {.tag = MAYBOOL_NIL, .as.Nil = 0};
    bool check2 = is_nil(y);
    bool val2 = unwrap_or(y, true);

    return val1;
}
