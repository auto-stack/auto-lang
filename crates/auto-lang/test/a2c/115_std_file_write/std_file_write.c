#include "std_file_write.at.h"

int main(void) {
    struct File file = File.open("output.txt");
    File_WriteLine(&file, "Hello, World!");
    File_Close(&file);
    return 0;
}
