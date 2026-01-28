#pragma once

#include "c/stdio.h"

struct File {
    char* path;
    void** file;
};

struct File File_Open(struct File *self, char*);
char* File_ReadText(struct File *self);
char* File_ReadLine(struct File *self);
int File_ReadChar(struct File *self);
int File_ReadBuf(struct File *self, char*, int);
void File_WriteLine(struct File *self, char*);
void File_Flush(struct File *self);
void File_Close(struct File *self);
void say(char* msg);
