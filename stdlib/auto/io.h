#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

void say(char* msg);
struct File {
    char* path;
    FILE* file;
};

str File_ReadText(struct File *self);
str File_ReadLine(struct File *self);
void File_WriteLine(struct File *self, str);
void File_Close(struct File *self);
void File_Flush(struct File *self);
int File_Getc(struct File *self);
void File_Putc(struct File *self, int);
void File_Ungetc(struct File *self, int);
int File_Read(struct File *self, [byte]0, int, int);
int File_Write(struct File *self, [byte]0, int, int);
str File_Gets(struct File *self, [byte]0);
void File_Puts(struct File *self, str);
int File_Seek(struct File *self, int, int);
int File_Tell(struct File *self);
void File_Rewind(struct File *self);
bool File_IsEof(struct File *self);
bool File_HasError(struct File *self);
void File_ClearError(struct File *self);
str File_ReadAll(struct File *self);
<unknown> File_WriteLines(struct File *self, [str]0);
struct File open(char* path);
struct File open_read(char* path);
struct File open_write(char* path);
struct File open_append(char* path);
