#pragma once

struct HashSet {
};

HashSet HashSet_New(struct HashSet *self);
void HashSet_Insert(struct HashSet *self, *char);
int HashSet_Contains(struct HashSet *self, *char);
void HashSet_Remove(struct HashSet *self, *char);
int HashSet_Size(struct HashSet *self);
void HashSet_Clear(struct HashSet *self);
void HashSet_Drop(struct HashSet *self);
