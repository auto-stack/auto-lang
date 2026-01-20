#pragma once

struct dstr {
    List data;
};

dstr dstr_New(struct dstr *self);
dstr dstr_FromByte(struct dstr *self, char);
dstr dstr_FromBytes(struct dstr *self, char, char);
int dstr_Len(struct dstr *self);
bool dstr_IsEmpty(struct dstr *self);
char dstr_Get(struct dstr *self, int);
void dstr_Push(struct dstr *self, char);
char dstr_Pop(struct dstr *self);
int dstr_Set(struct dstr *self, int, char);
void dstr_Insert(struct dstr *self, int, char);
char dstr_Remove(struct dstr *self, int);
void dstr_Clear(struct dstr *self);
void dstr_Reserve(struct dstr *self, int);
