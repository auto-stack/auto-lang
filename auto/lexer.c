#include "lexer.h"

char next_char(struct Src *s) {
    unsigned int p = s->pos.spos;
    s->pos.spos = s->pos.spos + 1;
    return s->content[p];
}

