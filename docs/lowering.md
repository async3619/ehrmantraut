# Lowering Layer Design Guide

## What is Lowering?

Lowering is the only language-specific layer in ehrmantraut. It transforms a tree-sitter Concrete Syntax Tree (CST) into a Common IR that downstream analysis passes (Scope Builder, CFG, DFG) can consume without knowing which language the source was written in.

```
tree-sitter CST (language-specific)
    │
    ▼
  Lowering          ← This document covers this layer
    │
    ▼
Common IR (language-neutral)
```

The goal of lowering is **recognition, not interpretation**. It answers "what constructs exist here?" — not "what do they mean at runtime?"

## Core Principles

### 1. Recognize structure, don't interpret semantics

Lowering identifies syntactic constructs and maps them to IR nodes. It does **not** evaluate, desugar, or infer runtime behavior.

**Good — recognize the construct:**

```
// Source: x ?? y
BinaryExpr { operator: "??", left: x, right: y }
```

**Bad — interpret the semantics:**

```
// Source: x ?? y
// Desugared to: x !== null && x !== undefined ? x : y
IfStmt { condition: BinaryExpr { ... }, ... }
```

The `??` operator is a binary expression. What it *means* (null-coalescing) is for analysis passes to determine based on the operator string. Lowering just says "there is a binary expression with operator `??`".

### 2. Unify syntax variations, not language semantics

Different languages — and even the same language — express identical concepts with different syntax. Lowering collapses these variations into a single IR node.

```
// All three are "a function named foo":
function foo() {}           // function declaration
const foo = function() {}   // function expression assigned to variable
const foo = () => {}        // arrow function assigned to variable

// After lowering, all produce:
FunctionDecl { name: "foo", params: [], body: Block { ... } }
```

This is the power of lowering: downstream passes see "a function" without caring about the syntactic form. However, **lowering does not merge semantically different constructs** just because they look similar:

```
// These are NOT the same:
new Foo()    // constructor invocation → NewExpr
Foo()        // function call → Call

// Even though they look structurally similar (callee + arguments),
// they have different semantics that analysis passes may care about.
```

### 3. Preserve source order — use annotations for language behavior

Lowering must never rearrange the IR tree. Language-specific behaviors are expressed through **annotations** on IR nodes, not through structural transformations. See [Architecture Overview](./architecture-overview.md) for the detailed rationale and examples.

```
// JS var hoisting:
console.log(x)    // line 1
var x = 1         // line 2

// Lowering preserves order, annotates:
Call { ... }                                                 // line 1
VariableDecl { name: "x", value: 1, scope_level: Function }  // line 2
//                                   ^^^^^^^^^^^^^^^^^^^^
//                                   annotation, not reordering
```

### 4. Opaque is the escape hatch, not a failure

When a CST node kind is not explicitly handled, it becomes an `OpaqueNode` (statement) or `OpaqueExpr` (expression). This is by design:

- Opaque nodes preserve the original CST kind and source text
- Analysis passes can still operate on the surrounding recognized structure
- New lowering support can be added incrementally without breaking existing analysis

However, opaque should be reserved for constructs that are **genuinely outside the current IR's scope**, not as a shortcut to avoid implementation. If a construct is common enough that analysis passes would benefit from recognizing it, it should be lowered.

## What Lowering Should Do

### Collapse syntax variations

Map multiple syntactic forms of the same concept to a single IR node type.

| Syntax variations | Common IR |
|---|---|
| `function f()`, `const f = () => {}`, `const f = function(){}` | `FunctionDecl` |
| `let x`, `const x`, `var x` | `VariableDecl` + `DeclKind` annotation |
| `for..in`, `for..of` | `ForStmt` + `ForKind` annotation |
| `while`, `do..while` | `WhileStmt` + `isDoWhile` flag |

### Strip syntactic noise

Remove tokens that exist only for parsing, not for analysis:

- Semicolons, commas, parentheses
- TypeScript type annotations, interfaces, type aliases
- Comments (unless analysis-relevant)

### Attach metadata as annotations

Encode language-specific properties that analysis passes may need:

| Annotation | Purpose |
|---|---|
| `scope_level: Block \| Function \| Module` | Scope builder uses this to place declarations correctly |
| `decl_kind: let \| const \| var \| ...` | Rules can distinguish mutability |
| `is_async`, `is_generator` | Async/generator function detection |
| `is_export`, `is_default` | Module export analysis |

### Normalize blocks

Wrap single statements in blocks so analysis passes always see a consistent `Block` structure:

```
// Source:
if (x) return

// After lowering:
IfStmt { condition: x, consequent: Block { body: [ReturnStmt] } }
//                                 ^^^^^
//                                 always a Block, never a bare statement
```

## What Lowering Should NOT Do

### Don't desugar

Desugaring transforms higher-level constructs into lower-level equivalents. This destroys information that analysis passes need.

| Source | Bad (desugaring) | Good (recognition) |
|---|---|---|
| `x?.y` | `x !== null ? x.y : undefined` | `MemberAccess { optional: true }` |
| `x ??= y` | `if (x == null) x = y` | `Assignment { operator: "??=" }` |
| `for (const x of arr)` | `const it = arr[Symbol.iterator](); ...` | `ForStmt { kind: Of }` |
| `class Foo extends Bar` | prototype chain setup | `ClassDecl { super_class: Bar }` |

### Don't evaluate

Lowering operates on syntax, not values. It never computes results or resolves references.

| Don't | Why |
|---|---|
| Constant folding (`1 + 2` → `3`) | Analysis pass responsibility |
| Reference resolution (`x` → which declaration?) | Scope builder responsibility |
| Type inference | Explicitly out of scope (see architecture doc) |
| Dead code elimination | CFG analysis responsibility |

### Don't invent structure

The IR should only contain constructs that have a direct correspondence in the source code. Don't generate synthetic nodes.

| Don't | Why |
|---|---|
| Insert implicit `return undefined` at function end | Analysis pass can infer this from CFG |
| Generate constructor if class has none | Analysis pass can detect missing constructor |
| Expand `...args` into individual arguments | Spread is a first-class concept |

### Don't encode language-specific runtime rules

Runtime semantics like hoisting order, temporal dead zones, or prototype chains are not the lowering layer's concern. The IR should provide enough annotations for analysis passes to *implement* these rules if needed, but lowering itself should not enforce them.

## Deciding Whether to Lower a Construct

Use this decision tree when considering whether to add IR support for a new syntax:

```
Is this construct commonly used in real-world code?
├── No → Keep as Opaque (regex lookbehinds, with statements, etc.)
└── Yes
    └── Would analysis passes benefit from recognizing it?
        ├── No → Keep as Opaque (purely syntactic sugar with no analysis impact)
        └── Yes
            └── Can it be mapped to an existing IR node type?
                ├── Yes → Map to existing type (arrow fn → FunctionDecl)
                └── No → Introduce a new IR node type
                    └── Is the new type language-neutral?
                        ├── Yes → Add to Common IR (most constructs)
                        └── No → Reconsider; use annotations or Opaque
```

### Examples

| Construct | Decision | Reasoning |
|---|---|---|
| Arrow function | Map to `FunctionDecl` | Same concept, different syntax |
| Optional chaining `?.` | Extend `MemberAccess` | Adds a flag, doesn't create a new concept |
| JSX `<Comp />` | Opaque | Framework-specific, not a general programming construct |
| `with` statement | Opaque | Deprecated, rarely used, complex scoping implications |
| `await` expression | New `AwaitExpr` | Common, affects control flow analysis |
| TypeScript `as` cast | Filter out | Type-only, no runtime effect |

## Adding New Lowering Support

When implementing a new lowering handler:

1. **Check existing IR types first.** Can the construct map to an existing node? Arrow functions → `FunctionDecl`, `new X()` → potentially a `Call` variant.

2. **If a new IR type is needed**, ensure it is language-neutral. Ask: "Would this type make sense if we were lowering Python or Rust too?"

3. **Add a match arm** in the appropriate dispatcher (`lower_node` for statements, `lower_expression` for expressions).

4. **Write tests** that verify the IR structure. Use snapshot-style assertions to ensure the lowered output is stable.

5. **Update type exports** — the TypeScript types in `types.d.ts` are auto-generated from Rust via `ts-rs`. Run the export after modifying IR types.

6. **Keep commits atomic** — separate IR type additions from lowering logic from tests (see [workflow guide](../.claude/rules/workflow.md)).
