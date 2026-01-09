# Declaration Tests

Tests for Use, Union, UnionField, Tag, TagField, EnumDecl, EnumItem, Alias to_atom() output

## Use - Simple Import

use math

---

use math

## Use - With Items

use math: add, sub

---

use math {
  add
  sub
} 

## Use - C Header

use c <stdio.h>

---

use.c <stdio.h>

## enum - Simple

enum Color {
    Red
    Green
    Blue
}

---

enum Color {
    Red
    Green
    Blue
}

## Tag - With Data

tag Option {
    Some(int)
    None
}

---

tag Option {
    field(Some, int)
    field(None)
}

## Enum - With Values

enum Status {
    Pending = 1
    Approved = 2
    Rejected = 3
}

---

enum Status {
    item (Pending, 1)
    item (Approved, 2)
    item (Rejected, 3)
}
