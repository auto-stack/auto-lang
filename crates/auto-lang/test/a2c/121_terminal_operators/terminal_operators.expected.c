#include "terminal_operators.h"

int add(int a, int b) {
    return a + b;
}

void print_item(int x) {
    say(x);
}

int main(void) {

    struct List list = List_New();
    List_Push(&list, 1);
    List_Push(&list, 2);
    List_Push(&list, 3);

    unknown sum = List_Iter(&list).reduce(0, add);
    say(sum);

    unknown count = List_Iter(&list).count();
    say(count);

    List_Iter(&list).for_each(print_item);
    return 0;
}
