#include "may_string.h"


bool is_nil(struct MayStr m) {
    switch (m.tag) {
    case MAYSTR_NIL:
        {
            return true;
        }
        break;
    case MAYSTR_VAL:
        {
            return false;
        }
        break;
    case MAYSTR_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

bool is_some(struct MayStr m) {
    switch (m.tag) {
    case MAYSTR_NIL:
        {
            return false;
        }
        break;
    case MAYSTR_VAL:
        {
            return true;
        }
        break;
    case MAYSTR_ERR:
        {
            return false;
        }
        break;
    }
    return false;
}

char* unwrap_or(struct MayStr m, char* default) {
    switch (m.tag) {
    case MAYSTR_NIL:
        {
            return default;
        }
        break;
    case MAYSTR_VAL:
        {
            return m.as.Val;
        }
        break;
    case MAYSTR_ERR:
        {
            return default;
        }
        break;
    }
    return "";
}

int main(void) {
    struct MayStr x = {.tag = MAYSTR_VAL, .as.Val = "hello"};
    unknown check1 = is_some(x);
    unknown val1 = unwrap_or(x, "default");

    struct MayStr y = {.tag = MAYSTR_NIL, .as.Nil = 0};
    unknown check2 = is_nil(y);
    unknown val2 = unwrap_or(y, "fallback");

    return val1;
}
