#include "io.h"

struct File File_Open(struct File *self, char*) {
    void** f = fopen(path, "r");
    
    return {};
}
char* File_ReadText(struct File *self) {
    if (self->file == NULL) {
        
    }

    char* buf = "                                                                                ";
    char* result = fgets(buf, 80, self->file);
    if (result == NULL) {
        
    }
    return result;
}
char* File_ReadLine(struct File *self) {
    if (self->file == NULL) {
        
    }

    char* buf = "                                                                                ";
    char* result = fgets(buf, 80, self->file);
    if (result == NULL) {
        
    }
    return result;
}
int File_ReadChar(struct File *self) {
    if (self->file == NULL) {
        return - 1;
    }
    return fgetc(self->file);
}
int File_ReadBuf(struct File *self, char*, int) {
    if (self->file == NULL) {
        
    }
    return fread(buf, 1, size, self->file);
}
void File_WriteLine(struct File *self, char*) {
    fputs(s, self->file);
    fputs("\n", self->file);
}
void File_Flush(struct File *self) {
    if (self->file != NULL) {
        fflush(self->file);
    }
}
void File_Close(struct File *self) {
    if (self->file != NULL) {
        fclose(self->file);
    }
}

void say(char* msg) {
}
