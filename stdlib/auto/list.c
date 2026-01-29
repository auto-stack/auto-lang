#include "list.h"

list_void*_void* List_New(struct List *self) {
}
int List_Len(struct List *self) {
}
bool List_IsEmpty(struct List *self) {
}
int List_Capacity(struct List *self) {
}
void* List_Get(struct List *self, int) {
}
void List_Push(struct List *self, void*) {
}
void* List_Pop(struct List *self) {
}
int List_Set(struct List *self, int, void*) {
}
void List_Clear(struct List *self) {
}
void List_Drop(struct List *self) {
}
listiter_void*_void* List_Iter(struct List *self) {
}

listiter_void*_void* ListIter_New(struct ListIter *self, list_void*_void**) {
}
may_void* ListIter_Next(struct ListIter *self) {
}
