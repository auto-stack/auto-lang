#include "io.h"

void say(char* msg) {
    printf("%s\n", msg);
}

str File_ReadText(struct File *self) {
    char* buf = "                                        ";
    fgets(buf, 40, self->file);
    return buf;
}
str File_ReadLine(struct File *self) {
    char* buf = "                                                                                ";
    fgets(buf, 80, self->file);
    return buf;
}
void File_WriteLine(struct File *self, str) {
    fputs(s, self->file);
    fputs("\n", self->file);
}
void File_Close(struct File *self) {
    fclose(self->file);
}
void File_Flush(struct File *self) {
    fflush(self->file);
}
int File_Getc(struct File *self) {
    return fgetc(self->file);
}
void File_Putc(struct File *self, int) {
    fputc(c, self->file);
}
void File_Ungetc(struct File *self, int) {
    ungetc(c, self->file);
}
int File_Read(struct File *self, [byte]0, int, int) {
    return fread(buf, size, count, self->file);
}
int File_Write(struct File *self, [byte]0, int, int) {
    return fwrite(buf, size, count, self->file);
}
str File_Gets(struct File *self, [byte]0) {
    fgets(buf, 80, self->file);
    return buf;
}
void File_Puts(struct File *self, str) {
    fputs(s, self->file);
}
int File_Seek(struct File *self, int, int) {
    return fseek(self->file, offset, origin);
}
int File_Tell(struct File *self) {
    return ftell(self->file);
}
void File_Rewind(struct File *self) {
    rewind(self->file);
}
bool File_IsEof(struct File *self) {
    int result = feof(self->file);
    if (result == 0) {
        false;
    } else {
        true;
    }
    return false;
}
bool File_HasError(struct File *self) {
    int result = ferror(self->file);
    if (result == 0) {
        false;
    } else {
        true;
    }
    return false;
}
void File_ClearError(struct File *self) {
    clearerr(self->file);
}
str File_ReadAll(struct File *self) {
}
<unknown> File_WriteLines(struct File *self, [str]0) {
}

struct File open(char* path) {
    open_read(path);
    return {};
}

struct File open_read(char* path) {
    FILE* f = fopen(path, "r");
    if (f == NULL) {
        printf("Failed to open file");
        exit(1);
    }
    struct File file = {.path = path, .file = f};
    return file;
}

struct File open_write(char* path) {
    FILE* f = fopen(path, "w");
    if (f == NULL) {
        printf("Failed to open file");
        exit(1);
    }
    struct File file = {.path = path, .file = f};
    return file;
}

struct File open_append(char* path) {
    FILE* f = fopen(path, "a");
    if (f == NULL) {
        printf("Failed to open file");
        exit(1);
    }
    struct File file = {.path = path, .file = f};
    return file;
}
