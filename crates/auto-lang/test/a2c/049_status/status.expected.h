#pragma once

#include <stdbool.h>
enum StatusKind {
    STATUS_ACTIVE,
    STATUS_INACTIVE,
};

struct Status {
    enum StatusKind tag;
    union {
        int Active;
        int Inactive;
    } as;
};
bool is_first(struct Status m);
