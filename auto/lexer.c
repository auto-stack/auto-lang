#include "lexer.h"

char Src_NextChar(struct Src *self) {

    if (self->pos.total >= self->len) {
        return - 1;
    }

    char n = self->content[self->pos.at];

    if (n == '\n') {
        self->pos.line = self->pos.line + 1;
        self->pos.at = 0;
    } else {
        self->pos.at = self->pos.at + 1;
    }
    self->pos.total = self->pos.total + 1;

    return n;
}

