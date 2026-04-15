#pragma once

#include <stdio.h>

struct Animal {
    char* name;
};

void Animal_Speak(struct Animal *self);
struct Dog {
    char* breed;
    char* name;
};

void Dog_Bark(struct Dog *self);
void Dog_Speak(struct Dog *self);
