## 1. General Principles and Character Set

### 1.1 Constraints and Linker Compatibility

All mangled symbols produced by the QW compiler must be 100% compatible with the target operating system's linker (GNU ld, LLVM lld, MSVC link.exe). Symbols may only contain the following character set:

- Alphabetic characters: `A-Z`, `a-z`
- Numeric characters: `0-9`
- Special characters: `_` (underscore), `$` (dollar sign), `@` (at sign)


> **Critical Rule:** The use of addressing or template-denoting characters such as `[`, `]`, `<`, `>`, `*`, `&` within symbols is strictly prohibited.

### 1.2 Character vs Byte Counting

All `<length>` specifiers in the symbol structure represent the **total number of bytes in UTF-8 encoding**, not the number of characters in the source code.

- _Example:_ For the method `fun café()`, the name length is encoded as `5` instead of `4`, because the letter `é` occupies 2 bytes (`5café`).

## 2. Formal Grammar (EBNF)

The syntactic structure of symbols is precisely defined by the following Extended Backus-Naur Form (EBNF):

EBNF

```
<mangled-name>      ::= "_qw_" <type-path> [ <interface-bound> ] [ <lifecycle-decl> | <method-decl> ] [ <special-suffix> ]
<type-path>         ::= <scope>* <type-kind> <source-name> [ <generic-list> ]

<scope>             ::= <length> <identifier>
<type-kind>         ::= "S" | "I" | "E" | "B"  (* Struct, Interface, Enum, Set *)
<source-name>       ::= <length> <identifier>

<interface-bound>   ::= "$" <type-path> "$"
<lifecycle-decl>    ::= <ctor-decl> | <dtor-decl>
<ctor-decl>         ::= "C" <argument-type>*
<dtor-decl>         ::= "D"
<method-decl>       ::= "F" <source-name> <self-type> <return-type> <argument-type>*

<generic-list>      ::= "G" <type>+
<special-suffix>    ::= "@vmt" | "@rtti"

<type>              ::= <primitive-type> | <modifier>* <complex-type>
<complex-type>      ::= "N" <type-path> "Z" | "x"
<self-type>         ::= <type>

<primitive-type>    ::= "v" | "b" | "c" | "h" | "f" | "d" | "g" | (<integer-sign> <integer-size>)
<integer-sign>      ::= "S" | "U"
<integer-size>      ::= "t" | "s" | "i" | "l" | "y" | "n"

<modifier>          ::= "P" | "R" | "M" | "V" | "Z" | <fixed-array>
<fixed-array>       ::= "A" <length> "_"

<length>            ::= [0-9]+
<identifier>        ::= [A-Za-z0-9_]+
```

### 2.1 Lifecycle Declarations

- **C (Constructor)**: No naming is required because the struct it belongs to is already specified within `<type-path>`. Since the return type is always the object itself, it does not need a `<return-type>` slot either. It only takes zero or more `<argument-type>` entries to support overloading.
- **D (Destructor)**: Since a struct can only have one destructor, cannot accept parameters, and cannot return a value, it is represented by a single `D` character.

## 3. Type System and Qualifiers

### 3.1 Primitive Types

Primitive types have a fixed width and do not take a length prefix.

| **Abbreviation** | **Equivalent**           |
| ----------------- | ------------------------ |
| **`v`**           | void                     |
| **`b`**           | bool (1bit)              |
| **`c`**           | char                     |
| **`p`**           | ptr                      |
| **`l`**           | null_t                   |
| **`St` / `Ut`**   | i8 / u8                  |
| **`Ss` / `Us`**   | i16 / u16                |
| **`Si` / `Ui`**   | i32 / u32                |
| **`Sl` / `Ul`**   | i64 / u64                |
| **`Sy` / `Uy`**   | i128 / u128              |
| **`Sn` / `Un`**   | isize / usize (platform) |
| **`h`**           | f16 (half)               |
| **`f`**           | f32 (float)              |
| **`d`**           | f64 (double)             |
| **`g`**           | f128 (quad)              |


### 3.2 Type Modifiers

Modifiers are always prepended to the left of the target type and can be chained consecutively (resolved from right to left).

- **`P`** : Pointer (`type^`) $\rightarrow$ `Pi` (int*)
- **`R`** : Reference (`type&`) $\rightarrow$ `Ri` (int&)
- **`M`** : Mutable (`mut type`) $\rightarrow$ `Mi` (mut int)
- **`V`** : Volatile (`vol type`)
- **`Z`** : Unbounded Array (`type[]`) $\rightarrow$ `ZSi` (i32[])
- **`A<size>_`** : Fixed-Size Array (`type[X]`). A trailing `_` is appended to indicate the end of the size value.
    
    - _Multi-Dimensional Array Rule:_ Multi-dimensional arrays are formed by chaining modifiers.
    - _Example:_ `i32[10][5]` $\rightarrow$ `A10_A5_Si`

### 3.3 Complex Types and `N...Z` Encapsulation

When a non-primitive, user-defined type (Struct, etc.) is passed as a function argument or generic parameter, its beginning is delimited by the **`N`** character and its end by the **`Z`** character. This allows the parser to precisely distinguish the boundaries between types.

## 4. Advanced Architectures and Resolution Rules

### 4.1 `x` (Backreference) Mechanism

To optimize symbol sizes and prevent bloat, when a type in a function's argument list is **identical to the complete complex type definition immediately preceding it**, the entire path is not rewritten; instead, a single **`x`** character is used in its place.

- _Example:_ In the call `fun foo(vector, vector)`, the second parameter is entirely carried as `x`.

### 4.2 Generic Parsing Rule

Generic lists are opened with the `G` character. If there are multiple generic parameters (`Map<i32, f64>`), the parameters are written consecutively without any separator: `GSi d`.

The parser recognizes that a generic argument has ended in the following two scenarios:

1. When the closing `Z` character of an `N...Z` block is encountered.
2. At structural break points where the type path ends and a method (`F`) or interface bound (`$`) begins.

### 4.3 Static and Instance Method Uniformity

There is no special "static" flag in the method formula. The system naturally resolves objectless (static) function calls by placing **`v` (void)** in the `<self>` slot.

- **Instance Method (`self` mutable):** The `<self>` field becomes `Mx` or `MN...Z`.
- **Instance Method (`self` immutable):** The `<self>` field becomes `x` or `N...Z`.
- **Static Method (no `self`):** The `<self>` field is directly **`v`**.

## 5. Comprehensive and Verified Example Scenarios

### 5.1 Interface Method Overrides

Consider a struct that overrides identically named methods from two different interfaces:

Code snippet

```
// Module: std
iface A { fun draw() -> void; }
struct C: A { fun [A]draw() -> void {} }
```

- **Generated Symbol:** `_qw_3stdS1C$3stdI1A$F4drawvMxv`
    
- **Symbol Breakdown:**
    
    - `_qw_3stdS1C` $\rightarrow$ Main Struct: `std::C` (Struct)
    - `$3stdI1A$` $\rightarrow$ Interface Lock: Overridden via `std::A` (Interface).
    - `F4draw` $\rightarrow$ 4-character method `draw`.
    - `Mx` $\rightarrow$ `<self>` parameter: mutable reference to the preceding struct (`mut C`).
    - `v` $\rightarrow$ Return type: `void`.

### 5.2 Complex Generic and Static Function Combination

Code snippet

```
// Module: std
struct vector<T> {
  fun dummy(vector&) static -> void {}
}
type veci = vector<i32>;
```

- **Generated Symbol:** `_qw_3stdS6vectorGSiF5dummyvvRx`
    
- **Symbol Breakdown:**
    
    - `_qw_3stdS6vectorGSi` $\rightarrow$ Main Struct: `std::vector<i32>`
    - `F5dummy` $\rightarrow$ 5-character function `dummy`.
    - `v` $\rightarrow$ `<self>` field: `void` (Because the function is **static**, it takes no object).
    - `v` $\rightarrow$ Return type: `void`.
    - `Rx` $\rightarrow$ First argument: `R` (Reference) + `x` (Backreference, i.e., the preceding struct `vector<i32>`).
