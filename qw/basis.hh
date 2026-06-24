/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <initializer_list>
#include <limits>
#include <memory>
#include <mutex>
#include <pthread.h>
#include <shared_mutex>
#include <stdexcept>
#include <type_traits>
#include <vector>



/// Modern C++
#define fun auto


/// Int
using i8  = std::int8_t;
using i16 = std::int16_t;
using i32 = std::int32_t;
using i64 = std::int64_t;

using u8  = std::uint8_t;
using u16 = std::uint16_t;
using u32 = std::uint32_t;
using u64 = std::uint64_t;

#ifdef __SIZEOF_INT128__
  using i128 = signed __int128;
  using u128 = unsigned __int128;
#endif

using i0 = std::ptrdiff_t;
using u0 = std::size_t;

using isize = i0; // like rust
using usize = u0; // like rust

using handle = u0;
using ohid = u0;


/// Float
using f32 = float;
using f64 = double;

using bf16 = __bf16;
using f16  = _Float16;
using f128 = _Float128;

#if INTPTR_MAX == INT64_MAX
  using f0 = f64;
#else
  using f0 = f32;
#endif

using fsize = f0; // like rust



/// Norm Types

/** 
 * @brief Normalized integer type structure.
 * @tparam T The underlying integral type.
 */
template <typename T>
  requires std::is_integral_v<T>
struct __norm
{
  public:
    constexpr inline __norm() {}
    constexpr inline __norm(f32 _): m_value(f_to_float(_)) {}


  private:
    T m_value{};

    constexpr static inline fun f_to_float(f32 value) -> T {
      constexpr f32 max_val = std::numeric_limits<T>::max();

      if constexpr (std::is_unsigned_v<T>) {
        auto clamped = std::clamp<f32>(value, 0.0, +1.0);

        return T(clamped * max_val + 0.5);
      }
      else {
        auto clamped = std::clamp<f32>(value, -1.0, +1.0);

        return T(clamped * max_val + (clamped >= 0 ? 0.5 : -0.5));
      }
    }


  public:
    constexpr inline fun& value() { return m_value; }

    constexpr inline operator f32() const {
      constexpr f32 max_val = std::numeric_limits<T>::max();
      
      if constexpr (std::is_unsigned_v<T>)
        return f32(m_value) / max_val;
      else
        return std::max(-1.0f, f32(m_value) / max_val);
    }

};


template <typename T>
struct is_norm_type: std::false_type {};

template <typename T>
struct is_norm_type<__norm<T>>: std::true_type {};

template <typename _Tp>
inline constexpr bool is_norm_v = is_norm_type<_Tp>::value;


using nu8  = __norm<u8>;
using nu16 = __norm<u16>;
using nu32 = __norm<u32>;

using ni8  = __norm<i8>;
using ni16 = __norm<i16>;
using ni32 = __norm<i32>;


/// Vector

template <typename T, u0 S>
struct vec_traits
{
  typedef T type __attribute__((vector_size(sizeof(T) * S)));
};


/**
 * @brief A generic SIMD vector structure.
 * @tparam T The element type.
 * @tparam S The number of elements in the vector.
 */
template <typename T, u0 S>
  requires std::is_arithmetic_v<T>
struct alignas(sizeof(T) *S) vec
{
  private:
    using vector_t = typename vec_traits<T, S>::type;
    vector_t _elems;

  public:
    vec() : _elems{} {}
    vec(vector_t _) : _elems(_) {}

    vec(T val) {
      for (u0 i{}; i < S; i++) _elems[i] = val;
    }
    
    vec(std::array<T, S> V) {
      for (u0 i{}; i < S; i++) _elems[i] = V[i];
    }

    vec(std::initializer_list<T> V) : _elems{} {
      if (V.size() > S) throw std::out_of_range("Liste boyutu vektör sınırını aştı!");
      u0 i{};
      for (auto val: V) _elems[i++] = val;
    }

    vec(const T* V) {
      for (u0 i{}; i < S; i++) _elems[i] = V[i];
    }

  public:
    [[nodiscard]] inline bool is_equal(const vec<T, S> &It) const {
      auto Mask = (_elems == It._elems);
      for (u0 i{}; i < S; i++) {
        if (!Mask[i]) return false;
      }
      return true;
    }
    
    [[nodiscard]] inline std::array<T, S> to_array() const {
      std::array<T, S> Arr;
      for (u0 i{}; i < S; i++) Arr[i] = _elems[i];
      return Arr;
    }

    [[nodiscard]] inline std::vector<T> to_vector() const {
      std::vector<T> Vec(S);
      for (u0 i{}; i < S; i++) Vec[i] = _elems[i];
      return Vec;
    }

    inline u0 size() const { return S; }
    
    inline void set(u0 index, T val) {
      if (index >= S) throw std::out_of_range("Index out of bounds");
      _elems[index] = val;
    }

    [[nodiscard]] inline T get(u0 index) const {
      if (index >= S) throw std::out_of_range("Index out of bounds");
      return _elems[index];
    }

    inline vec min(const vec &It) const {
      vec result;
      for (u0 i = 0; i < S; i++) result._elems[i] = (_elems[i] < It._elems[i]) ? _elems[i] : It._elems[i];
      return result;
    }

    inline vec max(const vec &It) const {
      vec result;
      for (u0 i = 0; i < S; i++) result._elems[i] = (_elems[i] > It._elems[i]) ? _elems[i] : It._elems[i];
      return result;
    }

public:  
    inline fun operator[](u0 __n) const -> T { return _elems[__n]; }
    
    inline fun operator== (const vec &It) const -> bool { return is_equal(It); }

    inline fun operator+ (const vec &It) const -> vec { return (_elems + It._elems); }
    inline fun operator- (const vec &It) const -> vec { return (_elems - It._elems); }
    inline fun operator* (const vec &It) const -> vec { return (_elems * It._elems); }
    inline fun operator/ (const vec &It) const -> vec { return (_elems / It._elems); }
    
    inline fun operator% (const vec &It) const -> vec requires std::is_integral_v<T> { 
      return (_elems % It._elems); 
    }

    inline fun operator+= (const vec &It) -> vec& { _elems += It._elems; return *this; }
    inline fun operator-= (const vec &It) -> vec& { _elems -= It._elems; return *this; }
    inline fun operator*= (const vec &It) -> vec& { _elems *= It._elems; return *this; }
    inline fun operator/= (const vec &It) -> vec& { _elems /= It._elems; return *this; }
    
    inline fun operator%= (const vec &It) -> vec& requires std::is_integral_v<T> { 
      _elems %= It._elems; return *this; 
    }
};


/// Data
struct data
{
  public:
    inline data() {}

    inline data(void* ptr, u0 size)
      : m_ptr(ptr)
      , m_size(size)
    {}


  private:
    void* m_ptr{};
    u0    m_size{};

  public:
    inline fun& ptr() { return m_ptr; }
    inline fun& size() { return m_size; }
};


/// Offs
struct offs
{
  public:
    inline offs() {}

    inline offs(u0 off, u0 size)
      : m_off(off)
      , m_size(size)
    {}


  private:
    u0 m_off{};
    u0 m_size{};

  public:
    inline fun& off() { return m_off; }
    inline fun& size() { return m_size; }
};




/// Flags
template <typename BitType>
struct flags
{
  public:
    using BitsType = BitType;
    using MaskType = typename std::underlying_type<BitType>::type;

  public:
    constexpr flags() noexcept : m_mask(0) {}
    
    constexpr flags(const flags &rhs) noexcept : m_mask(rhs.m_mask) {};
    
    constexpr flags(BitType bit) noexcept : m_mask(static_cast<MaskType>(bit)) {}

  private:
    constexpr explicit flags(MaskType flags) noexcept : m_mask(flags) {}


  private:
    MaskType m_mask;

  public:
    inline fun mask() { return m_mask; }


  public:
    //fun operator<=>(const flags<BitType>&) const = default;
    
    constexpr fun operator ==(const flags<BitType> &rhs) const noexcept -> bool { return m_mask == rhs.m_mask; }
    constexpr fun operator !=(const flags<BitType> &rhs) const noexcept -> bool { return m_mask != rhs.m_mask; }
  
    constexpr fun operator <(const flags<BitType> &rhs) const noexcept -> bool { return m_mask < rhs.m_mask; }
    constexpr fun operator >(const flags<BitType> &rhs) const noexcept -> bool { return m_mask > rhs.m_mask; }

    constexpr fun operator <=(const flags<BitType> &rhs) const noexcept -> bool { return m_mask <= rhs.m_mask; }
    constexpr fun operator >=(const flags<BitType> &rhs) const noexcept -> bool { return m_mask >= rhs.m_mask; }


  public:
    constexpr fun operator !() const noexcept -> bool { return !m_mask; }

    constexpr fun operator ~() const noexcept -> flags<BitType> { return ~m_mask; }

    constexpr fun operator &(const flags<BitType> &rhs) const noexcept -> flags<BitType> { return flags<BitType>( m_mask & rhs.m_mask ); }
    constexpr fun operator |(const flags<BitType> &rhs) const noexcept -> flags<BitType> { return flags<BitType>( m_mask | rhs.m_mask ); }
    constexpr fun operator ^(const flags<BitType> &rhs) const noexcept -> flags<BitType> { return flags<BitType>( m_mask ^ rhs.m_mask ); }


  public:
    constexpr fun operator =(const flags<BitType> &rhs) noexcept -> flags<BitType>& = default;

    constexpr fun operator &=(const flags<BitType> &rhs) noexcept -> flags<BitType>& { m_mask &= rhs.m_mask; return *this; }
    constexpr fun operator |=(const flags<BitType> &rhs) noexcept -> flags<BitType>& { m_mask |= rhs.m_mask; return *this; }
    constexpr fun operator ^=(const flags<BitType> &rhs) noexcept -> flags<BitType>& { m_mask ^= rhs.m_mask; return *this; }


  public:
    explicit constexpr operator bool() const noexcept { return !!m_mask; }
    explicit constexpr operator MaskType() const noexcept { return m_mask; }
};




/// Mutex
template <typename T>
using slock = std::shared_lock<T>;

template <typename T>
using ulock = std::unique_lock<T>;




/// Pointer
using nil_t = decltype(nullptr);
inline constexpr nil_t nil = nullptr;


template <typename T>
using uptr = std::unique_ptr<T>;

template <typename T>
inline fun make_uptr(T* obj) -> uptr<T> { return uptr<T>(obj); }

template<typename T, typename... args>
inline fun make_uptr(args&&... __args) -> uptr<T> { return uptr<T>(new T(std::forward<args>(__args)...)); }



template <typename T>
#ifdef __GLIBCXX__
using sptr = std::__shared_ptr<T, std::_Lock_policy::_S_single>;
#else
using sptr = std::shared_ptr<T>;
#endif

template <typename T>
inline fun make_sptr(T* obj) -> sptr<T> { return sptr<T>(obj); }

template<typename T, typename... args>
inline fun make_sptr(args&&... __args) -> sptr<T> { return sptr<T>(new T(std::forward<args>(__args)...)); }



template <typename T>
#ifdef __GLIBCXX__
using wptr = std::__weak_ptr<T, std::_Lock_policy::_S_single>;
#else
using wptr = std::weak_ptr<T>;
#endif


template <typename T>
#ifdef __GLIBCXX__
using wptr_from = std::__enable_shared_from_this<T, std::_Lock_policy::_S_single>;
#else
using wptr_from = std::enable_shared_from_this<T>;
#endif

