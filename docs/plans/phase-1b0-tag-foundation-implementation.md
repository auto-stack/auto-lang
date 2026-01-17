# Phase 1b.0: Tag Type Foundation Implementation

**Status**: 🔄 IN PROGRESS
**Started**: 2025-01-17
**Estimated Duration**: 2-3 weeks

## Objective

Complete tag type support in parser, evaluator, and transpiler to enable implementation of `tag May<T>`.

## Current Status (2025-01-17)

### ✅ Working
- Tag definition parsing: `tag Atom { Int int, Float float }`
- Tag AST structures: `Tag`, `TagField`, `TagCover`, `TagUncover`
- Tag metadata storage in universe

### ❌ Missing
- Tag construction evaluation: `Atom.Int(5)` → runtime error
- Tag pattern matching evaluation
- Tag method calls (static and instance)
- Tag type C transpilation

## Implementation Plan

### Step 1: Tag Construction Evaluation (Week 1, Days 1-2)

**File**: `crates/auto-lang/src/eval.rs`

**Problem**: When evaluating `Atom.Int(5)`, the parser generates:
```
(call (bina (name Atom) (op .) (name Int)) (args (int 5)))
```

But the evaluator doesn't handle this pattern.

**Solution**: Add tag construction evaluation in `eval_call()`:

```rust
// In eval_call(), detect Tag.Field(args) pattern
Expr::Call(box Expr::Bina(left, Op::Dot, right), args) => {
    // Check if left side is a tag type
    match self.lookup_type(left.as_name()) {
        Some(Type::Tag(tag)) => {
            // right is variant name, args is payload
            self.eval_tag_construction(tag, right.as_name(), args)
        }
        _ => // existing method call logic
    }
}
```

**New Function**: `eval_tag_construction()`

```rust
fn eval_tag_construction(
    &mut self,
    tag: &Tag,
    variant: &Name,
    args: &Vec<Expr>
) -> AutoResult<Value> {
    // Find variant in tag definition
    let field = tag.fields.iter()
        .find(|f| f.name == *variant)
        .ok_or_else(|| EvalError::UndefinedVariant)?;

    // Evaluate payload expression
    let payload = if args.len() == 1 {
        self.eval_expr(&args[0])?
    } else if args.len() == 0 {
        Value::Nil  // No payload
    } else {
        return Err(EvalError::TooManyArguments);
    };

    // Create tag value as a Node
    let mut node = auto_val::Node::new(tag.name.as_str());
    node.set_prop("variant", Value::str(variant.as_str()));
    node.set_prop("payload", payload);

    Ok(Value::Node(Rc::new(RefCell::new(node))))
}
```

**Tests**: `test/a2c/040_tag_types/construction.at`

### Step 2: Tag Pattern Matching Evaluation (Week 1, Days 3-4)

**File**: `crates/auto-lang/src/eval.rs`

**Problem**: Pattern matching like `is x { Atom.Int(i) => i }` doesn't work.

**Solution**: Add tag pattern matching in `eval_is()`:

```rust
// In eval_is(), match TagCover patterns
Expr::Is(var, eqs) => {
    let value = self.eval_expr(var)?;

    for eq in eqs {
        match &eq.expr {
            Cover::Tag(tag_cover) => {
                // Match tag variant
                if self.matches_tag_variant(&value, tag_cover)? {
                    // Bind payload variables
                    if let Some(elem) = &tag_cover.elem {
                        self.define(elem.as_str(), Meta::Var(elem.clone()));
                    }
                    return self.eval_expr(&eq.body);
                }
            }
            _ => // existing pattern matching
        }
    }

    Ok(Value::Nil)
}
```

**New Function**: `matches_tag_variant()`

```rust
fn matches_tag_variant(
    &mut self,
    value: &Value,
    cover: &TagCover
) -> AutoResult<bool> {
    match value {
        Value::Node(node) => {
            let node = node.borrow();
            let tag_name = node.get_prop("tag").unwrap_or(Value::Nil);
            let variant = node.get_prop("variant").unwrap_or(Value::Nil);

            // Check if tag and variant match
            tag_name.to_string() == cover.kind.to_string()
                && variant.to_string() == cover.tag.to_string()
        }
        _ => Ok(false),
    }
}
```

**Tests**: `test/a2c/040_tag_types/pattern_matching.at`

### Step 3: Tag Methods (Week 2, Days 1-2)

**File**: `crates/auto-lang/src/parser.rs`, `eval.rs`

**Problem**: Can't define methods inside tag definitions like:

```auto
tag May<T> {
    Value T

    fn is_some() bool {
        is self {
            Value(_) => true,
            _ => false
        }
    }
}
```

**Solution**: Extend tag parsing to support methods:

```rust
// In tag_stmt(), after parsing fields, look for methods
while !self.is_kind(TokenKind::RBrace) {
    if self.is_kind(TokenKind::Fn) {
        let method = self.parse_fn_decl()?;
        methods.push(method);
    } else {
        let field = self.tag_field()?;
        fields.push(field);
    }
    self.expect_eos(is_first)?;
}
```

**Evaluator Changes**: Tag methods are just regular functions with `self` parameter.

**Tests**: `test/a2c/040_tag_types/methods.at`

### Step 4: Tag Type C Transpilation (Week 2, Days 3-5)

**File**: `crates/auto-lang/src/trans/c.rs`

**Problem**: a2c doesn't generate C code for tag types.

**Solution**: Implement tag-to-C translation:

**Tag Definition** → C enum + union:

```c
// Input:
tag May {
    Empty Empty
    Value int
    Error str
}

// Output:
typedef enum {
    May_Empty = 0x00,
    May_Value = 0x01,
    May_Error = 0x02
} MayTag;

typedef struct {
    MayTag tag;
    union {
        int value;
        str error;
    } data;
} May;
```

**Tag Construction** → C function call:

```c
// Input: May.Value(42)
// Output: May_value(42)
```

**Pattern Matching** → C switch:

```c
// Input:
is x {
    Empty => print("empty"),
    Value(v) => print(v),
    Error(e) => print(e)
}

// Output:
switch (x.tag) {
    case May_Empty:
        print("empty");
        break;
    case May_Value:
        print(x.data.value);
        break;
    case May_Error:
        print(x.data.error);
        break;
}
```

**Tests**: `test/a2c/040_tag_types/c_transpile.at`

## Success Criteria

- [ ] Tag construction evaluates to Node values
- [ ] Tag pattern matching works with `is` statements
- [ ] Tag methods (static and instance) work
- [ ] Tag types transpile to correct C code
- [ ] 20+ tests passing

## Test Files

1. `construction.at` - Tag variant construction
2. `pattern_matching.at` - Pattern matching with `is`
3. `methods.at` - Static and instance methods
4. `c_transpile.at` - C code generation
5. `generics.at` - Generic type parameters (deferred)

## Timeline

- Week 1, Days 1-2: Tag construction
- Week 1, Days 3-4: Pattern matching
- Week 2, Days 1-2: Methods
- Week 2, Days 3-5: C transpilation
- Week 3: Testing and refinement

## Next Steps

1. Implement tag construction evaluation
2. Test with `Atom.Int(5)` pattern
3. Move to pattern matching
4. Implement C transpilation
5. Comprehensive testing
