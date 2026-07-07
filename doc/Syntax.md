# QW Language Syntax Grammar (PEG)

This document defines the syntax of the QW programming language in Parsing Expression Grammar (PEG) format.

## Lexical Rules

```peg
Whitespace     <- [ \t\n\r]+
Comment        <- '#' (!'\n' .)* '\n'
Spacing        <- (Whitespace / Comment)*

Keyword        <- Pub / Priv / Prot / Crate / Group 
                / Mod / Namespace / Alias / Var / TypeKwd 
                / Fun / Struct / Iface / Enum / Set / Init / Fini 
                / Let / If / Else / Ef / While / Break / Continue / Ret

Identifier     <- !Keyword [a-zA-Z_] [a-zA-Z0-9_]* Spacing
IntegerLiteral <- [0-9]+ Spacing
StringLiteral  <- '"' (!'"' .)* '"' Spacing
CharLiteral    <- '\'' (!'\'' .)* '\'' Spacing
EOF            <- !.
```

## Keywords

```peg
Pub       <- "pub" Spacing
Priv      <- "priv" Spacing
Prot      <- "prot" Spacing
Crate     <- "crate" Spacing
Group     <- "group" Spacing

Mod       <- "mod" Spacing
Namespace <- "namespace" Spacing
Alias     <- "alias" Spacing
Var       <- "var" Spacing
TypeKwd   <- "type" Spacing
Fun       <- "fun" Spacing
Struct    <- "struct" Spacing
Iface     <- "iface" Spacing
Enum      <- "enum" Spacing
Set       <- "set" Spacing
Init      <- "init" Spacing
Fini      <- "fini" Spacing

Let       <- "let" Spacing
If        <- "if" Spacing
Else      <- "else" Spacing
Ef        <- "ef" Spacing
While     <- "while" Spacing
Break     <- "break" Spacing
Continue  <- "continue" Spacing
Ret       <- "ret" Spacing
```

## Top Level & Route

In QW, a file contains a series of "Routes" (declarations). Each declaration can optionally take visibility modifiers and attribute lists. Attributes are specified within double brackets `[[ ... ]]`.

```peg
File       <- Spacing Route* EOF

Route      <- Attributes? Visibility? Declaration
Visibility <- Pub / Priv / Prot / Crate / Group
Attributes <- "[[" Spacing Attribute (',' Spacing Attribute)* "]]" Spacing
Attribute  <- Identifier (':' Spacing Identifier)?

*(Note: See the "Attributes" section below for supported attributes and their semantics.)*

Declaration <- NamespaceDecl 
             / AliasDecl 
             / VarDecl 
             / TypeDecl 
             / FuncDecl 
             / StructDecl 
             / IfaceDecl 
             / EnumDecl 
             / SetDecl 
             / ModDecl
```

## Declarations

```peg
ModDecl       <- Mod Identifier ';' Spacing

NamespaceDecl <- Namespace Identifier '{' Spacing Route* '}' Spacing

AliasDecl     <- Alias Identifier '=' Identifier ';' Spacing

VarDecl       <- Var Identifier (',' Spacing Identifier)* ':' Spacing Type ( '=' Spacing Expression )? ';' Spacing

TypeDecl      <- TypeKwd Identifier '=' Spacing Type ';' Spacing

FuncDecl      <- Fun Identifier '(' Spacing FuncParams? ')' Spacing ( "->" Spacing Type )? ( ';' Spacing / CodeBlock )
FuncParams    <- ParamGroup ( ';' Spacing ParamGroup )*
ParamGroup    <- Identifier ( ',' Spacing Identifier )* ':' Spacing Type

StructDecl    <- Struct Identifier ( ':' Spacing Type (',' Spacing Type)* )? '{' Spacing StructMember* '}' Spacing
StructMember  <- Visibility? ( StructVarDecl / StructFuncDecl / StructInitDecl / StructFiniDecl )
StructVarDecl <- Identifier (',' Spacing Identifier)* ':' Spacing Type ';' Spacing
StructFuncDecl<- FuncDecl
StructInitDecl<- Init '(' Spacing FuncParams? ')' Spacing ( ':' Spacing StructInitList )? CodeBlock
StructInitList<- StructInitItem ( ',' Spacing StructInitItem )*
StructInitItem<- Identifier '(' Spacing Expression ')' Spacing
StructFiniDecl<- Fini '(' Spacing FuncParams? ')' Spacing CodeBlock

IfaceDecl     <- Iface Identifier ( ':' Spacing Type (',' Spacing Type)* )? '{' Spacing IfaceMember* '}' Spacing
IfaceMember   <- Visibility? FuncDecl

EnumDecl      <- Enum Identifier ( ':' Spacing Type )? '{' Spacing EnumConst (',' Spacing EnumConst)* ','? Spacing '}' Spacing
EnumConst     <- Identifier ( '=' Spacing IntegerLiteral )?

SetDecl       <- Set Identifier ( ':' Spacing Type )? '{' Spacing EnumConst (',' Spacing EnumConst)* ','? Spacing '}' Spacing
```

## Types

In addition to basic types, QW also supports anonymous struct/iface types. Type modifiers such as pointer, reference, and array come after the type name (`Type^`, `Type&`, `Type[]`).

```peg
Type          <- ( StructType / IfaceType / FuncType / NickType ) TypeModifiers*
StructType    <- Struct '{' Spacing StructMember* '}' Spacing
IfaceType     <- Iface '{' Spacing IfaceMember* '}' Spacing
FuncType      <- Fun '(' Spacing FuncParams? ')' Spacing ( "->" Spacing Type )?
NickType      <- Identifier ( "::" Spacing Identifier )*

TypeModifiers <- PointerMod / RefMod / ArrayMod
PointerMod    <- '^' Spacing
RefMod        <- '&' Spacing
ArrayMod      <- '[' Spacing IntegerLiteral? ']' Spacing
```

## Statements

```peg
CodeBlock     <- '{' Spacing Statement* '}' Spacing

Statement     <- CodeBlock
               / IfStmt
               / WhileStmt
               / VarStmt
               / LetStmt
               / RetStmt
               / BreakStmt
               / ContinueStmt
               / ExprStmt

IfStmt        <- If '(' Spacing Expression ')' Spacing CodeBlock ElseIfStmt* ElseStmt?
ElseIfStmt    <- ( Else If / Ef ) '(' Spacing Expression ')' Spacing CodeBlock
ElseStmt      <- Else CodeBlock

WhileStmt     <- While '(' Spacing Expression ')' Spacing CodeBlock

VarStmt       <- Var Identifier (',' Spacing Identifier)* ':' Spacing Type ( '=' Spacing Expression )? ';' Spacing
LetStmt       <- Let Identifier (',' Spacing Identifier)* ':' Spacing Type ( '=' Spacing Expression )? ';' Spacing
RetStmt       <- Ret Expression? ';' Spacing
BreakStmt     <- Break ';' Spacing
ContinueStmt  <- Continue ';' Spacing
ExprStmt      <- Expression ';' Spacing
```

## Expressions & Precedence

```peg
Expression         <- AssignmentExpr
AssignmentExpr     <- LogicalOrExpr ( AssignmentOp Spacing LogicalOrExpr )*
AssignmentOp       <- ( "=" / "+=" / "-=" / "*=" / "/=" / "%=" / "&=" / "|=" / "^=" / "<<=" / ">>=" ) Spacing
LogicalOrExpr      <- LogicalAndExpr ( "||" Spacing LogicalAndExpr )*
LogicalAndExpr     <- BitwiseOrExpr ( "&&" Spacing BitwiseOrExpr )*
BitwiseOrExpr      <- BitwiseXorExpr ( "|" Spacing BitwiseXorExpr )*
BitwiseXorExpr     <- BitwiseAndExpr ( "^" Spacing BitwiseAndExpr )*
BitwiseAndExpr     <- EqualityExpr ( "&" Spacing EqualityExpr )*
EqualityExpr       <- RelationalExpr ( ("==" / "!=") Spacing RelationalExpr )*
RelationalExpr     <- ShiftExpr ( ("<=" / ">=" / "<" / ">") Spacing ShiftExpr )*
ShiftExpr          <- AdditiveExpr ( ("<<" / ">>") Spacing AdditiveExpr )*
AdditiveExpr       <- MultiplicativeExpr ( ("+" / "-") Spacing MultiplicativeExpr )*
MultiplicativeExpr <- UnaryExpr ( ("*" / "/" / "%") Spacing UnaryExpr )*

UnaryExpr          <- ( "!" / "~" / "@" / "-" ) Spacing UnaryExpr / PostfixExpr
PostfixExpr        <- PrimaryExpr ( 
                        "." Spacing Identifier 
                      / "::" Spacing Identifier 
                      / "(" Spacing Args? ")" Spacing 
                      / "[" Spacing Expression "]" Spacing 
                      / "<" Spacing TypeArgs? ">" Spacing 
                      / "?" Spacing 
                      )*
TypeArgs           <- Type ( "," Spacing Type )*

Args               <- Expression ( "," Spacing Expression )*

PrimaryExpr        <- IntegerLiteral 
                    / StringLiteral
                    / CharLiteral
                    / Identifier
                    / '(' Spacing Expression ')' Spacing
```

## Attributes

Attributes in QW provide metadata that influences compilation, code generation, or semantic behavior. They are declared within double square brackets (`[[ ... ]]`) and can optionally take a string-like identifier as a value (`name: value`).

The following attributes are strictly validated during the semantic analysis phase:

- **`symbol`**: Overrides the ABI mangling of the declaration.
  - `[[ symbol: bare ]]`: Disables mangling. The exact identifier name is exported (useful for FFI and C interoperability).
  - `[[ symbol: qw ]]`: Forces standard QW mangling.
- **`weak`**: Emits a weak symbol linkage for the given function. Cannot take a value.
  - `[[ weak ]]`: Marks the function as weak.
- **`calling`**: Specifies the calling convention for the function.
  - `[[ calling: fast ]]`: Uses LLVM's `Fast` calling convention (default in optimized contexts).
  - `[[ calling: cdecl ]]`: Uses the standard C calling convention (`cdecl`).
  - `[[ calling: cold ]]`: Uses LLVM's `Cold` calling convention (for rarely executed paths).
