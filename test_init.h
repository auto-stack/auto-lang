#pragma once

enum MyMayKind {
    MYMAY_NONE,
    MYMAY_SOME,
};

struct MyMay {
    enum MyMayKind tag;
    union {
        void none;
        T some;
    } as;
};
