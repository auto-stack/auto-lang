#include "bang_operator.h"

int main(void) {
    struct List list = List_New();
    List_Push(&list, 1);
    List_Push(&list, 2);
    List_Push(&list, 3);

    unknown collected = List_Iter(&list).collect();

    say(collected.len());
    return 0;
}
