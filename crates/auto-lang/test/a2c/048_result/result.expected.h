#pragma once

#include <stdbool.h>
enum ResultKind {
    RESULT_PASS,
    RESULT_FAIL,
};

struct Result {
    enum ResultKind tag;
    union {
        int Pass;
        int Fail;
    } as;
};
bool is_first(struct Result m);
