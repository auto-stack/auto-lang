#include "std_readline.h"

int main(void) {
    struct File f = File.open("Cargo.toml");

    char* s = File_ReadLine(&f);
    printf("%s\n", s);

    File_Close(&f);
    return 0;
}
