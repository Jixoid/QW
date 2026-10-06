# QW Compiler Intrinsics & `core::*` Specification

This document defines the built-in compiler intrinsics, the `core` library type registration, and compile-time reflection functions available under the `core::*` namespace in QW.

---

## 1. Overview of Intrinsics in QW

The QW compiler handles two tiers of intrinsics:

1. **Primitive Type System (`core::types`):** Low-level hardware types (`i8`..`i128`, `u8`..`u128`, `b8`..`b128`, `f16`..`f128`, `bool`) injected by `qwc_intrinsic` during Phase 1.
2. **Compile-Time Reflection & Type Queries (`core::*`):** Special built-in functions evaluated directly by semantic analysis (`qwc_hir_gen`) and the constant evaluator (`qwc_comptime`). They carry zero runtime cost.

---

## 2. Compile-Time Type Inspection (`core::is_*`)

All type query functions in this section take generic type parameters, execute at compile time, and return a `bool` constant.

### 2.1 Scalar & Primitive Queries

- **`core::is_int<T>() -> bool`**: Returns `true` if `T` is any signed or unsigned primitive integer (`i8` through `u128`, `isize`, `usize`).
- **`core::is_signed<T>() -> bool`**: Returns `true` if `T` is a signed integer type (`i8`, `i16`, `i32`, `i64`, `i128`, `isize`).
- **`core::is_unsigned<T>() -> bool`**: Returns `true` if `T` is an unsigned integer type (`u8`, `u16`, `u32`, `u64`, `u128`, `usize`).
- **`core::is_float<T>() -> bool`**: Returns `true` if `T` is a floating-point type (`f16`, `f32`, `f64`, `f128`).
- **`core::is_bool<T>() -> bool`**: Returns `true` if `T` is `bool`.
- **`core::is_char<T>() -> bool`**: Returns `true` if `T` is a character (`char` / `u8`).
- **`core::is_void<T>() -> bool`**: Returns `true` if `T` is the unit type `()`.

### 2.2 Pointer, Reference & Sequence Queries

- **`core::is_pointer<T>() -> bool`**: Returns `true` if `T` is a raw pointer (`^U` or `^mut U`).
- **`core::is_reference<T>() -> bool`**: Returns `true` if `T` is a safe reference (`&U` or `&mut U`).
- **`core::is_array<T>() -> bool`**: Returns `true` if `T` is a fixed-size array (`[U; N]`).
- **`core::is_slice<T>() -> bool`**: Returns `true` if `T` is a dynamically-sized slice (`[U]`).
- **`core::is_vector<T>() -> bool`**: Returns `true` if `T` is a fixed SIMD vector (`[U * N]`) or scalable vector (`[U *]`).

### 2.3 User-Defined & Compound Type Queries

- **`core::is_struct<T>() -> bool`**: Returns `true` if `T` is a `struct`.
- **`core::is_iface<T>() -> bool`**: Returns `true` if `T` is an interface (`iface`).
- **`core::is_trait<T>() -> bool`**: Returns `true` if `T` is a `trait`.
- **`core::is_enum<T>() -> bool`**: Returns `true` if `T` is an `enum`.
- **`core::is_flags<T>() -> bool`**: Returns `true` if `T` is a bitwise `flags` type.
- **`core::is_variant<T>() -> bool`**: Returns `true` if `T` is a sum type (`variant`).
- **`core::is_function<T>() -> bool`**: Returns `true` if `T` is a function or task signature.

---

## 3. Type Size & Layout Queries

### `core::is_size<T>() -> usize`
Returns the byte size of type `T` according to the target architecture's data layout:
```qw
let int_size = core::is_size<i32>();   # Evaluates to 4 at compile time
```

### `core::is_align<T>() -> usize`
Returns the byte alignment requirement of type `T`:
```qw
let ptr_align = core::is_align<^u8>(); # Evaluates to 8 on 64-bit platforms
```

---

## 4. Type Relations & Equivalence

### `core::is_same<T, U>() -> bool`
Returns `true` if types `T` and `U` represent the exact same type:
```qw
let same = core::is_same<i32, i32>();  # true
let diff = core::is_same<i32, u32>();  # false
```

### `core::is_convertible<From, To>() -> bool`
Evaluates whether a value of type `From` can be converted or cast to type `To`:
```qw
let ok = core::is_convertible<i16, i32>(); # true
```

---

## 5. Compile-Time Cast (`core::cast`)

```qw
pub fun cast<To>(value: any) -> To;
```

Performs compile-time verified scalar and enumeration conversions:
- `enum <-> int`: Verifies that numeric values correspond to valid enum discriminants.
- `flags <-> int`: Verifies that combined flag masks are valid.
- If an overflow or out-of-bounds conversion occurs, the compiler generates a compile-time diagnostic error.
