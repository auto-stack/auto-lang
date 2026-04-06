#include "file_operations.h"

int main(void) {
    struct File file = open_read("test.txt");
    char* line = File_read_line(file);
    printf("%s\n", "Read: ");
    printf("%s\n", line);
    printf("%s\n", "\n");
    File_close(file);

    struct File out = open_write("output.txt");
    File_write_line(out, "Hello, AutoLang!");
    File_write_line(out, "File I/O test");
    File_flush(out);
    File_close(out);

    struct File app = open_append("output.txt");
    File_write_line(app, "This line is appended");
    File_close(app);

    struct File rw = open_write("rw_test.txt");
    File_write_line(rw, "Line 1");
    File_write_line(rw, "Line 2");
    File_flush(rw);
    File_close(rw);
    return 0;
}
