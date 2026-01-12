#include "advanced_io.h"

int main(void) {
    struct File file = open_write("seek_test.txt");

    File_Puts(&file, "Hello");

    File_Seek(&file, 0, 0);

    int pos = File_Tell(&file);

    File_Rewind(&file);

    File_Close(&file);
    return 0;
}
