#pragma once

#include "token.h"

struct Src {
    char* content;
    struct Pos pos;
};

char next_char(struct Src *s);
struct Lexer {
    struct Pos pos;
};
