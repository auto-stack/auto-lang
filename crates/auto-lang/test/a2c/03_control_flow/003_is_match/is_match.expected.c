#include "is.h"

int main(void) {
    int x = 10;

    switch (x) {
    case 0:
        {
            printf("%s\n", "X is ZERO");
        }
        break;
    case 1:
        {
            printf("%s\n", "X is ONE");
        }
        break;
    default:
        {
            printf("%s\n", "X is Large");
        }
        break;
    }
    return 0;
}
