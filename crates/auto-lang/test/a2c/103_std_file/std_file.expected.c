#include "std_file.h"

int main(void) {
    struct File file = File.open("Cargo.toml");
    char* s = File_ReadText(&file);
    printf("%s\n", s);
    File_Close(&file);
    return 0;
}
