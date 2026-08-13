
## Type
```
"&" X => Reference
"^" X => Pointer
"?" X => Optional
".." X => Range

"[" X ("," NUM)* "]" => Array
"[" X "x" NUM "]" => Simd
"(" X ("," X)* ")" => Tuple
"<" X ("," X)* ">" => Variant
"{" X ("," X)* "}" => Struct

```


## AST
**Visitor:**
+ Scope Based Name Hashing `in scope things: HashMap<String, AstId>`

**Lowering:**
+ Self Parameter Injection `function of object, &self`
+ Path/Nick Resulation `find(name: &str) -> AstId`
	+ pass 1: Traits
		+ event: Register the trait,type Binary
	+ pass 2: *


## HIR
**Visitor:**
+ Type Checker `type_check(*)`
+ Check Generic Requires `requires T: std::ops::Add + std::ops::Sub`

**Lowering:**
+ Generic Specialization / Monomorphization `generic_specialization()`
+ Size Calculation `calc_size(lay: &LayoutEngine) -> usize`
	+ Niche Layout Calculation
+ Static Trait Relocation `trait Add.add(rhs: &X)`


## MIR
**Lowering:**
+ Create VTables `gen_vtable() -> [u8]`

