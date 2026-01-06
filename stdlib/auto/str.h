#pragma once

#include <stdio.h>

struct sstr {
    unknown size;
    char[0] data;
};

void sstr_Print(struct sstr *self);
struct dstr {
    unknown size;
    char[0] data;
};

void dstr_Print(struct dstr *self);
struct vstr {
    unknown size;
    char* data;
};
