// Minimal uthash-compatible header
// Based on https://github.com/troydhanson/uthash
// This is a simplified version for AutoLang HashMap/HashSet

#ifndef AUTO_UTHASH_H
#define AUTO_UTHASH_H

#include <string.h>

// uthash handle structure
typedef struct UT_hash_handle {
    struct UT_hash_table *table;
    struct UT_hash_handle *hh_prev;
    struct UT_hash_handle *hh_next;
    void *key;
    unsigned keylen;
} UT_hash_handle;

// uthash table structure
typedef struct UT_hash_table {
    UT_hash_handle *hh_head;
    unsigned num_buckets;
    unsigned log2_num_buckets;
} UT_hash_table;

// Macros for hash operations (simplified)
#define HASH_FIND_STR(head, findstr, out) \
    do { \
        (out) = NULL; \
        if ((head)) { \
            UT_hash_handle *_hh; \
            for (_hh = (head); _hh != NULL; _hh = _hh->hh_next) { \
                if (strcmp(((void*)_hh)->key, (findstr)) == 0) { \
                    (out) = (void*)_hh; \
                    break; \
                } \
            } \
        } \
    } while(0)

#define HASH_ADD_STR(head, keyfield, add) \
    do { \
        UT_hash_handle *_hh_add = &((add)->hh); \
        _hh_add->key = ((void*)(add))->key; \
        _hh_add->keylen = (unsigned)strlen((char*)((add)->key)); \
        if (!(head)->hh_head) { \
            (head)->hh_head = _hh_add; \
            _hh_add->hh_prev = NULL; \
            _hh_add->hh_next = NULL; \
        } else { \
            _hh_add->hh_next = (head)->hh_head; \
            _hh_add->hh_prev = NULL; \
            (head)->hh_head->hh_prev = _hh_add; \
            (head)->hh_head = _hh_add; \
        } \
    } while(0)

#define HASH_DEL(head, delptr) \
    do { \
        UT_hash_handle *_hh_del = &((delptr)->hh); \
        if (_hh_del->hh_prev) { \
            _hh_del->hh_prev->hh_next = _hh_del->hh_next; \
        } else { \
            (head) = _hh_del->hh_next; \
        } \
        if (_hh_del->hh_next) { \
            _hh_del->hh_next->hh_prev = _hh_del->hh_prev; \
        } \
    } while(0)

#define HASH_ITER(hh, head, el, tmp) \
    for((el) = (head), (tmp) = ((el) ? ((el)->hh_next) : NULL); \
        (el) != NULL; \
        (el) = (tmp), (tmp) = ((el) ? ((el)->hh_next) : NULL))

#endif
