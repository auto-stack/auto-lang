#pragma once

#include <stdio.h>

typedef struct Reader_vtable {
    void (*read)(void *self);
} Reader_vtable;

struct MyReader {
    char* data;
};

char* MyReader_Read(struct MyReader *self);

