# QW Programlama Dili

QW, Rust ile geliştirilmiş minimalist, modern bir sistem programlama dili ve modüler derleyici altyapısıdır. QAOS ekosisteminin bir parçası olarak, verimli ve düşük seviyeli makine kodu üretimi ve JIT için LLVM kullanan, arena bellek yönetimli temiz bir mimariye odaklanmaktadır.

Diğer diller: [en](README.md) | [Dökümantasyon Dizini](doc/README.md)

---

## Öne Çıkan Özellikler

- **Minimalist ve Açık Sözdizimi:** Önek işaretçiler (`^T`), güvenli referanslar (`&T`), opsiyonel (`?T`) ve hata (`!T`) tipleri içeren temiz PEG tabanlı sözdizimi.
- **İfade Odaklı Model:** Blok sonu ifadeleri, örüntü eşleme (`match`) ve ifade tabanlı kontrol akışı (`if/ef/else`, `loop/else`, `while/else`, `for/in/else`).
- **Polimorfizm ve Fat Pointer'lar:** Tekil veri kalıtımlı değer yapıları (struct), negatif ofsetli Sanal Metot Tabloları (VMT) üzerinden sıfır maliyetli yukarı çevrim (upcasting) ve dinamik arayüz (iface) çağrıları.
- **SIMD ve Ölçeklenebilir Vektörler:** Sabit boyutlu SIMD vektörleri (`[T * N]`) ve mimariden bağımsız ölçeklenebilir vektörler (`[T *]`) için birinci sınıf dil desteği.
- **Yüksek Performanslı Çok Aşamalı Derleyici:** Bitişik bellek arenaları, hafif 32-bit düğüm kimlikleri (ID) ve merkezi string dize havuzu (interner) üzerine inşa edilmiştir.
- **Entegre Araçlar ve JIT:** LLVM bitcode üretimi, hızlı kontrol (`check`), aşama profillemesi (`--timings`, `--usages`) ve bellek içi JIT çalıştırma (`qw run`) desteğine sahip `qw` komut satırı arayüzü.

---

## Derleyici Mimarisi

Derleme hattı dört bağımsız ve ayrık aşamadan oluşur:

```
Kaynak (.qw) ──> Aşama 1: Ayrıştırma ve Kapsam ──> Aşama 2: HIR ve Tipler ──> Aşama 3: MIR ve Bellek ──> Aşama 4: LLVM ve JIT
```

1. **Aşama 1 (Önyüz):** Bayt seviyesinde hızlı sözcüksel analiz (`qwc_lexer`), Pratt sözdizim analizi (`qwc_parse`), AST arenası (`qwc_ast`) ve sözcüksel kapsam/içe aktarma çözümlemesi (`qwc_resolve`).
2. **Aşama 2 (Anlamsal Analiz):** Üst seviye ara temsil üretimi (`qwc_hir_gen`), tip çıkarımı ve katı imza denetimi, yerleşik çekirdek tipler (`qwc_intrinsic`), sembol dışa aktarım haritası (`ExportMap`) ve QW Birim serileştirmesi (`.qwu`).
3. **Aşama 3 (Orta Seviye IR ve Yerleşim):** Kontrol Akış Çizgesi (CFG) inşası (`qwc_mir`), bellek boyutu ve hizalama hesaplamaları, VMT tabloları üretimi ve Fat Pointer çözümlemesi.
4. **Aşama 4 (Arka Uç ve Yürütme):** Inkwell ile LLVM IR üretimi (`qwc_cgen_llvm`), bitcode çıktısı (`build/out.bc`), metin formatında LLVM IR (`build/out.ll`) ve bellek içi JIT yürütme.

Ayrıntılı bilgi için [Derleyici Mimarisi (Architecture.md)](doc/Architecture.md) belgesine göz atabilirsiniz.

---

## Hızlı Başlangıç

### Derleyiciyi Derleyin

```bash
cargo build --release
```

### Bir QW Projesi Başlatın ve Çalıştırın

```bash
# Yeni bir paket oluşturun
cargo run -- init my_project
cd my_project

# JIT ile hemen çalıştırın
cargo run -- run

# LLVM bitcode derlemesi yapın
cargo run -- build
```

---

## Dökümantasyon

Tüm teknik belgelere [`/doc`](doc/README.md) dizininden erişebilirsiniz:

- [**Dökümantasyon Dizini (`doc/README.md`)**](doc/README.md) - Belgelerin genel özeti ve rehber.
- [**Sözdizimi ve Dilbilgisi (`doc/Syntax.md`)**](doc/Syntax.md) - Resmi PEG dilbilgisi kuralları, anahtar kelimeler, operatörler ve ifadeler.
- [**Derleyici Mimarisi (`doc/Architecture.md`)**](doc/Architecture.md) - Derleme aşamaları, crate haritası ve bellek modeli.
- [**Tip Sistemi Tanımı (`doc/TypeSystem.md`)**](doc/TypeSystem.md) - İlkel tipler, yapılar, arayüzler, trait'ler, variant'lar, işaretçiler ve vektörler.
- [**VMT ve Fat Pointer ABI (`doc/VMT.md`)**](doc/VMT.md) - Sanal Metot Tabloları, negatif ofset başlıkları ve dinamik arayüz yönetimi.
- [**İsim Karıştırma (Name Mangling) (`doc/Mangling.md`)**](doc/Mangling.md) - Bağlayıcı uyumluluğu, mevcut derleyici implementasyonu ve EBNF ABI belirtimi.
- [**CLI ve Yapılandırma (`doc/CLI_and_Config.md`)**](doc/CLI_and_Config.md) - `qw` komut satırı kullanımı, derleyici bayrakları ve `qw.conf` formatı.

---

## Lisans

Bu proje GNU Genel Kamu Lisansı versiyon 3 (GPL3) kapsamında lisanslanmıştır.

Copyright (c) 2025-2026 Kadir Aydın.

---

## QAOS

Açık kaynak topluluğu tarafından ❤️ ile geliştirildi.
