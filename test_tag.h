#pragma once

enum MayKind {
    MAY_NIL,
    MAY_VAL,
};

struct May {
    enum MayKind tag;
    union {
        <unknown> nil;
        T val;
    } as;
};
