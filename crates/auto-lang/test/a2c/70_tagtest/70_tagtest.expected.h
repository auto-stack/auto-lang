#pragma once

#include <stdbool.h>
enum TagTestKind {
    TAGTEST_FIRST,
    TAGTEST_SECOND,
};

struct TagTest {
    enum TagTestKind tag;
    union {
        int First;
        int Second;
    } as;
};

bool is_first(struct TagTest m);
