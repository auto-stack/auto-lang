#include "io.h"

void say(char* msg) {
    printf("%s\n", msg);
}

void File_Close(struct File *self) {
    fclose(self->file);
}

struct File open(char* path) {
    FILE* f = fopen(path, "r");
    if (f == NULL) {
        printf("Failed to open file");
        exit(1);
    }
    struct File file = {.path = path, .file = f};
    return file;
}
