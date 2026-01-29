#include "std_file_flush.h"

int main(void) {
    struct File file = File_Open("output.txt");
    File_WriteLine(&file, "Hello");
    File_Flush(&file);
    File_Close(&file);
    return 0;
}
