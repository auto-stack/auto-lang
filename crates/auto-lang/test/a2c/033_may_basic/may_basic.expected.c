#include "may_basic.h"


bool is_nil(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            int x = m.as.Nil;
            {
                return true;
            }
            break;
        }
    case MAYINT_VAL:
        {
            int x = m.as.Val;
            {
                return false;
            }
            break;
        }
    case MAYINT_ERR:
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

bool is_some(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            int x = m.as.Nil;
            {
                return false;
            }
            break;
        }
    case MAYINT_VAL:
        {
            int x = m.as.Val;
            {
                return true;
            }
            break;
        }
    case MAYINT_ERR:
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

bool is_err(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            int x = m.as.Nil;
            {
                return false;
            }
            break;
        }
    case MAYINT_VAL:
        {
            int x = m.as.Val;
            {
                return false;
            }
            break;
        }
    case MAYINT_ERR:
        {
            int x = m.as.Err;
            {
                return true;
            }
            break;
        }
    }
    return false;
}

int unwrap(struct MayInt m) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            int x = m.as.Nil;
            {
                panic("unwrap on nil");
            }
            break;
        }
    case MAYINT_VAL:
        {
            int v = m.as.Val;
            {
                m.as.Val;
            }
            break;
        }
    case MAYINT_ERR:
        {
            int x = m.as.Err;
            {
                panic("unwrap on error");
            }
            break;
        }
    }
    return 0;
}

int unwrap_or(struct MayInt m, int default) {
    switch (m.tag) {
    case MAYINT_NIL:
        {
            int x = m.as.Nil;
            {
                return default;
            }
            break;
        }
    case MAYINT_VAL:
        {
            int v = m.as.Val;
            {
                return m.as.Val;
            }
            break;
        }
    case MAYINT_ERR:
        {
            int x = m.as.Err;
            {
                return default;
            }
            break;
        }
    }
    return 0;
}

int main(void) {
    struct MayInt x = {.tag = MAYINT_VAL, .as.Val = 42};
    unknown check1 = is_some(m.as.Err);
    unknown val1 = unwrap(m.as.Err);

    struct MayInt y = {.tag = MAYINT_NIL, .as.Nil = 0};
    unknown check2 = is_nil(y);
    unknown val2 = unwrap_or(y, 0);

    struct MayInt z = {.tag = MAYINT_ERR, .as.Err = 1};
    unknown check3 = is_err(z);

    return val1;
}
