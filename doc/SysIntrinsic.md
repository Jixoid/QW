# QW Sys Intrinsic Functions

Sistemdeki `sys::*` fonksiyonları (intrinsic/built-in) kullanım amaçlarına göre aşağıda kategorize edilmiştir.

## 1. Boyut Bilgisi (Size Information)

Derleme zamanında tipin boyutunu döndürmek için kullanılır.

* **`sys::is_size<T>() -> usize`**
  Verilen `T` tipinin bellekte kapladığı bayt cinsinden boyutu döndürür. (Not: Şu an prototip aşamasında olup varsayılan olarak 8 döndürmektedir).

## 2. Tip Karşılaştırma ve Dönüşüm (Type Comparison & Conversion)

İki tipin birbirine göre durumlarını derleme zamanında sınamak için kullanılır.

* **`sys::is_same<T, U>() -> bool`**
  Verilen iki tipin derleyici seviyesinde tamamen aynı olup olmadığını kontrol eder. Tiplerin isimleri üzerinden birebir eşleşme arar.
* **`sys::is_convertible<From, To>() -> bool`**
  `From` tipindeki bir ifadenin örtük (implicit) olarak `To` tipine dönüştürülüp dönüştürülemeyeceğini test eder. Derleyicinin tip atama ve dönüşüm kurallarını (sema conversion) kullanarak sınama yapar.
* **`sys::cast<To>(value) -> To`**
  Bir değeri `To` tipine dönüştürür. Şu an için `enum <-> int` ve `set <-> int` dönüşümlerini destekler. Enum'dan sayıya veya Set'ten sayıya yapılan dönüşümlerde, değerin hedeflenen tam sayı tipine sığıp sığmadığı; aynı şekilde tam sayıdan Set veya Enum'a yapılan dönüşümlerde o değere karşılık gelen elemanın varlığı (sınır denetimi / bounds checking) çok sıkı bir şekilde derleme zamanında denetlenir ve taşma/sınır aşımı durumunda derleme hatası üretilir.

## 3. Temel Tip Kontrolleri (Primitive Type Checking)

Bir tipin derleyicinin sunduğu temel (primitive) veri tiplerinden hangisine girdiğini kontrol eder.

* **`sys::is_int<T>() -> bool`**
  Tipin `I8` ile `U128` arasındaki temel tam sayı tiplerinden biri olup olmadığını kontrol eder.
* **`sys::is_float<T>() -> bool`**
  Tipin kayan noktalı (float, double vs.) bir sayı tipi olup olmadığını kontrol eder.
* **`sys::is_bool<T>() -> bool`**
  Tipin tam olarak boolean tipinde olup olmadığını test eder.
* **`sys::is_char<T>() -> bool`**
  Tipin karakter (`char`) tipinde olup olmadığını kontrol eder.
* **`sys::is_void<T>() -> bool`**
  Tipin `void` tipinde olup olmadığını kontrol eder.
* **`sys::is_ptr<T>() -> bool`**
  Tipin temel `ptr` (ham adres) tipinde olup olmadığını test eder.
* **`sys::is_signed<T>() -> bool`**
  Tipin işaretli bir tip (signed integer veya işaretli diğer türler) olup olmadığını kontrol eder.
* **`sys::is_unsigned<T>() -> bool`**
  Tipin işaretsiz bir tip (unsigned integer veya işaretsiz diğer türler) olup olmadığını test eder.

## 4. Karmaşık Tip Kontrolleri (Compound Type Checking)

Daha üst seviye oluşturulmuş karmaşık yapıları tanımlamak için kullanılır.

* **`sys::is_pointer<T>() -> bool`**
  Tipin türetilmiş bir işaretçi (pointer type) türü olup olmadığını kontrol eder.
* **`sys::is_reference<T>() -> bool`**
  Tipin bir referans türü (reference type) olup olmadığını kontrol eder.
* **`sys::is_array<T>() -> bool`**
  Tipin boyutlu/sabit (ZArrayType) ya da dinamik (PArrayType) bir dizi olup olmadığını sınar.
* **`sys::is_struct<T>() -> bool`**
  Tipin bir kayıt / yapı (record/struct type) olup olmadığını kontrol eder.
* **`sys::is_function<T>() -> bool`**
  Tipin bir fonksiyon tipi (func type) olup olmadığını kontrol eder.
* **`sys::is_enum<T>() -> bool`**
  Tipin bir enum türü (enum type) olup olmadığını kontrol eder.
* **`sys::is_set<T>() -> bool`**
  Tipin bir set türü (set bitmask type) olup olmadığını kontrol eder.
