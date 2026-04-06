#include "predicates.h"

bool is_gt_3(int x) {
    return x > 3;
}

bool is_positive(int x) {
    return x > 0;
}

bool is_5(int x) {
    return x == 5;
}

int main(void) {

    struct List list = List_New();
    List_Push(&list, 1);
    List_Push(&list, 2);
    List_Push(&list, 3);
    List_Push(&list, 4);
    List_Push(&list, 5);

    unknown has_gt_3 = List_Iter(&list).any(is_gt_3);
    say(has_gt_3);

    unknown has_lt_0 = List_Iter(&list).any(is_positive);
    say(has_lt_0);

    unknown all_positive = List_Iter(&list).all(is_positive);
    say(all_positive);

    unknown all_gt_3 = List_Iter(&list).all(is_gt_3);
    say(all_gt_3);

    unknown found = List_Iter(&list).find(is_gt_3);
    say(found);
    return 0;
}
