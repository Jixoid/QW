# QW Type System Specification

This document provides a comprehensive reference for the QW type system, detailing primitive types, compound user-defined structures, pointer and reference semantics, collections, vectors, and casting rules.

---

## 1. Type System Principles

The QW type system is designed for high-performance systems programming:

- **Static and Explicit:** All types are resolved at compile time during semantic analysis (`qwc_hir_gen`).
- **Predictable Layouts:** Primitive and structural layouts follow deterministic alignment rules compatible with the C ABI and LLVM data layouts.
- **Prefix Modifier Syntax:** Modifiers (`^`, `&`, `?`, `!`, `..`) are placed before the base type, ensuring clean readability from left to right.
- **Duality of Dispatch:** Direct static calls on structures versus dynamic virtual dispatch via Fat Pointers (`{ data_ptr, vmt_ptr }`) on interfaces.

---

## 2. Primitive Types

All primitive types are automatically registered in the global scope via the compiler's `core` crate (`qwc_intrinsic`):

| Type | Bit Width | Description | Example Literal |
| :--- | :---: | :--- | :--- |
| **`bool`** | 1 | Boolean value (`true` or `false`) | `true`, `false` |
| **`i8`** | 8 | 8-bit signed two's-complement integer | `127_i8`, `-12` |
| **`i16`** | 16 | 16-bit signed integer | `32000_i16` |
| **`i32`** | 32 | 32-bit signed integer (default integer) | `42`, `-100` |
| **`i64`** | 64 | 64-bit signed integer | `10000000000_i64` |
| **`i128`** | 128 | 128-bit signed integer | `10000000000000000000_i128` |
| **`u8`** | 8 | 8-bit unsigned integer | `255_u8` |
| **`u16`** | 16 | 16-bit unsigned integer | `65535_u16` |
| **`u32`** | 32 | 32-bit unsigned integer | `4000000_u32` |
| **`u64`** | 64 | 64-bit unsigned integer | `18446744073709551615_u64` |
| **`u128`** | 128 | 128-bit unsigned integer | `..._u128` |
| **`b8` .. `b128`** | 8..128 | Bit-exact, untyped bit-pattern integers | `0xFF_b8` |
| **`isize` / `usize`**| 32/64 | Target platform pointer-sized signed/unsigned int | `0_usize` |
| **`f16`** | 16 | IEEE 754 half-precision float | `1.5_f16` |
| **`f32`** | 32 | IEEE 754 single-precision float | `3.14159_f32` |
| **`f64`** | 64 | IEEE 754 double-precision float | `2.718281828459` |
| **`f128`** | 128 | IEEE 754 quad-precision float | `1.0_f128` |
| **`()` (Unit)** | 0 | Zero-sized type representing the absence of a value | `()` |

---

## 3. Compound & User-Defined Types

### Structures (`struct`)
Structures represent value-type product records. They support single inheritance of state, field definitions, constructors (`init`), destructors (`fini`), methods, and inline trait implementations:

```qw
struct Point {
  x: f32;
  y: f32;
}

impl Point {
  init new(x: f32, y: f32): x(x), y(y) {}

  fun distance(other: Point) -> f32 {
    let dx = self.x - other.x;
    let dy = self.y - other.y;
    ret (dx * dx + dy * dy);
  }
}
```

### Interfaces (`iface`)
Interfaces define polymorphic contracts for dynamic dispatch. An interface may mandate both method signatures and dynamic property offsets:

```qw
iface Printable {
  fun print();
}

struct Document {
  title: [u8];
}

impl Document: Printable {
  fun print() {
    // Print implementation
  }
}
```

When accessed through an interface reference (`&Printable`), the compiler emits a **Fat Pointer** containing the address of the data and a pointer to the Virtual Method Table (VMT).

### Traits (`trait`)
Traits define compile-time and runtime behavioral contracts that can be implemented for any existing type via `impl` blocks:

```qw
trait Equal {
  fun equals(other: Self) -> bool;
}

impl Point: Equal {
  fun equals(other: Point) -> bool {
    self.x == other.x && self.y == other.y
  }
}
```

### Enums & Bit Flags (`enum`, `flags`)
- **`enum`**: Enumerated constants with implicit or explicit discriminant values:
  ```qw
  enum Status {
    Pending,
    Running,
    Completed = 10,
    Failed,
  }
  ```
- **`flags`**: Type-safe bitflags designed for bitwise operations:
  ```qw
  flags Permissions {
    Read = 1,
    Write = 2,
    Execute = 4,
  }
  ```

### Variants (`variant`)
Variants represent tagged unions (sum types) capable of carrying tuple payloads:

```qw
variant WebEvent {
  PageLoad,
  KeyPress(u8),
  Click(i32, i32),
}

fun handle_event(e: WebEvent) {
  match e {
    WebEvent::PageLoad => {},
    WebEvent::KeyPress(key) => {},
    WebEvent::Click(x, y) => {},
  }
}
```

---

## 4. Pointers & References

QW distinguishes between non-owning raw pointers and references:

### Raw Pointers (`^T`, `^mut T`)
- **`^T`**: Constant raw pointer.
- **`^mut T`**: Mutable raw pointer.
- **Dereferencing:** Performed using the postfix `^` operator:
  ```qw
  let p: ^i32 = ...;
  let val: i32 = p^;           // Read value from pointer
  ```
- **Address-of:** Performed using the postfix `&` operator:
  ```qw
  var num: i32 = 100;
  let p: ^i32 = num&;          // Obtain raw pointer
  ```

### References (`&T`, `&mut T`)
- **`&T`**: Safe, immutable reference.
- **`&mut T`**: Safe, mutable reference.
- References guarantee proper alignment and are non-null by contract.

---

## 5. Sequences, Arrays & Vectors

| Syntax | Category | Description | Memory Representation |
| :--- | :--- | :--- | :--- |
| **`[T]`** | Slice | Dynamically-sized slice view | Fat Pointer: `{ ptr: ^T, len: usize }` |
| **`[T; N]`** | Fixed Array | Contiguous array of `N` elements | Inlined array of `N * sizeof(T)` bytes |
| **`[T * N]`** | SIMD Vector | Fixed SIMD vector of `N` lanes | Hardware vector register (`<N x T>`) |
| **`[T *]`** | Scalable Vector | Architecture-scalable vector | Scalable vector register (`<vscale x N x T>`) |

```qw
// Fixed-size array initialized with propagate syntax:
var buffer: [u8; 1024];

// SIMD vector:
let v1: [f32 * 4];
```

---

## 6. Wrapper Types: Option & Error Propagation

- **Option Type (`?T`):** Represents an optional value:
  ```qw
  fun find_item(id: i32) -> ?Item;
  ```
- **Error/Fail Type (`!T`):** Represents a result that might fail:
  ```qw
  fun read_file(path: [u8]) -> ([u8] ! io::Error);
  ```
- **Operators:**
  - **`?` (Try / Propagate):** If the value is absent or an error, returns early from the surrounding function.
  - **`!!` (Force Unwrap):** Asserts the value is present; crashes or aborts if absent/error.

---

## 7. Type Conversions & Casting (`as`)

Explicit type conversions are performed via the `as` operator:

```qw
let small: i16 = 500;
let wide: i64 = small as i64;         // Scalar integer extension

let raw_ptr: ^u8 = alloc(...) as ^u8; // Pointer type reinterpretation

// Interface upcasting (constructs a Fat Pointer):
let circle = Circle { radius: 10.0 };
let drawable: &Drawable = circle& as &Drawable;
```

Interface casting verifies at compile time that the source structure implements all required methods and properties of the destination interface, constructing the proper VMT pointer in the emitted MIR.
