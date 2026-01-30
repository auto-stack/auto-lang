#include "file_operations.at.h"

int main(void) {
    struct File file = open_read("test.txt");
    char* line = File_ReadLine(&file);
    printf("%s\n", "Read: ");
    printf("%s\n", line);
    printf("%s\n", "\n");
    File_Close(&file);

    struct File out = open_write("output.txt");
    File_WriteLine(&out, "Hello, AutoLang!");
    File_WriteLine(&out, "File I/O test");
    File_Flush(&out);
    File_Close(&out);

    struct File app = open_append("output.txt");
    File_WriteLine(&app, "This line is appended");
    File_Close(&app);

    struct File rw = open_write("rw_test.txt");
    File_WriteLine(&rw, "Line 1");
    File_WriteLine(&rw, "Line 2");
    File_Flush(&rw);
    File_Close(&rw);
    return 0;
}
