#include "extended_adapters.h"

int main(void) {

    list_void*_void* list1 = List.new();
    list1.push(1);
    list1.push(2);
    list1.push(3);
    list1.push(4);
    list1.push(5);

    list_void*_void* list2 = List.new();
    list2.push(10);
    list2.push(20);
    list2.push(30);
    list2.push(40);
    list2.push(50);

    unknown limited = list1.iter().limit(3);
    say(limited.count());

    unknown skipped = list1.iter().skip(2);
    say(skipped.count());

    unknown enumerated = list1.iter().enumerate();
    say(enumerated.count());

    unknown zipped = list1.iter().zip(list2.iter());
    say(zipped.count());

    unknown chained = list1.iter().chain(list2.iter());
    say(chained.count());
    return 0;
}
