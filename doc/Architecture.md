# QW Compiler Architecture

This document describes the high-level architecture, pipeline phases, crate organization, and internal subsystems of the reference QW compiler.

---

## 1. Overview & Architectural Philosophy

The QW compiler is designed around four foundational principles:

1. **Strictly Decoupled Phases:** Each compilation stage (AST, HIR, MIR, LLVM IR) has a distinct Intermediate Representation (IR) and isolated lowering pass.
2. **Compact Arena-Allocated Graph:** AST, HIR, and MIR nodes are stored in contiguous arena memory buffers, indexed by strongly typed, lightweight 32-bit IDs (`ItemId`, `TypeId`, `ExprId`, `SymbId`).
3. **Interned String Representation:** All identifiers and textual symbols are mapped to 32-bit `Sid` values in `StrInterner` for fast hashing and comparison.
4. **LLVM-Backed Code Generation:** High-performance, multi-target machine code generation leveraging LLVM via the `inkwell` bindings.

```
                      +-----------------------------+
                      |       Source (*.qw)         |
                      +-----------------------------+
                                     |
                                     v
                       [ Phase 1: Lexer & Parser ]
                        (qwc_lexer, qwc_parse)
                                     |
                                     v
                         [ AST (qwc_ast) + Scopes ]
                        (qwc_resolve::ScopeMap)
                                     |
                                     v
                       [ Phase 2: HIR Generation ]
                        (qwc_hir_gen, qwc_hir)
                                     |
                          +----------+---------------+
                          |          |               |
                          v          |               v
                  [ ExportMap ]      |     [ QW Unit Serializer ]
               (Symbol Resolution)   |    (.qwu / qwc_unit package)
                          |          |
                          v          v
                       [ Phase 3: MIR Generation ]
                        (qwc_mir_gen, qwc_mir)
                     (CFG, Layout, VMT Generation)
                                     |
                                     v
                      [ Phase 4: Code Generation ]
                         (qwc_cgen_llvm / LLVM)
                                     |
                     +---------------+---------------+
                     |                               |
                     v                               v
            [ LLVM Bitcode ]                  [ JIT / VM Run ]
          (build/out.bc / .ll)             (In-memory execution)
```

---

## 2. Multi-Pass Compilation Pipeline

The compilation process is managed by `qwc_route::build` through four successive passes:

### Pass 1: Lexing, Parsing & Scope Resolution

- **Lexer (`qwc_lexer`):** Reads the source file via memory-mapped buffers (`File`) in `qwc_arena`. Emits strongly-typed tokens (`Word`) consisting of `(offset, length, file_id, WK)` without heap allocations.
- **Parser (`qwc_parse`):** Uses recursive descent and Pratt parsing for expressions with operator precedence. Builds the `ast::Krate` tree inside the AST arena.
- **Module Loader:** Recursively resolves `mod <name>;` declarations into matching `<name>.qw` or `<name>/mod.qw` files on disk.
- **Core Intrinsics Injection:** Initializes the compiler's `core` crate via `qwc_intrinsic::new_core`, registering primitive types (`bool`, `i8`..`i128`, `u8`..`u128`, `b8`..`b128`, `f16`..`f128`).
- **Scope Collector (`qwc_resolve`):** Traverses the AST to establish lexical scopes, registers declared symbols, resolves imports (`use crate::...`, `use super::...`), and produces a `ScopeMap` and list of `impl` blocks.

### Pass 2: High-Level Intermediate Representation (HIR) & Type Checking

- **HIR Lowering (`qwc_hir_gen`):** Converts the AST into the typed `hir::Krate`.
- **Type Checking:** Validates function signatures, enforces type compatibility on expressions, resolves method calls, and checks trait compliance for `impl` blocks.
- **Type Casting (`as`):** Validates scalar conversions and interface upcasting.
- **Export Resolution (`qwc_resolve::ExportMap`):** Determines the visibility of all top-level items (Internal, Export, Import).
- **Unit Serialization (`qwc_unit`):** Encodes the compiled HIR crate and its `ExportMap` into a binary unit file (`build/out.qwu`) using `postcard` and `serde`.

### Pass 3: Mid-Level Intermediate Representation (MIR) & Layout

- **MIR Lowering (`qwc_mir_gen`):** Translates high-level language constructs into a linear, control-flow graph (CFG) representation with explicit basic blocks (`mir::Block`) and instructions (`mir::Inst`).
- **Control Flow Linearization:** Transforms `if/else`, `match`, `while`, and `loop` into basic block jumps and conditional branch terminators.
- **Memory & Type Layouting:** Computes the byte size and alignment of all structures using target layout configurations (`mir::LayoutInfo`).
- **VMT Construction:** Generates Virtual Method Tables for types that implement interfaces, allocating negative metadata slots and positive method pointer tables.
- **Fat Pointer Generation:** Lowers dynamic interface casts (`struct as Iface`) into `{ data_ptr, vmt_ptr }` fat pointer tuples.

### Pass 4: LLVM IR Code Generation & Execution

- **LLVM Backend (`qwc_cgen_llvm`):** Implements the `qwc_cgen::ICGen` trait to lower MIR into LLVM IR via `inkwell`.
- **Symbol Emission (`SymbLow`):** Emits global variables, functions, and VMT constants with appropriate linkage and visibility attributes.
- **Instruction Lowering (`InstLow` & `BlokLow`):** Emits LLVM instructions for integer/float operations, memory access (`Alloca`, `Load`, `Store`, `Gep`), function calls, and control flow branching.
- **Artifact Output:** Writes binary LLVM bitcode (`build/out.bc`) and, optionally, textual IR (`build/out.ll`).
- **JIT Execution:** When requested via `qw run`, initializes an LLVM `ExecutionEngine` to run the JIT-compiled binary in-process.

---

## 3. Workspace Crates Map

The compiler is organized as a Cargo workspace with 20 modular crates:

| Crate | Directory | Purpose |
| :--- | :--- | :--- |
| **`QW`** (Root) | `src/` | Main CLI entry point binary (`main.rs`). Initializes crash handlers (`ICE`) and delegates to `qwc_route`. |
| **`qwc_arena`** | `crates/qwc_arena` | Arena allocation infrastructure, file management (`File`, `Files`), and contiguous ID-indexed storage. |
| **`qwc_ast`** | `crates/qwc_ast` | Abstract Syntax Tree node definitions (`Item`, `Type`, `Expr`, `Patt`, `Field`, `Visibility`, `Attribute`). |
| **`qwc_cgen`** | `crates/qwc_cgen` | Abstract backend interface trait (`ICGen`). |
| **`qwc_cgen_llvm`** | `crates/qwc_cgen_llvm` | Concrete LLVM code generation backend using Inkwell. |
| **`qwc_comptime`** | `crates/qwc_comptime` | Constant evaluator for compile-time expressions. |
| **`qwc_diagnostic`** | `crates/qwc_diagnostic` | Compiler diagnostics, span tracking, error codes (`CodedMsg`), and formatted snippet rendering. |
| **`qwc_ds`** | `crates/qwc_ds` | Data structure loader and parser for QW Configuration (`qw.conf`) and structured metadata. |
| **`qwc_dump`** | `crates/qwc_dump` | Pretty printers and textual dumpers for AST, Scope, HIR, and MIR representations. |
| **`qwc_hir`** | `crates/qwc_hir` | High-level Intermediate Representation structures and types. |
| **`qwc_hir_gen`** | `crates/qwc_hir_gen` | AST-to-HIR lowering, semantic analysis, and type checking. |
| **`qwc_intrinsic`** | `crates/qwc_intrinsic` | Core crate generator and compiler intrinsic type registrations. |
| **`qwc_lexer`** | `crates/qwc_lexer` | High-throughput lexical analyzer producing `Word` tokens and `WK` word kinds. |
| **`qwc_mangling`** | `crates/qwc_mangling` | ABI name mangling engine for symbols, method implementations, and VMTs. |
| **`qwc_mir`** | `crates/qwc_mir` | Mid-level Intermediate Representation (CFG, basic blocks, SSA instructions, layouts). |
| **`qwc_mir_gen`** | `crates/qwc_mir_gen` | HIR-to-MIR lowering pass and control-flow graph builder. |
| **`qwc_parse`** | `crates/qwc_parse` | Recursive descent and Pratt parser constructing AST nodes from tokens. |
| **`qwc_resolve`** | `crates/qwc_resolve` | Lexical scope resolution, module import binding, and symbol export mapping. |
| **`qwc_route`** | `crates/qwc_route` | CLI command driver (`init`, `build`, `run`, `check`), package config parser, and compilation orchestrator. |
| **`qwc_string_interner`**| `crates/qwc_string_interner`| Global string interner mapping string slices to compact `Sid` keys. |
| **`qwc_unit`** | `crates/qwc_unit` | Binary serialization for compiled package units (`.qwu`). |

---

## 4. Memory Management & Node Arenas

To maximize cache locality and eliminate pointer-chasing overhead:

- Nodes of each IR type (`ast::Item`, `hir::Expr`, `mir::Block`, etc.) are held in separate continuous vectors inside their respective `Krate` structures.
- References between nodes use strongly typed identifiers rather than heap pointers:
  ```rust
  pub struct ItemId(NonZeroU32);
  pub struct TypeId(NonZeroU32);
  pub struct ExprId(NonZeroU32);
  ```
- Contiguous sequences of nodes (such as function argument lists or block statements) are stored as contiguous slice ranges (`ItemRng`, `TypeRng`, `ExprRng`).
- Source spans are represented as compact `(offset: u32, len: NonZeroU16, fid: u16)` structures, allowing any node to track its precise origin in source code with minimal memory overhead.

---

## 5. Diagnostic Architecture

The compiler features a modern diagnostic engine in `qwc_diagnostic`:

- **Structured Diagnostic Codes:** Errors and warnings are cataloged as reusable `CodedMsg` constants with documentation.
- **Source Underlining:** Emits compiler messages with terminal color highlighting, multi-line span indicators (`^~~~`), and context notes.
- **Suggestions with Levenshtein Distance:** When an unresolved identifier is encountered in `qwc_resolve`, the diagnostic engine uses Damerau-Levenshtein distance against active scope symbols to provide "Did you mean?" suggestions.
- **Crash Protection (`ICE`):** Internal Compiler Error handling catches panics and formats actionable bug report templates for compiler developers.
