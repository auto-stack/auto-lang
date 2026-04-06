#include "inheritance.h"

void Animal_Speak(struct Animal *self) {
    printf("%s\n", "Animal sound");
}

void Dog_Bark(struct Dog *self) {
    printf("%s\n", "Woof!");
}
void Dog_Speak(struct Dog *self) {
    printf("%s\n", "Animal sound");
}

int main(void) {
    unknown dog = Dog();
    dog.name = "Buddy";
    dog.breed = "Labrador";

    dog.speak();
    dog.bark();

    printf("%s\n", "Test passed");
    return 0;
}
