# ehrmantraut Architecture Overview

## Purpose

ehrmantraut is a language-neutral static analysis infrastructure that provides AST parsing, Scope/Symbol Table, Control Flow Graph (CFG), and Data Flow Graph (DFG). Clients can consume these data structures to implement their own inspection rules, linters, or code analysis tools.

## Architecture

```
Source Code
    │
    ▼
[tree-sitter CST]              ← External library handles raw parsing
    │
    ▼
[Lowering (per language)]       ← Only language-specific part in ehrmantraut
    │
    ▼
[Common IR]                     ← Simplified, language-neutral representation
    │
    ├──→ Scope Builder (common) ── → Scope / Symbol Table
    ├──→ CFG Builder (common)   ── → Control Flow Graph
    └──→ DFG Engine (common)    ── → Data Flow Graph
    │
    ▼
[Client: inspection rules, linters, etc.]
```

### Key Principle

- **Language-specific code**: Only the Lowering layer (tree-sitter CST → Common IR)
- **Language-neutral code**: IR definition + Scope Builder + CFG Builder + DFG Engine
- Adding a new language requires implementing **only the Lowering layer**

## Layers

### 1. tree-sitter (External)

Raw parsing of source code into a Concrete Syntax Tree (CST). Each language has its own grammar and node types. This is not abstracted — tree-sitter handles it.

### 2. Lowering (Per Language)

Transforms language-specific tree-sitter CST nodes into Common IR nodes.

Example — both JS and Python produce the same IR:

```
// JS:  let x = 1        // Python:  x = 1
// tree-sitter:           // tree-sitter:
// lexical_declaration    // assignment
//   variable_declarator  //   identifier "x"
//     identifier "x"    //   integer "1"
//     number "1"

// After lowering (both):
VariableDecl { name: "x", value: Literal(1) }
```

Non-trivial cases to consider in IR design:
- Python `for...else` → decompose into `For` + `If`, or add `ForElse` to IR?
- JS `var` hoisting → annotate with `scope_level: Function` (see below)
- Rust `let Some(x) = expr` → `VariableDecl` + pattern destructuring

These decisions should be driven by what inspection rules actually need.

#### Critical Rule: Lowering Must Not Rearrange Structure

Lowering must preserve original source ordering and express language-specific semantics through **annotations**, not structural transformations.

**Bad** — moving declaration destroys source order:

```
// Original JS:
function foo() {
  console.log(x)   // line 2: hoisting causes undefined
  var x = 1         // line 3
}

// BAD lowering (structural rearrangement):
FunctionBody {
  VariableDecl { name: "x" }                                  // moved to top
  Call { callee: "console.log", args: [Ref("x")] }            // line 2
  Assignment { target: "x", value: 1 }                         // line 3
}
// Problem: "used before declaration" is no longer detectable
```

**Good** — preserving order with annotation:

```
// GOOD lowering (annotation-based):
FunctionBody {
  Call { callee: "console.log", args: [Ref("x")] }            // line 2
  VariableDecl { name: "x", value: 1, scope_level: Function }  // line 3
}
// scope_level: Function tells Scope Builder to register in function scope
// Source ordering is preserved → "used before declaration" is detectable
```

This principle applies generally: whenever Lowering needs to encode language-specific behavior, use annotations on IR nodes rather than restructuring the IR tree. Restructuring risks losing information that inspection rules depend on.

### 3. Common IR

A simplified AST containing only constructs needed for analysis:

| Category | Node Types |
|---|---|
| Declarations | `VariableDecl`, `FunctionDecl`, `ClassDecl` |
| Control Flow | `If`, `For`, `While`, `Switch`, `TryCatch`, `Return`, `Break`, `Continue` |
| Expressions | `Assignment`, `Call`, `MemberAccess`, `BinaryExpr` |
| Scoping | `Block`, `FunctionBody`, `Module` |

This is **not** a full AST. It intentionally discards language-specific details that are irrelevant to analysis.

### 4. Scope / Symbol Table (Common)

Built from Common IR. The output is language-neutral:

> "Symbol X is declared at location A, referenced at locations B, C"

Construction is language-neutral because scoping semantics are encoded during the Lowering step.

### 5. Control Flow Graph (Common)

Built from Common IR. The output is a graph of basic blocks and edges.

Language-specific control flow semantics (exception propagation, async/await, etc.) are normalized during Lowering.

### 6. Data Flow Graph (Common)

Operates on top of CFG. Algorithms like reaching definitions and live variable analysis are inherently language-neutral.

## Design Decisions

### Type Information

**Not provided.** Type information is fundamentally language-specific and would require reimplementing each language's type checker. Clients that need type info (e.g., for type mismatch or narrowed type checks) should use language-specific tools directly (e.g., TypeScript Compiler API).

### Why Not a Language-Neutral AST?

A fully unified AST either becomes too generic (losing useful information) or too complex (superset of all languages). Instead:

- **AST level**: Keep language-specific (tree-sitter CST, no information loss)
- **IR level and above**: Language-neutral (intentional, analysis-focused abstraction)

## What Clients Can Build

Each layer enables different categories of analysis:

| Infrastructure | Example Use Cases |
|---|---|
| AST (tree-sitter query) | Pattern-based checks (e.g., `debugger statement`, naming conventions, code style) |
| Scope / Symbol Table | Symbol resolution checks (e.g., unused imports, unresolved references, duplicate declarations) |
| CFG | Control flow checks (e.g., unreachable code, infinite loops, inconsistent returns) |
| Data Flow | Data flow checks (e.g., unused assignments, self-assignment, redundant variables) |

Type-dependent analysis (e.g., type mismatch, narrowed type) is out of scope and should be handled by clients using language-specific type checkers.
