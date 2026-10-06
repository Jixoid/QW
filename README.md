# QW Language

QW is a minimalist, modern systems programming language and modular compiler infrastructure written in Rust. Part of the QAOS ecosystem, it focuses on a clean, arena-allocated architecture leveraging LLVM for efficient, low-level machine code generation and in-memory JIT execution.

Other languages: [tr](README.tr.md) | [Documentation Index](doc/README.md)

---

## Key Features

- **Minimalist & Explicit Syntax:** Clean PEG-based grammar with prefix pointers (`^T`), safe references (`&T`), option (`?T`), and error (`!T`) types.
- **Unified Expression Model:** Tail-expression blocks, pattern matching (`match`), and expression-oriented control flow (`if/ef/else`, `loop/else`, `while/else`, `for/in/else`).
- **Polymorphism & Fat Pointers:** Value-layout structures with single data inheritance combined with zero-cost upcasting and dynamic interface dispatch via negative-offset Virtual Method Tables (VMT).
- **SIMD & Scalable Vectors:** First-class support for fixed SIMD vectors (`[T * N]`) and architecture-scalable vectors (`[T *]`).
- **High-Performance Multi-Pass Compiler:** Built on contiguous memory arenas, lightweight 32-bit node IDs, and global string interning.
- **Integrated Tooling & JIT:** Integrated CLI tool (`qw`) supporting compilation to LLVM bitcode, fast checking, phase profiling (`--timings`, `--usages`), and in-memory JIT execution (`qw run`).

---

## Compiler Architecture

The compilation pipeline is divided into four distinct, decoupled passes:

```
Source (.qw) ──> Pass 1: Parse & Scope ──> Pass 2: HIR & Types ──> Pass 3: MIR & Layout ──> Pass 4: LLVM IR & JIT
```

1. **Pass 1 (Front-end):** Fast byte-level tokenization (`qwc_lexer`), Pratt parsing (`qwc_parse`), AST arena generation (`qwc_ast`), and lexical scope/import resolution (`qwc_resolve`).
2. **Pass 2 (Semantic Analysis):** High-level IR generation (`qwc_hir_gen`), type inference and strict signature checking, intrinsic type registration (`qwc_intrinsic`), export mapping, and QW Unit serialization (`.qwu`).
3. **Pass 3 (Mid-Level IR & Optimization):** Control Flow Graph (CFG) construction (`qwc_mir`), memory layout calculation, VMT generation, and fat-pointer dynamic dispatch lowering.
4. **Pass 4 (Back-end & Execution):** LLVM code generation via Inkwell (`qwc_cgen_llvm`), bitcode output (`build/out.bc`), human-readable LLVM IR (`build/out.ll`), and in-memory JIT execution.

For detailed information, see [Compiler Architecture](doc/Architecture.md).

---

## Quick Start

### Build the Compiler

```bash
cargo build --release
```

### Create and Run a QW Project

```bash
# Initialize a new project
cargo run -- init my_project
cd my_project

# Run the project via JIT
cargo run -- run

# Build LLVM bitcode
cargo run -- build
```

---

## Documentation

Full documentation is available in the [`/doc`](doc/README.md) directory:

- [**Documentation Index (`doc/README.md`)**](doc/README.md)
- [**Syntax & Grammar Reference (`doc/Syntax.md`)**](doc/Syntax.md) - Formal PEG specification, keywords, operators, and expressions.
- [**Compiler Architecture (`doc/Architecture.md`)**](doc/Architecture.md) - Pipeline phases, workspace crates map, and memory model.
- [**Type System Specification (`doc/TypeSystem.md`)**](doc/TypeSystem.md) - Primitives, structs, interfaces, traits, variants, pointers, and vectors.
- [**VMT & Fat Pointer ABI (`doc/VMT.md`)**](doc/VMT.md) - Virtual Method Tables, negative-offset headers, and dynamic dispatch.
- [**Name Mangling Specification (`doc/Mangling.md`)**](doc/Mangling.md) - Linker compatibility, runtime mangling, and EBNF ABI specification.
- [**CLI & Configuration (`doc/CLI_and_Config.md`)**](doc/CLI_and_Config.md) - `qw` command-line reference, diagnostic flags, and `qw.conf`.

---

## License

This project is licensed under the GNU General Public License version 3 (GPL3).

Copyright (c) 2025-2026 Kadir Aydın.

---

## QAOS

Built with ❤️ by the open source community.
