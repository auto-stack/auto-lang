# Declaration Tests

Tests for Use, Union, UnionField, Tag, TagField, EnumDecl, EnumItem, Alias to_atom() output

## Use - Simple Import

use math

---

use(path("math"))

## Use - With Items

use math::{add, sub}

---

use(path("math"), items(str("add"), str("sub")))

## Use - C Header

use c <stdio.h>

---

use(kind("c"), path("stdio.h"))

## Use - Rust Import

use rust std::collections

---

use(kind("rust"), path("std::collections"))

## Union - Simple

union Maybe {
    Some(int)
    None
}

---

union(name("Maybe")) {
    field(name("Some"), type(int))
    field(name("None"), type(void))
}

## Union - Multiple Fields

union Shape {
    Circle(radius: float, center: Point)
    Rectangle(width: float, height: float)
    Triangle(points: [Point; 3])
}

---

union(name("Shape")) {
    field(name("Circle")) {
        member(name("radius"), type(float))
        member(name("center"), type(Point))
    }
    field(name("Rectangle")) {
        member(name("width"), type(float))
        member(name("height"), type(float))
    }
    field(name("Triangle")) {
        member(name("points"), type(array(Point, 3)))
    }
}

## Tag - Simple

tag Color {
    Red
    Green
    Blue
}

---

tag(name("Color")) {
    field(name("Red"), type(void))
    field(name("Green"), type(void))
    field(name("Blue"), type(void))
}

## Tag - With Data

tag Option {
    Some(int)
    None
}

---

tag(name("Option")) {
    field(name("Some"), type(int))
    field(name("None"), type(void))
}

## TagField - Simple

Red

---

field(name("Red"), type(void))

## TagField - With Type

Some(int)

---

field(name("Some"), type(int))

## Enum - Without Values

enum Direction {
    North
    South
    East
    West
}

---

enum(name("Direction")) {
    item(name("North"))
    item(name("South"))
    item(name("East"))
    item(name("West"))
}

## Enum - With Values

enum Status {
    Pending = 1
    Approved = 2
    Rejected = 3
}

---

enum(name("Status")) {
    item(name("Pending"), value(1))
    item(name("Approved"), value(2))
    item(name("Rejected"), value(3))
}

## EnumItem - Without Value

North

---

item(name("North"))

## EnumItem - With Value

Pending = 1

---

item(name("Pending"), value(1))

## Alias - Simple

alias Handle = int

---

alias(name("Handle"), target(int))

## Alias - Complex Type

alias StringMap = Map<str, str>

---

alias(name("StringMap"), target(Map(str, str)))

## UnionField - Simple

Some(int)

---

field(name("Some"), type(int))

## UnionField - With Members

Circle(radius: float, center: Point)

---

field(name("Circle")) {
    member(name("radius"), type(float))
    member(name("center"), type(Point))
}
