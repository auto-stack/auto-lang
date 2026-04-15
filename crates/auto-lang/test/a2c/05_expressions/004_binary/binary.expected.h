#pragma once

#include <stdbool.h>
enum BinaryKind {
    BINARY_YES,
    BINARY_NO,
};

struct Binary {
    enum BinaryKind tag;
    union {
        int Yes;
        int No;
    } as;
};
bool is_first(struct Binary m);
