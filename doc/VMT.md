# QW Application Binary Interface (ABI) & Virtual Method Table (VMT) Specification

This document defines the official runtime layout and binary structure of Virtual Method Tables (VMT) for interfaces and structures in the QW programming language. 

All VMT entries are **word-sized** (i.e., 8 bytes on 64-bit architectures, 4 bytes on 32-bit architectures). Tables are presented using index offsets relative to the main VMT symbol pointer.

---

## 1. General Architecture & Offset Layout

The QW runtime uses a negative-offset metadata header strategy. The official VMT symbol pointer (`<Name>@vmt`) serves as the **anchor point (Index 0)**, which points directly to the first virtual function or structural metadata entry. 

- **Negative Offsets (Header / Metadata):** Contains Run-Time Type Information (RTTI), interface compliance tables, instance sizes, alignment rules, and destructor pointers.
- **Positive Offsets (Table Body):** Contains function pointers to virtual method implementations and member field offset metadata.

---

## 2. Interface (Iface) VMT Layout

Interfaces in QW dictate both behaviors (methods) and state requirements (properties). Since QW uses Fat Pointers for interfaces, property locations are dynamic and resolved via the VMT.

### 2.1 Single Interface with Properties

**Source Code:**
```qw
iface A {
  fun A();
  int V;
}
```


**Memory Layout:**

| Index (Word) | Offset      | Description                                                       |
| :----------: | :---------: | :---------------------------------------------------------------- |
| **-1**       | `-1 * word` | Type ID (RTTI) of `A`                                             |
| **0**        | `0`         | Pointer to `A::A` implementation (Anchor: `A@vmt`)                |
| **+1**       | `+1 * word` | Byte offset of property `V` relative to the struct's data pointer |

### 2.2 Multiple Interface Inheritance

When an interface inherits from multiple other interfaces, its negative offsets contain pointers to the pure VMTs of its base interfaces, allowing for zero-cost upcasting.

**Source Code:**
```qw
iface B {
  fun B();
}

iface C: A, B {
  fun C();
}
```

**Memory Layout for `C`:**

| Index (Word) |   Offset    | Description                                   |
| :----------: | :---------: | :-------------------------------------------- |
|    **-6**    | `-6 * word` | Pointer to `A@vmt` (For upcasting `C` to `A`) |
|    **-5**    | `-5 * word` | Pointer to `B@vmt` (For upcasting `C` to `B`) |
|    **-4**    | `-4 * word` | Type ID of `A`                                |
|    **-3**    | `-3 * word` | Type ID of `B`                                |
|    **-2**    | `-2 * word` | Number of base interfaces (Value: 2)          |
|    **-1**    | `-1 * word` | Type ID (RTTI) of `C`                         |
|    **0**     |     `0`     | Pointer to `A::A` (Anchor: `C@vmt`)           |
|    **+1**    | `+1 * word` | Byte offset of property `V`                   |
|    **+2**    | `+2 * word` | Pointer to `B::B`                             |
|    **+3**    | `+3 * word` | Pointer to `C::C`                             |

When a Fat Pointer of type `C` is upcast to `B`, the runtime merely reads the VMT pointer at index `-5` and creates a new Fat Pointer with the same data address but the VMT of `B`.

---

## 3. Structure (Struct) VMT Layout

Structures in QW utilize single-inheritance for data but can implement unlimited interfaces. When a struct implements virtual methods, it is assigned a VMT that unifies structural memory management with polymorphic interface behavior.

### 3.1 Polymorphic Struct implementing Interfaces

**Source Code:**
```qw
struct Impl: A, B {
  fun C() virtual;
  int V;
}
```

**Memory Layout for `Impl`:**

| Index (Word) | Offset | Description |
| :---: | :---: | :--- |
| **-9** | `-9 * word` | Pointer to `A@vmt` |
| **-8** | `-8 * word` | Pointer to `B@vmt` |
| **-7** | `-7 * word` | Type ID of `A` |
| **-6** | `-6 * word` | Type ID of `B` |
| **-5** | `-5 * word` | Number of base interfaces (Value: 2) |
| **-4** | `-4 * word` | Size of `Impl` instance in bytes (Memory Allocation) |
| **-3** | `-3 * word` | Alignment of `Impl` instance |
| **-2** | `-2 * word` | Pointer to `Impl::fini` (Virtual Destructor) |
| **-1** | `-1 * word` | Type ID (RTTI) of `Impl` |
| **0** | `0` | Pointer to `Impl::A` (Anchor: `Impl@vmt`) |
| **+1** | `+1 * word` | Byte offset of property `V` inside `Impl` |
| **+2** | `+2 * word` | Pointer to `Impl::B` |
| **+3** | `+3 * word` | Pointer to `Impl::C` |

The structure's VMT is directly compatible with the interface VMT expectations. Interface references to `Impl` will naturally access the required function pointers and property offsets at the predefined positive indices, while runtime systems (like memory allocators or `sys::cast`) can safely query the structural metadata located at the deeper negative indices.
