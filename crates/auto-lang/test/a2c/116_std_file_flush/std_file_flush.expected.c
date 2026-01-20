#include "std_file_flush.h"

int main(void) {
    struct File file = File.open("output.txt");
    File_WriteLine(&file, "Hello");
    File_Flush(&file);
    File_Close(&file);
    return 0;
}
