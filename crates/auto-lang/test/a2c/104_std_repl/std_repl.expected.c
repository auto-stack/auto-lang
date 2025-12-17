#include "std_repl.h"

int main(void) {
    char* lineptr = NULL;
    int n = 0;

    while (1) {
        printf("%s\n", "Enter a line of text: ");

        int rn = getline(&lineptr, &n, stdin);

        if (rn != -1) {
            if (lineptr[0] == 'q') {
                break;
            } else {
                printf("%s %s\n", "You entered:", lineptr);
            }
        } else {
            printf("%s\n", "Error reading line");
        }
    }

    free(lineptr);
    return 0;
}
