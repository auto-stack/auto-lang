#include "may_bool.h"


bool is_nil(struct MayBool m) {
    switch (m.tag) {
    case MAYBOOL_NIL:
        {
            int x = m.as.Nil;
            {
                return true;
            }
            break;
        }
    case MAYBOOL_VAL:
        {
            bool x = m.as.Val;
            {
                return false;
            }
            break;
        }
    case MAYBOOL_ERR:
        {
            int x = m.as.Err;
            {
                return false;
            }
            break;
        }
    }
    return false;
}

bool is_some(struct MayBool m) {
    switch (m.tag) {
    case MAYBOOL_NIL:
        {
            int x = m.as.Nil;
            {
                return false;
            }
            break;
        }
    case MAYBOOL_VAL:
        {
            bool x = m.as.Val;
            {
                return true;
            }
            break;
        }
    case MAYBOOL_ERR:
        {
            int x = m.as.Err;
            {
                return false;
            }
            break;
        }
    }
    return false;
}

bool unwrap_or(struct MayBool m, bool default) {
    switch (m.tag) {
    case MAYBOOL_NIL:
        {
            int x = m.as.Nil;
            {
                return default;
            }
            break;
        }
    case MAYBOOL_VAL:
        {
            bool v = m.as.Val;
            {
                return m.as.Val;
            }
            break;
        }
    case MAYBOOL_ERR:
        {
            int x = m.as.Err;
            {
                return default;
            }
            break;
        }
    }
    return false;
}

int main(void) {
    struct MayBool x = {.tag = MAYBOOL_VAL, .as.Val = true};
    unknown check1 = is_some(m.as.Err);
    unknown val1 = unwrap_or(m.as.Err, false);

    struct MayBool y = {.tag = MAYBOOL_NIL, .as.Nil = 0};
    unknown check2 = is_nil(y);
    unknown val2 = unwrap_or(y, true);

    return val1;
}
