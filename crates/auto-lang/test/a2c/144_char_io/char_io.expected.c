#include "char_io.h"

int main(void) {
    struct File file = open_write("char_test.txt");
    File_Putc(&file, 65);
    File_Putc(&file, 66);
    File_Putc(&file, 67);
    File_Close(&file);

    struct File file2 = open_write("string_test.txt");
    File_Puts(&file2, "Hello from puts!");
    File_Close(&file2);
    return 0;
}
