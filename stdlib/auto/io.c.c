#include "io.h"

File File_Open(struct File *self, str) {
    struct FILE* f = fopen(path, "r");
    
    return {};
}
str File_ReadText(struct File *self) {
    if (self->file == NULL) {
        
    }

    char* buf = "                                                                                ";
    char* result = fgets(buf, 80, self->file);
    if (result == NULL) {
        
    }
    return result;
}
str File_ReadLine(struct File *self) {
    if (self->file == NULL) {
        
    }

    char* buf = "                                                                                ";
    char* result = fgets(buf, 80, self->file);
    if (result == NULL) {
        
    }
    return result;
}
void File_WriteLine(struct File *self, str) {
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
