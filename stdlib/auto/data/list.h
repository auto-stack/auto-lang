#pragma once

struct List {
};

List List_New(struct List *self);
int List_Len(struct List *self);
bool List_IsEmpty(struct List *self);
char List_Get(struct List *self, int);
void List_Push(struct List *self, char);
char List_Pop(struct List *self);
int List_Set(struct List *self, int, char);
void List_Insert(struct List *self, int, char);
char List_Remove(struct List *self, int);
void List_Clear(struct List *self);
void List_Reserve(struct List *self, int);
void List_Drop(struct List *self);
