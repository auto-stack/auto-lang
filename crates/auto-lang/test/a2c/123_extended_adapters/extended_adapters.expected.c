#include "extended_adapters.h"

int main(void) {

    struct List list1 = List_New();
    List_Push(&list1, 1);
    List_Push(&list1, 2);
    List_Push(&list1, 3);
    List_Push(&list1, 4);
    List_Push(&list1, 5);

    struct List list2 = List_New();
    List_Push(&list2, 10);
    List_Push(&list2, 20);
    List_Push(&list2, 30);
    List_Push(&list2, 40);
    List_Push(&list2, 50);

    unknown limited = List_Iter(&list1).limit(3);
    say(limited.count());

    unknown skipped = List_Iter(&list1).skip(2);
    say(skipped.count());

    unknown enumerated = List_Iter(&list1).enumerate();
    say(enumerated.count());

    unknown zipped = List_Iter(&list1).zip(List_Iter(&list2));
    say(zipped.count());

    unknown chained = List_Iter(&list1).chain(List_Iter(&list2));
    say(chained.count());
    return 0;
}
