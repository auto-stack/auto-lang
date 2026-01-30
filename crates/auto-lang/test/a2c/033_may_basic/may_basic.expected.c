#include "may_basic.h"


bool is_nil(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            return true;
        }
        break;
    case MAYINT_VAL:
        {
            return false;
        }
        break;
    case MAYINT_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

bool is_some(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            return false;
        }
        break;
    case MAYINT_VAL:
        {
            return true;
        }
        break;
    case MAYINT_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

bool is_err(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            return false;
        }
        break;
    case MAYINT_VAL:
        {
            return false;
        }
        break;
    case MAYINT_ERR:
        {
            return true;
        }
        break;
    }
    return false;
}

int unwrap(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            panic("unwrap on nil");
        }
        break;
    case MAYINT_VAL:
        {
            m.as.Val;
        }
        break;
    case MAYINT_ERR:
        {
            panic("unwrap on error");
        }
        break;
    }
    return 0;
}

int unwrap_or(struct MayInt m, int default) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            return default;
        }
        break;
    case MAYINT_VAL:
        {
            return m.as.Val;
        }
        break;
    case MAYINT_ERR:
        {
            return default;
        }
        break;
    }
    return 0;
}

int main(void) {
    struct MayInt x = {.tag = MAYINT_VAL, .as.Val = 42};
    bool check1 = is_some(x);
    int val1 = unwrap(x);

    struct MayInt y = {.tag = MAYINT_NIL, .as.Nil = 0};
    bool check2 = is_nil(y);
    int val2 = unwrap_or(y, 0);

    struct MayInt z = {.tag = MAYINT_ERR, .as.Err = 1};
    bool check3 = is_err(z);

    return val1;
}
