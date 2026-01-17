#pragma once

#include <stdbool.h>
enum MayBoolKind {
    MAYBOOL_NIL,
    MAYBOOL_VAL,
    MAYBOOL_ERR,
};

struct MayBool {
    enum MayBoolKind tag;
    union {
        int Nil;
        bool Val;
        int Err;
    } as;
};
bool is_nil(struct MayBool m);
bool is_some(struct MayBool m);
bool unwrap_or(struct MayBool m, bool default);
