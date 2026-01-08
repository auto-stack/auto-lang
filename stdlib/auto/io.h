#pragma once

#include <stdio.h>
#include <stdlib.h>

void say(char* msg);
struct File {
    char* path;
    FILE* file;
};

str File_ReadText(struct File *self);
void File_Close(struct File *self);
struct File open(char* path);
