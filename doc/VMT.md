# QW Application Binary Interface (ABI) & Virtual Method Table (VMT) Specification

This document defines the runtime binary layout, memory structure, and dispatch mechanisms of Virtual Method Tables (VMT) and Fat Pointers in the QW programming language.

---

## 1. General Architecture & Offset Layout

All VMT entries are **word-sized** (8 bytes on 64-bit architectures, 4 bytes on 32-bit architectures). The QW runtime utilizes an **anchor-point strategy**:

```
                         Negative Offsets (Metadata & RTTI Header)
                         -----------------------------------------
           [-N * word]   Base Interface VMT Pointers (for upcasting)
           ...
           [-4 * word]   Instance Byte Size (Memory allocation)
           [-3 * word]   Instance Alignment
           [-2 * word]   Virtual Destructor Pointer (`fini`)
           [-1 * word]   Run-Time Type Information (RTTI / Type ID)
    Anchor -> [ 0    ]   First Virtual Method Function Pointer
                         -----------------------------------------
           [+1 * word]   Property Offset / Second Virtual Method
           [+2 * word]   Virtual Method Pointer 2
           ...
                         Positive Offsets (Method Body & Properties)
```

The official VMT symbol pointer (`<Name>@vmt`) serves as the **anchor point (Index 0)**, pointing directly to the first virtual function.
- **Negative Offsets (Header):** Metadata, type identification, inheritance hierarchy pointers, allocator metrics, and finalizer hooks.
- **Positive Offsets (Body):** Executable function pointers and byte offsets for dynamic interface properties.

---

## 2. Fat Pointer Representation

Interface references (`&Iface` and `&mut Iface`) are represented as **Fat Pointers** occupying two machine words (16 bytes on 64-bit platforms):

```
+-----------------------------------+-----------------------------------+
|            Data Pointer           |            VMT Pointer            |
|              (^void)              |              (^VMT)               |
+-----------------------------------+-----------------------------------+
```

1. **`data_ptr`**: Points directly to the allocated struct instance data in memory.
2. **`vmt_ptr`**: Points to the anchor point (Index 0) of the corresponding VMT.

### Dynamic Dispatch Call Mechanics
When invoking a virtual method at index `+k`:
```
fn_ptr   = *(vmt_ptr + k * sizeof(word))
ret_val  = fn_ptr(data_ptr, arg1, arg2, ...)
```

---

## 3. Interface (Iface) VMT Layout

QW interfaces support both method signatures and required property fields. Because interface layouts are abstract, property locations in implementing structs are resolved at runtime via the VMT.

### 3.1 Single Interface with Properties

```qw
iface Stream {
  fun write(buf: ^u8, len: usize) -> usize;
  status: i32;
}
```

**Memory Layout:**

| Index (Word) | Offset | Description |
| :---: | :---: | :--- |
| **-1** | `-1 * word` | Type ID (RTTI) of `Stream` |
| **0** | `0` | Pointer to `Stream::write` implementation (Anchor: `Stream@vmt`) |
| **+1** | `+1 * word` | Byte offset of property `status` relative to the struct's base address |

### 3.2 Multiple Interface Inheritance & Zero-Cost Upcasting

When an interface inherits from multiple parent interfaces, negative offsets contain pointers to the base interface VMTs:

```qw
iface Reader {
  fun read(buf: ^u8, len: usize) -> usize;
}

iface ReadWriteStream: Stream, Reader {
  fun flush();
}
```

**Memory Layout for `ReadWriteStream`:**

| Index (Word) | Offset | Description |
| :---: | :---: | :--- |
| **-6** | `-6 * word` | Pointer to `Stream@vmt` (Enables zero-cost upcast to `Stream`) |
| **-5** | `-5 * word` | Pointer to `Reader@vmt` (Enables zero-cost upcast to `Reader`) |
| **-4** | `-4 * word` | Type ID of `Stream` |
| **-3** | `-3 * word` | Type ID of `Reader` |
| **-2** | `-2 * word` | Number of base interfaces (Value: 2) |
| **-1** | `-1 * word` | Type ID (RTTI) of `ReadWriteStream` |
| **0** | `0` | Pointer to `Stream::write` (Anchor: `ReadWriteStream@vmt`) |
| **+1** | `+1 * word` | Byte offset of property `status` |
| **+2** | `+2 * word` | Pointer to `Reader::read` |
| **+3** | `+3 * word` | Pointer to `ReadWriteStream::flush` |

**Zero-Cost Upcasting:** To cast a `ReadWriteStream` Fat Pointer to a `Reader` Fat Pointer, the runtime reads the base VMT pointer from index `-5` and creates a new Fat Pointer with the same `data_ptr` and the target `vmt_ptr`.

---

## 4. Structure (Struct) VMT Layout

Structures have a concrete value layout in memory. When a structure implements one or more interfaces, a VMT is synthesized linking structural layout information with interface function pointers:

```qw
struct FileStream: Stream, Reader {
  handle: i64;
  status: i32;

  fun write(buf: ^u8, len: usize) -> usize { ... }
  fun read(buf: ^u8, len: usize) -> usize { ... }
  fini close() { ... }
}
```

**Memory Layout for `FileStream`:**

| Index (Word) | Offset | Description |
| :---: | :---: | :--- |
| **-9** | `-9 * word` | Pointer to `Stream@vmt` |
| **-8** | `-8 * word` | Pointer to `Reader@vmt` |
| **-7** | `-7 * word` | Type ID of `Stream` |
| **-6** | `-6 * word` | Type ID of `Reader` |
| **-5** | `-5 * word` | Number of base interfaces (Value: 2) |
| **-4** | `-4 * word` | Total size of `FileStream` instance in bytes (Memory allocation) |
| **-3** | `-3 * word` | Required alignment of `FileStream` |
| **-2** | `-2 * word` | Pointer to destructor `FileStream::fini` |
| **-1** | `-1 * word` | Type ID (RTTI) of `FileStream` |
| **0** | `0` | Pointer to `FileStream::write` (Anchor) |
| **+1** | `+1 * word` | Byte offset of `status` within `FileStream` (`8` bytes) |
| **+2** | `+2 * word` | Pointer to `FileStream::read` |

---

## 5. Compiler Implementation Status

### Current Implementation (`qwc_mir_gen` & `qwc_cgen_llvm`)
- **VMT Caching:** In `qwc_mir_gen::symb_p`, VMT symbols are constructed and memoized per `(struct_ty, iface_ty)` pair.
- **LLVM Global Constant:** `qwc_cgen_llvm` emits VMTs as constant global structs initialized with:
  ```llvm
  @qw_vmt_FileStream_Stream = private constant { i64, i64, ptr, ptr } {
    i64 16,        ; instance size
    i64 8,         ; alignment
    ptr null,      ; virtual destructor pointer
    ptr @qw_impl_FileStream_Streamwrite ; method pointer
  }
  ```
- **Fat Pointer Boxing (`as &Iface`):** Lowers in MIR to a 2-word stack allocation containing `{ data_address, vmt_global_ref }`.
- **Target ABI Evolution:** Transitioning towards runtime negative-offset GEP indexing to support zero-cost polymorphic upcasting and multi-interface dispatch tables.
