= AutoDown Comprehensive Example

This document demonstrates _various_ *formatting* features.

== Text Formatting

AutoDown supports *bold text*, _italic text_, and `inline code`.

You can also create #link("https://example.com")[links] and embed images:

#image("https://example.com/logo.png", alt: "AutoDown Logo")

== Lists

=== Unordered Lists

- First item with *bold*
- Second item with _italic_
- Third item with `code`

=== Ordered Lists

+ Step one
+ Step two
+ Step three

== Code Blocks

```rust
fn main() {
    println!("Hello, AutoDown!");
}
```

== Math

Inline math: $E = mc^2$

Block math:

$ sum_i^n f(i) = integral_0^infinity f(x) dx $

== Blockquotes

#quote[
This is a blockquote.
It can contain multiple lines.
]

---

== Advanced Features

=== Conditional Content

#if show_advanced {
This content only appears when show_advanced is true.
} else {
This is the default content.
}

=== Loop Content

#for item in items {
- Item: #{item}
}

=== Component Call

#Card(title: "Example", width: 100)[
This is the card content with #{variable} interpolation.
]

== Table Example

#table(
  columns: 3,
  [
    Name], [
    Type], [
    Description],
  [id], [int], [Unique identifier],
  [name], [str], [Display name],
  [active], [bool], [Active status],
)

---

Thank you for reading!
