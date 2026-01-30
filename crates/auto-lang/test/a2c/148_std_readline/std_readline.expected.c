#include "std_readline.h"

int main(void) {
    struct File file = File_Open("test_lines.txt");
    char* line1 = File_ReadLine(&file);
    char* line2 = File_ReadLine(&file);
    File_Close(&file);
    return 0;
}
