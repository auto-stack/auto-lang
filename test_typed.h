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
enum MyMay_intKind {
    MYMAY_INT_NONE,
    MYMAY_INT_SOME,
};

struct MyMay_int {
    enum MyMay_intKind tag;
    union {
        void none;
        int some;
    } as;
};
