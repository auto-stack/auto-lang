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

    list_void*_void* list = List.new();
    list.push(1);
    list.push(2);
    list.push(3);
    list.push(4);
    list.push(5);

    unknown has_gt_3 = list.iter().any(is_gt_3);
    say(has_gt_3);

    unknown has_lt_0 = list.iter().any(is_positive);
    say(has_lt_0);

    unknown all_positive = list.iter().all(is_positive);
    say(all_positive);

    unknown all_gt_3 = list.iter().all(is_gt_3);
    say(all_gt_3);

    unknown found = list.iter().find(is_gt_3);
    say(found);
    return 0;
}
