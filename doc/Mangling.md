# QW Name Mangling Specification & ABI

This document defines the symbol naming conventions and Application Binary Interface (ABI) name mangling rules used by the QW compiler.

---

## 1. General Principles and Linker Compatibility

All symbols produced by the QW compiler must be 100% compatible with standard operating system linkers (GNU `ld`, LLVM `lld`, and MSVC `link.exe`).

### Character Set Rules
Symbols may only contain the following characters:
- **Alphabetic:** `A-Z`, `a-z`
- **Numeric:** `0-9`
- **Linker-Safe Delimiters:** `_` (underscore), `$` (dollar sign), `@` (at sign)

> **Prohibited Characters:** Addressing or template characters such as `[`, `]`, `<`, `>`, `*`, `&`, `^`, or `:` must never appear in raw emitted linker symbols.

### UTF-8 Byte Length Encoding
All length specifiers prefixing names represent the **exact number of bytes in UTF-8 encoding**, not source character counts.
- *Example:* For identifier `café`, the byte count is `5` (since `é` takes 2 bytes in UTF-8), resulting in `5café`.

---

## 2. Current Compiler Implementation (`qwc_mangling`)

The active reference compiler implementation in `crates/qwc_mangling` provides fast, structured symbol mangling for modules, functions, methods, and virtual method tables:

### 2.1 Free Functions and Top-Level Symbols
Free functions and global symbols use the prefix `qw_` followed by each scope segment and the symbol name, each prefixed with its UTF-8 length:
```
qw_<scope_len><scope_name>...<name_len><name>
```

- **Example 1:** Free function `main` at root:
  - Emitted Symbol: `qw_4main`
- **Example 2:** Function `calculate` inside module `math::stats`:
  - Emitted Symbol: `qw_4math5stats9calculate`
- **Example 3 (Entry Point):** When marked with `![entry]`, the compiler emits:
  - Emitted Symbol: `qw_entry`

### 2.2 Method Implementations (`new_impl`)
Methods implemented for structures and interfaces use the `qw_impl_` prefix, encoding the target type and method identifier:
```
qw_impl_<target_type_path>_<optional_iface_path><method_name>
```

- **Inherent Struct Method:**
  ```qw
  struct Vector { fun length() -> f32; }
  ```
  - Emitted Symbol: `qw_impl_6Vector_6length`
- **Interface Implementation Method:**
  ```qw
  impl Vector: Printable { fun print(); }
  ```
  - Emitted Symbol: `qw_impl_6Vector_9Printable5print`

### 2.3 Virtual Method Tables (`new_vmt`)
VMTs associated with a structure implementing an interface use the `qw_vmt_` prefix:
```
qw_vmt_<struct_path>_<iface_name>
```

- **Example:** Struct `Circle` implementing `Drawable`:
  - Emitted Symbol: `qw_vmt_6Circle_8Drawable`

---

## 3. Formal Type-Safe Mangling Grammar (Target ABI Specification)

For full signature-based type safety, method overloading, and template specialization, the formal target ABI grammar is defined below in Extended Backus-Naur Form (EBNF):

```ebnf
<mangled-name>      ::= "_qw_" [ <special-prefix> ] <type-path> [ <method-decl> ]

<special-prefix>    ::= "vmt_" | "rtti_" | "init_" | "fini_"

<type-path>         ::= <scope>* <source-name> [ <generic-list> ]

<scope>             ::= <length> <identifier>
<source-name>       ::= <length> <identifier>

<method-decl>       ::= <source-name> <self-type> <return-type> <type>*

<generic-list>      ::= "G" <length> <type>*

<type>              ::= <primitive-type> | <modifier>* <complex-type>
<complex-type>      ::= "N" <type-path> "Z" | "x"
<self-type>         ::= <type>

<primitive-type>    ::= "v" | "b" | "c" | "p" | "l" | "h" | "f" | "d" | "g" 
                      | (<integer-sign> <integer-size>)
<integer-sign>      ::= "S" | "U"
<integer-size>      ::= "t" | "s" | "i" | "l" | "y" | "n"

<modifier>          ::= "P" | "R" | "M" | "V" | "Z" | <fixed-array>
<fixed-array>       ::= "A" <length> "_"

<length>            ::= [0-9]+
<identifier>        ::= [A-Za-z0-9_]+
```

### 3.1 Type Encoding Table

| Code | Type | Meaning |
| :---: | :--- | :--- |
| **`v`** | `()` / `void` | Unit / void |
| **`b`** | `bool` | 1-bit boolean |
| **`c`** | `char` | 8-bit character |
| **`p`** | `^void` | Raw untyped pointer |
| **`l`** | `null_t` | Null pointer |
| **`St` / `Ut`** | `i8` / `u8` | 8-bit signed / unsigned integer |
| **`Ss` / `Us`** | `i16` / `u16` | 16-bit signed / unsigned integer |
| **`Si` / `Ui`** | `i32` / `u32` | 32-bit signed / unsigned integer |
| **`Sl` / `Ul`** | `i64` / `u64` | 64-bit signed / unsigned integer |
| **`Sy` / `Uy`** | `i128` / `u128`| 128-bit signed / unsigned integer |
| **`Sn` / `Un`** | `isize` / `usize`| Pointer-sized platform integer |
| **`h`** | `f16` | 16-bit half-precision float |
| **`f`** | `f32` | 32-bit single-precision float |
| **`d`** | `f64` | 64-bit double-precision float |
| **`g`** | `f128` | 128-bit quad-precision float |

### 3.2 Modifiers & Complex Type Encapsulation

- **`P`**: Raw Pointer (`^T`)
- **`R`**: Reference (`&T`)
- **`M`**: Mutable (`mut`)
- **`V`**: Volatile
- **`Z`**: Slice (`[T]`)
- **`A<size>_`**: Fixed Array (`[T; size]`). E.g. `[i32; 10]` encodes as `A10_Si`.
- **`N...Z`**: Encapsulates user-defined and compound types (`N3std6StringZ`).
- **`x` (Backreference):** Replaces a type identical to the immediately preceding complex type to prevent symbol bloat. E.g. `fun cmp(Vector, Vector)` encodes the second parameter as `x`.

---

## 4. Method Uniformity: Static vs Instance

In the full ABI specification, static functions and instance methods share the same `<method-decl>` formula:
- **Instance Method (mutable `self`):** `<self>` field contains `Mx` or `MN...Z`.
- **Instance Method (immutable `self`):** `<self>` field contains `x` or `N...Z`.
- **Static Function (no `self`):** `<self>` field is set directly to **`v`** (void).

### Detailed ABI Example

```qw
mod std {
  struct Buffer<T> {
    fun copy(buf: Buffer<T>&) static -> void {}
  }
}
type BufferI32 = std::Buffer<i32>;
```

- **Mangled Target Symbol:** `_qw_3std6BufferG1SiF4copyvvRx`
- **Breakdown:**
  - `_qw_`: Prefix
  - `3std6Buffer`: Path `std::Buffer`
  - `G1Si`: 1 generic argument (`i32`)
  - `F4copy`: Method name `copy`
  - `v`: `<self>` parameter: `void` (static function)
  - `v`: Return type: `void`
  - `Rx`: First argument: `R` (Reference) + `x` (Backreference to `std::Buffer<i32>`)
