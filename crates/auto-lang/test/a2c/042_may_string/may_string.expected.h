#pragma once

#include <stdbool.h>
enum MayStrKind {
    MAYSTR_NIL,
    MAYSTR_VAL,
    MAYSTR_ERR,
};

struct MayStr {
    enum MayStrKind tag;
    union {
        int Nil;
        str Val;
        int Err;
    } as;
};
bool is_nil(struct MayStr m);
bool is_some(struct MayStr m);
char* unwrap_or(struct MayStr m, char* default);
