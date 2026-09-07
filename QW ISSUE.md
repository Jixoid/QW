
**1. `Patt` ile `Thing::MatchArm` Arasındaki Uyuşmazlık**

- `Patt` enumu tanımlanmış (`One`, `Under`, `Rest`, `Tuple`, `Array`).
    
- Ancak `Thing::MatchArm` şu şekilde tanımlı:
    
    Rust
    
    ```
    MatchArm(ExprId /* pat */, ExprId /* body */)
    ```
    
    Desen pozisyonunda `ExprId` duruyor. `match` gövdesinde desen eşleme yapılacaksa buranın `(PattId, ExprId)` (veya opsiyonel bir `guard: Option<ExprId>`) olması gerekir. Eğer desenler ifade olarak parse ediliyorsa `Patt` yapısı atıl kalır.
    

**2. `Patt` (Desen Eşleme) Çok Kısıtlı** Mevcut `Patt` yapısı enum/struct dillerinin ihtiyaç duyduğu temel kalıpları karşılayamaz:

- **Literal Desenler:** `match x { 0 => ..., "ok" => ... }` için `Lit(ExprId)` veya `Number(Span)` eksik.
    
- **Enum / Variant Eşleme:** `Option::Some(val)` veya `ImageType::e1` gibi desenleri karşılayacak bir `Path(Rng, Option<PattId>)` varyantı yok.
    
- **Struct Yıkımı (_Destructuring_):** `Vec2 { x, y }` benzeri alan eşleme deseni yok.
    

**3. Tiplerin `ItemKind` Yerine `TypeKind` İçinde Tanımlanması** `Struct`, `Enum`, `Flags`, `Iface` tanımları `ItemKind` yerine `TypeKind` içine konmuş; `ItemKind` ise bunları `ItemTy(TypeId)` ile sarmalıyor.

- Bu yaklaşım tipleri dilde _first-class_ yapar (güzel bir tercih).
    
- Ancak `TypeKind::Struct(Rng /* ItemId */)` ifadesi, `Type`'ın `Item`'a, `Item`'ın da `Type`'a işaret ettiği döngüsel bir referans kurar. Arenalarda indeks kullandığınız için bellek sızıntısı olmaz; fakat semantik analizde bir struct'ın boyutunu hesaplarken isim çözümleme döngülerine (_cyclic dependencies_) dikkat etmelisiniz.
    

**4. `Die` İfadesinin Tipi** `ExprKind::Die { lvar: Span }`:

- `panic!` veya hata fırlatma için düşünülmüş görünüyor.
    
- Ancak `lvar` neden bir `Span`? Kullanıcı `die "bağlantı koptu"` veya `die error_code` gibi bir **ifade** ile programı sonlandırmak istediğinde buraya `Option<ExprId>` verilmesi daha geniş bir kullanım sağlar.
    

**5. `ExprKind` Bellek Boyutu (Enum Padding)** Rust'ta bir enum'ın boyutu en büyük varyantına göre belirlenir:

- `Block { label: Option<Span>, rng: Rng, expr: Option<ExprId> }` ve `ForIn { vars, iter, blok, elsb }` varyantları bellekte **~24-28 bayt** yer tutar.
    
- `Bool(Span)` veya `Unit()` gibi minik varyantlar da bu 28 baytlık alanı işgal eder.
    
- `Expr` struct'ı `pos: Span` (8 bayt) + `ExprKind` (32 bayt) ile toplamda **~40 bayt** civarındadır. 40 bayt AST için makul bir sınırdır; ancak varyantlara daha büyük alanlar (örneğin 3'ten fazla ID içeren struct'lar) eklememeye dikkat edin, aksi halde tüm arena şişer.