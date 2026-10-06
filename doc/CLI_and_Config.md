# QW CLI & Configuration Reference

This document covers the command-line interface (`qw`), compiler diagnostic and dumping flags, package configuration (`qw.conf`), and standard project structure.

---

## 1. CLI Commands Overview

The QW compiler provides four primary commands via the `qw` binary:

```bash
qw <command> [OPTIONS]
```

| Command | Alias | Description |
| :--- | :---: | :--- |
| **`init`** | `i` | Initializes a new QW package with `qw.conf` and a starter `src/main.qw`. |
| **`build`**| `b` | Compiles the package to LLVM bitcode (`build/out.bc`) and a QW Unit (`build/out.qwu`). |
| **`run`**  | `r` | Compiles and executes the package in-process using the LLVM JIT VM. |
| **`check`**| `c` | Rapidly verifies syntax, scoping, and type validity without generating code. |

---

## 2. Command Details & Flags

### 2.1 `qw init`
Initializes a new QW package in the target directory:

```bash
qw init [OPTIONS] [path]
```

**Options:**
- `--path <PATH>`: Directory to initialize (default: current working directory `.`).
- `-n, --name <NAME>`: Package name (default: directory name).
- `-d, --desc <DESC>`: Package description string.
- `--no-git`: Skip automatic `git init` and `.gitignore` creation.
- `-f, --force`: Force initialization even if the destination directory is not empty.

**Example:**
```bash
qw init --name my_app --desc "Demo application" ./my_app
```

---

### 2.2 `qw build`
Orchestrates the four compiler passes to produce compiled binaries and intermediate artifacts:

```bash
qw build [OPTIONS]
```

**Options:**
- `--path <PATH>`: Path to the package root containing `qw.conf` (default: `.`).
- `--variant <VARIANT>`: Build variant: `debug` (default), `release`, or `relwithdebinfo`.
- `--dump <STAGES>`: Comma-separated list of compiler stages to dump (see [Section 3](#3-intermediate-representation-dumps---dump)).
- `--timings`: Displays execution duration for each compilation phase.
- `--usages`: Displays memory allocation and node counts across AST, HIR, and MIR arenas.
- `-v, --verbose`: Enables verbose compiler output. Can be passed multiple times (`-vv`) for granular phase metrics.

---

### 2.3 `qw run`
Compiles the application and executes it immediately via LLVM's in-memory ExecutionEngine (JIT):

```bash
qw run [OPTIONS]
```

Takes the same diagnostic flags as `qw build` (`--timings`, `--usages`, `-v`, `--dump`). Returns the integer exit code of the compiled program.

---

### 2.4 `qw check`
Runs Pass 1 (Lexing, Parsing, Scoping) and Pass 2 (HIR Lowering and Type Checking) without invoking MIR generation or LLVM:

```bash
qw check [OPTIONS]
```

Use `qw check` in editor integrations (such as the QW Language Server) and CI/CD pipelines for lightning-fast feedback on syntax and type errors.

---

## 3. Intermediate Representation Dumps (`--dump`)

The compiler allows inspecting internal state at each compilation phase via the `--dump` option:

```bash
qw build --dump ast,hir,mir
```

| Stage | Output | Description |
| :--- | :--- | :--- |
| **`ast`** | AST Tree | Formatted Abstract Syntax Tree with span locations and node IDs. |
| **`scope`** | Scope Map | Lexical scopes, identifier bindings, and resolved import paths. |
| **`hir`** | HIR Representation | High-level IR with resolved types, expressions, and namespace roots. |
| **`export`** | Export Map | Public, private, and internal symbol visibility table. |
| **`mir`** | MIR Control Flow Graph | Mid-level IR basic blocks, SSA instructions, and memory layouts. |
| **`lir`** | LLVM IR | Human-readable LLVM Intermediate Representation text. |

---

## 4. Compiler Performance Profiling

### Timing Diagnostics (`--timings`)
Passing `--timings` reports wall-clock execution time for each compilation pass:

```
timings: 2.14ms
  pass 1: 850µs
    parse: 620µs
    scope: 230µs
  pass 2: 710µs
    hgen: 540µs
    export: 170µs
  pass 3: 280µs
    mgen: 280µs
  pass 4: 300µs
    cgen: 300µs
```

### Memory & Arena Usage (`--usages -vv`)
Passing `--usages` displays active node memory vs allocated arena capacity:

```
usages: 34.20kb / 128.00kb
  ast: 14.10kb / 64.00kb
    type: 3.20kb
    expr: 7.10kb
    item: 2.40kb
  hir: 11.30kb / 32.00kb
  mir: 8.80kb / 32.00kb
```

---

## 5. Package Configuration (`qw.conf`)

QW projects are configured using a `qw.conf` file placed at the root of the project. The configuration format uses the lightweight Data Structure (`qwc_ds`) syntax:

```conf
name: "demo_project"
desc: "A high-performance systems service"
vers: "0.1.0"

wspace: {
  core: "../core"
}
```

### Configuration Keys:
- **`name`** *(String, Required)*: Package identifier.
- **`desc`** *(String, Optional)*: Package description.
- **`vers`** *(String, Required)*: Semantic version string (e.g. `"0.1.0"`).
- **`wspace`** *(Struct, Optional)*: Workspace paths mapping dependencies to local directories.

---

## 6. Standard Package Directory Layout

A standard QW project follows this structure:

```
my_project/
├── qw.conf              # Package manifest
├── src/
│   ├── main.qw          # Application entry point (or lib.qw for libraries)
│   ├── utils.qw         # Submodule (declared via `mod utils;`)
│   └── network/
│       └── mod.qw       # Nested directory module (declared via `mod network;`)
└── build/               # Generated build artifacts (ignored in git)
    ├── out.bc           # LLVM bitcode
    ├── out.ll           # Emitted textual LLVM IR (when --dump lir is passed)
    └── out.qwu          # Compiled binary QW Unit
```
