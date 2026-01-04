#pragma once

#include "pos.h"

struct Src {
    char* content;
    unsigned int len;
    struct Pos pos;
};

char Src_NextChar(struct Src *self);
struct Lexer {
    struct Pos pos;
};
