#pragma once

#include <stdbool.h>
enum TypeKind {
    TYPE_A,
    TYPE_B,
};

struct Type {
    enum TypeKind tag;
    union {
        int A;
        int B;
    } as;
};
bool is_first(struct Type m);
