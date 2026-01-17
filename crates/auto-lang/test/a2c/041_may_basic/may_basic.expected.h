#pragma once

#include <stdbool.h>
enum MayIntKind {
    MAYINT_NIL,
    MAYINT_VAL,
    MAYINT_ERR,
};

struct MayInt {
    enum MayIntKind tag;
    union {
        int Nil;
        int Val;
        int Err;
    } as;
};
bool is_nil(struct MayInt m);
bool is_some(struct MayInt m);
bool is_err(struct MayInt m);
int unwrap(struct MayInt m);
int unwrap_or(struct MayInt m, int default);
