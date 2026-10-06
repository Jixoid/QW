# QW Language Syntax Grammar (PEG)

This document defines the formal syntax and lexical grammar of the QW programming language in Parsing Expression Grammar (PEG) format, accurately reflecting the reference compiler implementation.

---

## 1. Lexical Rules

```peg
Whitespace     <- [ \t\n\r]+
LineComment    <- "//" (!'\n' .)* '\n'
BlockComment   <- "/*" (BlockComment / (!"*/" .))* "*/"
Spacing        <- (Whitespace / LineComment / BlockComment)*

Keyword        <- Pub / Priv / Prot / Crate / Super / Mut
                / Let / Var / Using / Fun / Task / Struct / Iface / Trait 
                / Enum / Flags / Variant / Init / Fini / Impl / Generic / Use / Mod
                / If / Ef / Else / Match / Loop / While / For / In
                / Ret / Break / Continue / Die / As
                / Unsafe / Relaxed / True / False / Undef / Unreachable
                / SelfB / SelfS / TypeKwd

Identifier     <- !Keyword [a-zA-Z_] [a-zA-Z0-9_]* Spacing
IntegerLiteral <- [0-9]+ ('.' [0-9]+)? Spacing
StringLiteral  <- '"' (!'"' ('\\' . / .))* '"' Spacing
CharLiteral    <- '\'' (!'\'' ('\\' . / .))* '\'' Spacing
EOF            <- !.
```

---

## 2. Keywords

```peg
# Visibility & Modifiers
Pub         <- "pub" Spacing
Priv        <- "priv" Spacing
Prot        <- "prot" Spacing
Crate       <- "crate" Spacing
Super       <- "super" Spacing
Mut         <- "mut" Spacing

# Declarations & Definitions
Let         <- "let" Spacing
Var         <- "var" Spacing
Using       <- "using" Spacing
Fun         <- "fun" Spacing
Task        <- "task" Spacing
Struct      <- "struct" Spacing
Iface       <- "iface" Spacing
Trait       <- "trait" Spacing
Enum        <- "enum" Spacing
Flags       <- "flags" Spacing
Variant     <- "variant" Spacing
Init        <- "init" Spacing
Fini        <- "fini" Spacing
Impl        <- "impl" Spacing
Generic     <- "generic" Spacing
Requires    <- "requires" Spacing
Use         <- "use" Spacing
Mod         <- "mod" Spacing

# Control Flow & Expressions
If          <- "if" Spacing
Ef          <- "ef" Spacing
Else        <- "else" Spacing
Match       <- "match" Spacing
Loop        <- "loop" Spacing
While       <- "while" Spacing
For         <- "for" Spacing
In          <- "in" Spacing
Ret         <- "ret" Spacing
Break       <- "break" Spacing
Continue    <- "continue" Spacing
Die         <- "die" Spacing
As          <- "as" Spacing

# Contexts & Intrinsics
Unsafe      <- "unsafe" Spacing
Relaxed     <- "relaxed" Spacing
True        <- "true" Spacing
False       <- "false" Spacing
Undef       <- "undef" Spacing
Unreachable <- "unreachable" Spacing
SelfB       <- "Self" Spacing
SelfS       <- "self" Spacing
TypeKwd     <- "type" Spacing
```

---

## 3. Attributes

Attributes provide compiler directives, code generation flags, and ABI control. They are declared with `![ ... ]` (or `#[ ... ]`):

```peg
Attributes  <- ( BangAttr / HashAttr ) AttributeList ']' Spacing
BangAttr    <- "![" Spacing
HashAttr    <- "#[" Spacing
AttributeList <- Attribute ( ',' Spacing Attribute )* ','? Spacing

Attribute   <- Identifier ':' Spacing Identifier          # Key-Value, e.g. ![calling: fast]
             / Identifier '(' Spacing AttributeList? ')'   # List, e.g. ![feature(foo, bar)]
             / Identifier '=' Spacing Expression          # Assignment, e.g. ![align = 16]
             / Identifier                                 # Flag, e.g. ![weak]
```

### Supported Attributes

- **`![C]`**: Enforces C ABI calling convention and linkage for external function bindings.
- **`![symbol: bare]`**: Disables name mangling for the declared symbol (exported raw).
- **`![symbol: qw]`**: Enforces standard QW name mangling.
- **`![weak]`**: Emits a weak linker symbol.
- **`![calling: <convention>]`**: Specifies the function calling convention:
  - `fast`: LLVM Fast calling convention.
  - `cdecl`: Standard C calling convention.
  - `cold`: Optimizes code paths for rarely executed functions.
- **`![entry]`**: Explicitly marks the package execution entry point.
- **`![default: allocator]`**: Registers the struct as the default memory allocator.

---

## 4. Top-Level Declarations & Routes

A QW source file consists of a sequence of top-level declarations (routes). Scope-wide default visibility can be set via block labels like `pub:` or `priv:`.

```peg
File        <- Spacing VisibilityBlock* EOF

VisibilityBlock <- ( Visibility ':' Spacing )? Route*

Visibility  <- Pub / Priv / Prot / Crate / Super

Route       <- Attributes? Visibility? Declaration

Declaration <- VarDecl
             / UsingDecl
             / FuncDecl
             / TaskDecl
             / StructDecl
             / IfaceDecl
             / TraitDecl
             / EnumDecl
             / FlagsDecl
             / VariantDecl
             / ImplDecl
             / GenericDecl
             / UseDecl
             / ModDecl
```

---

## 5. Declarations

### Variable Declarations
```peg
VarDecl     <- ( Let / Var ) Identifier ( ':' Spacing Type )? ( '=' Spacing Expression )? ';' Spacing
```
- `let` introduces an immutable binding.
- `var` introduces a mutable binding.

### Type Aliases (`using`)
```peg
UsingDecl   <- Using Identifier '=' Spacing Type ';' Spacing
```

### Function & Task Declarations
```peg
FuncDecl    <- Fun Identifier '(' Spacing ParamList? ')' Spacing ( "->" Spacing Type )? ( CodeBlock / ';' Spacing )
TaskDecl    <- Task Identifier '(' Spacing ParamList? ')' Spacing ( "->" Spacing Type )? ( CodeBlock / ';' Spacing )

ParamList   <- ( SelfParam ','? Spacing )? ( ParamGroup ( ',' Spacing ParamGroup )* )?
SelfParam   <- ( '&' Spacing Mut? Spacing / Mut? Spacing )? SelfS
ParamGroup  <- Identifier ( ',' Spacing Identifier )* ':' Spacing Type
```

### Structure Declarations (`struct`)
```peg
StructDecl    <- Struct Identifier ( ':' Spacing BaseList )? '{' Spacing StructMember* '}' Spacing
BaseList      <- Type ( ',' Spacing Type )*

StructMember  <- ( Visibility ':' Spacing )? Attributes? Visibility? ( MemberVar / MemberType / StructFunc / StructInit / StructFini / InlineImpl )
MemberVar     <- Identifier ':' Spacing Type ';' Spacing
MemberType    <- TypeKwd Identifier ( '=' Spacing Type )? ';' Spacing
StructFunc    <- FuncDecl
StructInit    <- Init Identifier '(' Spacing ParamList? ')' Spacing ( ':' Spacing InitList )? ( CodeBlock / ';' Spacing )
InitList      <- InitItem ( ',' Spacing InitItem )*
InitItem      <- Identifier '(' Spacing Expression ')' Spacing
StructFini    <- Fini Identifier '(' Spacing ParamList? ')' Spacing ( CodeBlock / ';' Spacing )
InlineImpl    <- Impl ':' Spacing Type '{' Spacing StructFunc* '}' Spacing
```

### Interfaces & Traits (`iface`, `trait`)
```peg
IfaceDecl     <- Iface Identifier ( ':' Spacing BaseList )? '{' Spacing IfaceMember* '}' Spacing
IfaceMember   <- ( Visibility ':' Spacing )? Attributes? Visibility? ( MemberVar / StructFunc )

TraitDecl     <- Trait Identifier ( ':' Spacing BaseList )? '{' Spacing TraitMember* '}' Spacing
TraitMember   <- ( Visibility ':' Spacing )? Attributes? Visibility? StructFunc
```

### Enums, Flags & Variants
```peg
EnumDecl      <- Enum Identifier '{' Spacing EnumList? '}' Spacing
EnumList      <- EnumItem ( ',' Spacing EnumItem )* ','? Spacing
EnumItem      <- Identifier ( '=' Spacing Expression )?

FlagsDecl     <- Flags Identifier '{' Spacing EnumList? '}' Spacing

VariantDecl   <- Variant Identifier '{' Spacing VariantList? '}' Spacing
VariantList   <- VariantItem ( ',' Spacing VariantItem )* ','? Spacing
VariantItem   <- Identifier ( '(' Spacing Type ( ',' Spacing Type )* ')' )?
```

### Implementations (`impl`)
```peg
ImplDecl      <- Impl Type ( ':' Spacing Type )? '{' Spacing StructFunc* '}' Spacing
```

### Generics
```peg
GenericDecl   <- Generic '<' Spacing GenericGroup ( ',' Spacing GenericGroup )* '>' Spacing GenericReqs? ( '{' Spacing Route* '}' Spacing / Route )
GenericGroup  <- Identifier ( ',' Spacing Identifier )* ( ':' Spacing Type )?
GenericReqs   <- Requires ReqItem ( ';' Spacing ReqItem )* ';'? Spacing
ReqItem       <- Identifier ':' Spacing Type ( '|' Spacing Type )*
```

### Modules & Imports
```peg
ModDecl       <- Mod Identifier ( ';' Spacing / '{' Spacing Route* '}' Spacing )
UseDecl       <- Use UsePath ';' Spacing
UsePath       <- ( Crate / Super / Identifier ) ( "::" Spacing ( Identifier / '*' ) )*
```

---

## 6. Type Grammar

Types in QW are specified cleanly with prefix modifiers for pointers, references, options, and errors, and bracketed constructs for collections and vectors.

```peg
Type          <- PrimaryType ( PostPath / PostSpecialize )*

PrimaryType   <- OptionType
               / FailType
               / RangeType
               / PtrType
               / RefType
               / SliceArrayVecType
               / TupleType
               / UnitType
               / FuncType
               / TaskType
               / SelfB
               / TypeKwd
               / Identifier

OptionType    <- '?' Spacing Type                         # ?T (Option)
FailType      <- '!' Spacing Type                         # !T (Fail / Error)
RangeType     <- ".." Spacing Type                        # ..T (Range)

PtrType       <- '^' Spacing Mut? Type                    # ^T or ^mut T (Raw Pointer)
RefType       <- '&' Spacing Mut? Type                    # &T or &mut T (Reference)

SliceArrayVecType <- '[' Spacing Type (
                    ']' Spacing                           # [T] (Slice)
                  / ';' Spacing Expression ']' Spacing    # [T; N] (Fixed Array)
                  / '*' Spacing Expression ']' Spacing    # [T * N] (SIMD Vector)
                  / '*' Spacing ']' Spacing               # [T *] (Scalable Vector)
                )

TupleType     <- '(' Spacing Type ( ',' Spacing Type )+ ','? ')' Spacing
UnitType      <- '(' ')' Spacing

FuncType      <- Fun '(' Spacing TypeArgs? ')' Spacing ( "->" Spacing Type )?
TaskType      <- Task '(' Spacing TypeArgs? ')' Spacing ( "->" Spacing Type )?

PostPath      <- "::" Spacing Identifier
PostSpecialize<- '<' Spacing TypeArgs? '>' Spacing
TypeArgs      <- ( Type / Expression ) ( ',' Spacing ( Type / Expression ) )*
```

---

## 7. Statements & Expressions

In QW, expressions and statements are unified: almost every statement evaluates to an expression. Blocks evaluate to the value of their trailing expression (omitting `;`).

```peg
CodeBlock     <- ( '`' Identifier ':' Spacing )? '{' Spacing Statement* Expression? '}' Spacing

Statement     <- VarDecl
               / Expression ';' Spacing
               / CodeBlock
               / ControlExpr

Expression    <- AssignmentExpr

AssignmentExpr<- LogicalOrExpr ( AssignOp Spacing LogicalOrExpr )*
AssignOp      <- "=" / "<-" 
               / "+=" / "-=" / "*=" / "/=" / "%=" 
               / "<<=" / ">>=" / "&&=" / "||=" / "^^="

LogicalOrExpr <- LogicalXorExpr ( "||" Spacing LogicalXorExpr )*
LogicalXorExpr<- LogicalAndExpr ( "^^" Spacing LogicalAndExpr )*
LogicalAndExpr<- EqualityExpr ( "&&" Spacing EqualityExpr )*
EqualityExpr  <- RelationalExpr ( ( "==" / "!=" ) Spacing RelationalExpr )*
RelationalExpr<- PipeExpr ( ( "<=" / ">=" / "<" / ">" ) Spacing PipeExpr )*
PipeExpr      <- RangeExpr ( "|" Spacing RangeExpr )*
RangeExpr     <- ShiftExpr ( ( ".." / "..=" ) Spacing ShiftExpr )*
ShiftExpr     <- AdditiveExpr ( ( "<<" / ">>" ) Spacing AdditiveExpr )*
AdditiveExpr  <- MultiplicativeExpr ( ( "+" / "-" ) Spacing MultiplicativeExpr )*
MultiplicativeExpr <- UnaryExpr ( ( "*" / "/" / "%" ) Spacing UnaryExpr )*

UnaryExpr     <- ( "-" / "+" / "!" ) Spacing UnaryExpr    # Prefix Unary
               / PostfixExpr

PostfixExpr   <- PrimaryExpr (
                   '.' Spacing Identifier                 # Member Access
                 / "::" Spacing Identifier                # Scope Access
                 / "::" '<' Spacing TypeArgs? '>' Spacing # Specialization
                 / '(' Spacing Args? ')' Spacing          # Function Call
                 / '[' Spacing Args? ']' Spacing          # Indexing
                 / As Spacing Type                        # Type Cast
                 / '?' Spacing                            # Try / Propagate
                 / "!!" Spacing                           # Force Unwrap
                 / '&' Spacing                            # Address-of
                 / '^' Spacing                            # Dereference
                 / StructInitBlock                        # Struct Field Init
                 )*

StructInitBlock <- '{' Spacing ( FieldInit ( ',' Spacing FieldInit )* ','? )? '}' Spacing
FieldInit     <- Identifier ':' Spacing Expression

PrimaryExpr   <- IntegerLiteral
               / StringLiteral
               / CharLiteral
               / True / False / Undef / Unreachable
               / SelfB / SelfS
               / Identifier
               / '(' Spacing Expression ')' Spacing
               / CodeBlock
               / ControlExpr

Args          <- Expression ( ',' Spacing Expression )*
```

---

## 8. Control Flow Expressions

```peg
ControlExpr   <- IfExpr
               / MatchExpr
               / WhileExpr
               / LoopExpr
               / ForExpr
               / JumpExpr
               / UnsafeExpr
               / RelaxedExpr

IfExpr        <- If Expression CodeBlock ( Ef Expression CodeBlock )* ( Else CodeBlock )?
MatchExpr     <- Match Expression '{' Spacing MatchArm* '}' Spacing
MatchArm      <- Pattern "=>" Spacing Expression ( ',' Spacing / & '}' )

WhileExpr     <- While Expression CodeBlock ( Else CodeBlock )?
LoopExpr      <- Loop CodeBlock ( Else CodeBlock )?
ForExpr       <- For Pattern In Expression CodeBlock ( Else CodeBlock )?

JumpExpr      <- Ret ( '`' Identifier )? Expression?
               / Break ( '`' Identifier )? Expression?
               / Continue ( '`' Identifier )?
               / Die

UnsafeExpr    <- Unsafe CodeBlock
RelaxedExpr   <- Relaxed CodeBlock
```

---

## 9. Patterns

Patterns are used in variable bindings (`let`, `var`), function parameters, and `match` arms:

```peg
Pattern       <- Identifier                               # Variable Bind
               / '_' Spacing                              # Wildcard Ignored
               / ".." Spacing                             # Rest Pattern
               / '(' Spacing PatternList? ')' Spacing     # Tuple Pattern
               / '[' Spacing PatternList? ']' Spacing     # Array Pattern

PatternList   <- Pattern ( ',' Spacing Pattern )* ','? Spacing
```

---

## 10. Operator Precedence & Associativity Table

The table below summarizes all QW binary and unary operators from lowest to highest precedence:

| Precedence | Operator | Description | Associativity |
| :---: | :--- | :--- | :---: |
| **1 (Lowest)** | `=` | Assignment | Right-to-Left |
| | `<-` | Exchange / Swap | Right-to-Left |
| | `+=`, `-=`, `*=`, `/=`, `%=`, `<<=`, `>>=`, `&&=`, `\|\|=`, `^^=` | Compound Assignment | Right-to-Left |
| **2** | `\|\|` | Logical OR | Left-to-Right |
| **3** | `^^` | Logical XOR | Left-to-Right |
| **4** | `&&` | Logical AND | Left-to-Right |
| **5** | `==`, `!=` | Equality, Inequality | Left-to-Right |
| **6** | `<`, `>`, `<=`, `>=` | Relational Comparisons | Left-to-Right |
| **7** | `\|` | Pipe Operator / Stream | Left-to-Right |
| **8** | `..`, `..=` | Exclusive and Inclusive Ranges | Left-to-Right |
| **9** | `<<`, `>>` | Bitwise Shifts | Left-to-Right |
| **10** | `+`, `-` | Addition, Subtraction | Left-to-Right |
| **11** | `*`, `/`, `%` | Multiplication, Division, Remainder | Left-to-Right |
| **12** | `-`, `+`, `!` (Prefix) | Negation, Identity, Logical NOT | Right-to-Left (Prefix) |
| | `?`, `!!`, `&`, `^` (Postfix) | Try, Force Unwrap, Address-of, Dereference | Left-to-Right (Postfix) |
| **13 (Highest)**| `.`, `::`, `()`, `[]`, `as`, `{ fields }` | Member, Scope, Call, Index, Cast, Struct Init | Left-to-Right |
