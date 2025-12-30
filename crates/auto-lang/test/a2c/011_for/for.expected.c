#include "for.h"

int main(void) {
    for (int j = 0; j < 10; j++) {
        printf("%d\n", j);
    }

    int arr[3] = {1, 2, 3};
    for (int i = 0; i < 3; i++) {
        int n = arr[i];
        printf("%d\n", n);
    }
    return 0;
}
