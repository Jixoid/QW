# QW Core Intrinsic Functions Specification (`sys::*`)

This document defines the built-in (intrinsic) functions under the `sys::` namespace in the QW programming language. These functions are evaluated directly by the compiler at compile time to perform type inspection, size calculations, and safe type conversions.

## 1. Size Information

### `sys::is_size`

* **Declaration (QW IDL):**
```qw
pub fun is_size<T>() -> usize;

```


* **Compile-time Semantics:** Returns the size of the given type `T` in bytes at compile time. (Note: This function is currently in the prototype stage and defaults to returning 8).



---

## 2. Type Comparison & Conversion

### `sys::is_same`

* **Declaration (QW IDL):**
```qw
pub fun is_same<T, U>() -> bool;

```


* **Compile-time Semantics:** Checks whether two provided types are completely identical at the compiler level. It performs a strict, literal match based on the names of the types.



### `sys::is_convertible`

* **Declaration (QW IDL):**
```qw
pub fun is_convertible<From, To>() -> bool;

```


* **Compile-time Semantics:** Evaluates whether an expression of type `From` can be implicitly converted to type `To`. This validation is performed by testing the compiler's semantic conversion and type assignment rules.



### `sys::cast`

* **Declaration (QW IDL):**
```qw
pub fun cast<To>(value: any) -> To;

```


* **Compile-time Semantics:** Converts a given value to the target type `To`. Currently, this intrinsic supports conversions between `enum <-> int` and `set <-> int`. During these conversions, the compiler strictly enforces compile-time bounds checking to ensure the value fits within the target integer type or maps to an existing valid element. If an overflow or out-of-bounds condition occurs, a compile-time error is generated.



---

## 3. Primitive Type Checking

The following intrinsic functions evaluate whether a type belongs to the compiler's primitive type taxonomy. All functions in this section return a `bool` and are resolved at compile time.

* **`sys::is_int<T>`**
* **IDL:** `pub fun is_int<T>() -> bool;`
* **Semantics:** Checks if `T` is one of the core primitive integer types ranging from `I8` to `U128`.




* **`sys::is_float<T>`**
* **IDL:** `pub fun is_float<T>() -> bool;`
* **Semantics:** Checks if `T` is a floating-point number type.




* **`sys::is_bool<T>`**
* **IDL:** `pub fun is_bool<T>() -> bool;`
* **Semantics:** Tests if `T` is exactly a boolean type.




* **`sys::is_char<T>`**
* **IDL:** `pub fun is_char<T>() -> bool;`
* **Semantics:** Checks if `T` is a character (`char`) type.




* **`sys::is_void<T>`**
* **IDL:** `pub fun is_void<T>() -> bool;`
* **Semantics:** Checks if `T` is a `void` type.




* **`sys::is_ptr<T>`**
* **IDL:** `pub fun is_ptr<T>() -> bool;`
* **Semantics:** Tests if `T` is a primitive `ptr` (raw memory address) type.




* **`sys::is_signed<T>`**
* **IDL:** `pub fun is_signed<T>() -> bool;`
* **Semantics:** Checks if `T` is a signed type, such as a signed integer.




* **`sys::is_unsigned<T>`**
* **IDL:** `pub fun is_unsigned<T>() -> bool;`
* **Semantics:** Tests if `T` is an unsigned type, such as an unsigned integer.





---

## 4. Compound Type Checking

The following intrinsic functions are used to identify complex or user-defined type structures. All functions in this section return a `bool` and are resolved at compile time.

* **`sys::is_pointer<T>`**
* **IDL:** `pub fun is_pointer<T>() -> bool;`
* **Semantics:** Checks if `T` is a derived pointer type.




* **`sys::is_reference<T>`**
* **IDL:** `pub fun is_reference<T>() -> bool;`
* **Semantics:** Checks if `T` is a reference type.




* **`sys::is_array<T>`**
* **IDL:** `pub fun is_array<T>() -> bool;`
* **Semantics:** Verifies if `T` is either a fixed-size array (`ZArrayType`) or a dynamic array (`PArrayType`).




* **`sys::is_struct<T>`**
* **IDL:** `pub fun is_struct<T>() -> bool;`
* **Semantics:** Checks if `T` is a record or struct type.




* **`sys::is_function<T>`**
* **IDL:** `pub fun is_function<T>() -> bool;`
* **Semantics:** Checks if `T` is a function type.




* **`sys::is_enum<T>`**
* **IDL:** `pub fun is_enum<T>() -> bool;`
* **Semantics:** Checks if `T` is an enum type.




* **`sys::is_set<T>`**
* **IDL:** `pub fun is_set<T>() -> bool;`
* **Semantics:** Checks if `T` is a set bitmask type.
