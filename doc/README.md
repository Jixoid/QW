# QW Language Documentation

Welcome to the official documentation for the **QW Programming Language** and its modular compiler toolchain.

QW is a modern, minimalist systems programming language designed for predictable execution, high performance, and explicit memory and type semantics. Its reference compiler is written in Rust as part of the QAOS ecosystem, using a custom arena-allocated multi-pass architecture with an LLVM backend.

---

## Documentation Index

| Document | Description |
| :--- | :--- |
| [**Syntax & Grammar (`Syntax.md`)**](Syntax.md) | Complete Parsing Expression Grammar (PEG), lexical rules, declarations, types, statements, expressions, precedence tables, and attributes. |
| [**Compiler Architecture (`Architecture.md`)**](Architecture.md) | Multi-pass compiler pipeline (AST → Scope → HIR → MIR → LLVM IR), memory and arena layout, workspace crates map, and diagnostics engine. |
| [**Type System (`TypeSystem.md`)**](TypeSystem.md) | Primitive types, compound types (`struct`, `iface`, `trait`, `enum`, `flags`, `variant`), pointer/reference semantics, slices, arrays, vectors, and casting rules. |
| [**Virtual Method Tables & ABI (`VMT.md`)**](VMT.md) | Runtime ABI specification, negative-offset metadata headers, Fat Pointer layout for dynamic dispatch, and zero-cost upcasting. |
| [**Name Mangling (`Mangling.md`)**](Mangling.md) | Linker compatibility rules, current runtime mangling implementation (`qwc_mangling`), and the complete formal EBNF ABI mangling specification. |
| [**Compiler Intrinsics (`SysIntrinsic.md`)**](SysIntrinsic.md) | Built-in compile-time reflection, type queries, and layout introspection functions (`sys::*`). |
| [**CLI & Configuration (`CLI_and_Config.md`)**](CLI_and_Config.md) | The `qw` command-line tool (`init`, `build`, `run`, `check`), inspection flags (`--timings`, `--usages`, `--dump`), and `qw.conf` package configuration. |

---

## Quick Tour of QW

A minimal QW program starts with `main`:

```qw
fun main() -> i32 {
  ret 0;
}
```

### Key Language Highlights

- **Explicit Pointers & References:**
  ```qw
  var value: i32 = 42;
  let ptr: ^i32 = value&;       // Raw pointer obtained via '&'
  let val: i32 = ptr^;          // Dereferenced via postfix '^'
  ```

- **Object-Oriented & Polymorphic Abstractions:**
  ```qw
  iface Drawable {
    fun draw();
  }

  struct Circle {
    radius: f32;

	impl: Drawable {
		fun draw() {
	      // Draw circle implementation
	    }
	}
  }
  ```

- **Algebraic Data Types & Match Expressions:**
  ```qw
  variant Shape {
    Circle(f32),
    Rectangle(f32, f32),
  }

  fun area(s: Shape) -> f32 {
    ret match s {
      Shape::Circle(r) => 3.14159 * r * r,
      Shape::Rectangle(w, h) => w * h,
    };
  }
  ```

- **Modern Control Flow with Expressions:**
  ```qw
  let status = if count > 0 { 1 } ef count == 0 { 0 } else { -1 };

  while condition {
    // Loop body
  } else {
    // Optional else block when loop finishes naturally
  }
  ```

- **FFI & Low-Level Control:**
  ```qw
  mod clib {
    ![C]
    fun malloc(size: usize) -> ^u8;
    ![C]
    fun free(ptr: ^u8);
  }
  ```

---

## Contributing to Documentation

When updating the language specification or compiler behavior:
1. Verify changes against the source in `crates/qwc_lexer`, `crates/qwc_parse`, `crates/qwc_hir_gen`, and `crates/qwc_mir_gen`.
2. Keep the formal PEG and EBNF grammars synchronized with parser changes.
3. Ensure cross-links between documentation files remain valid.
