#pragma once

#include <stdbool.h>
enum TargetKind {
    TARGET_NEAR,
    TARGET_FAR,
};

struct Target {
    enum TargetKind tag;
    union {
        int Near;
        int Far;
    } as;
};
bool is_first(struct Target m);
