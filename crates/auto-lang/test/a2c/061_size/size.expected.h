#pragma once

#include <stdbool.h>
enum SizeKind {
    SIZE_BIG,
    SIZE_SMALL,
};

struct Size {
    enum SizeKind tag;
    union {
        int Big;
        int Small;
    } as;
};
bool is_first(struct Size m);
