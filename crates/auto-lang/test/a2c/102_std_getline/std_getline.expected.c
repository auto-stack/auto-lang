#include <stdio.h>
#include <stdlib.h>

int main(void) {
    char* lineptr = "";
    int n = 0;

    printf("%s\n", "Enter a line of text: ");

    int charsRead = getline(&lineptr, &n, stdin);

    if (charsRead != -1) {
        printf("%s %s\n", "You entered: ", lineptr); // lineptr already contains the newline
    } else {
        printf("%s\n", "Error reading line");
    }

    free(lineptr);
    return 0;
}
