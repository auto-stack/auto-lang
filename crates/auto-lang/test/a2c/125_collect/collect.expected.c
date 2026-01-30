#include "collect.h"

int double(int x) {
    return x * 2;
}

bool is_gt_3(int x) {
    return x > 3;
}

int main(void) {

    struct List list = List_New();
    List_Push(&list, 1);
    List_Push(&list, 2);
    List_Push(&list, 3);
    List_Push(&list, 4);
    List_Push(&list, 5);

    unknown mapped = List_Iter(&list).map(double).collect();
    say(mapped.len());

    unknown filtered = List_Iter(&list).filter(is_gt_3).collect();
    say(filtered.len());

    unknown chained = List_Iter(&list).map(double).filter(is_gt_3).collect();
    say(chained.len());
    return 0;
}
