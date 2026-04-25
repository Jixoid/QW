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
#include <concepts>
#include <cstdlib>
#include <cstring>
#include <initializer_list>
#include <memory>
#include <pthread.h>
#include <stdexcept>
#include <vector>

#include "Basis.h"


// Modern C++
#define fun auto


// Vector
template <typename T, u0 S>
  requires std::is_arithmetic_v<T>
struct alignas(sizeof(T) *S) vec
{
private:
  using vector_t = T __attribute__((vector_size(sizeof(T)*S)));
  vector_t _elems;


public:
  vec(): _elems{0} {}
  vec(vector_t V) : _elems(V) {}

  vec(T val)
  {
    std::vector<T> Data(S);

    for (auto &X: Data) X = val;

    __builtin_memcpy(&_elems, Data.data(), sizeof(vector_t));
  }
  
  vec(std::array<T,S> V)
  {
    __builtin_memcpy(&_elems, V.data(), sizeof(vector_t));
  }

  vec(std::initializer_list<T> V)
  {
    if (V.size() > S)
      throw std::out_of_range("List size out of bounds");

    
    std::copy(V.begin(), V.end(), (T*)&_elems);
  }

  vec(T *V)
  {
    __builtin_memcpy(&_elems, V, sizeof(vector_t));
  }


public:
  [[nodiscard]] inline bool is_equal(const vec<T,S> &It) const
  {
    auto Mask = (_elems == It._elems);
    for (u0 i = 0; i < S; ++i)
      if (!Mask[i])
        return false;
    
    return true;
  }
  
  [[nodiscard]] inline std::array<T,S> to_array() const
  {
    std::array<T,S> Arr;
    
    __builtin_memcpy(Arr.data(), &_elems, sizeof(vector_t));
    return Arr;
  }

  [[nodiscard]] inline std::vector<T> to_vector() const
  {
    std::vector<T> Vec(S);
    
    __builtin_memcpy(Vec.data(), &_elems, sizeof(vector_t));
    return Vec;
  }

  inline u0 size() const { return S; }
  
  inline void set(u0 index, T val)
  {
    if (index >= S) throw std::out_of_range("Index out of bounds");
      _elems[index] = val;
  }

  [[nodiscard]] inline T get(size_t index)
  {
    if (index >= S) throw std::out_of_range("Index out of bounds");
    return _elems[index];
  }


  inline vec min(const vec &It) const { return (_elems < It._elems) ? _elems : It._elems; }
  inline vec max(const vec &It) const { return (_elems > It._elems) ? _elems : It._elems; }


public:  
  inline T operator[](u0 __n) const { return _elems[__n]; }
  
  inline vec operator== (const vec &It) const { return (_elems == It._elems); }

  inline vec operator+ (const vec &It) const { return (_elems + It._elems); }
  inline vec operator- (const vec &It) const { return (_elems - It._elems); }
  inline vec operator* (const vec &It) const { return (_elems * It._elems); }
  inline vec operator/ (const vec &It) const { return (_elems / It._elems); }
  inline vec operator% (const vec &It) const { return (_elems % It._elems); }

  inline vec& operator+=  (const vec &It) { _elems += It._elems; return *this; }
  inline vec& operator-=  (const vec &It) { _elems -= It._elems; return *this; }
  inline vec& operator*=  (const vec &It) { _elems *= It._elems; return *this; }
  inline vec& operator/=  (const vec &It) { _elems /= It._elems; return *this; }
  inline vec& operator%=  (const vec &It) { _elems %= It._elems; return *this; }

};


// Data
template <std::unsigned_integral T>
struct __data
{
  point Point;
  T     Size;
};


// Pointer
template <typename T>
using uptr = std::unique_ptr<T>;

template <typename T>
fun make_uptr(T* obj) -> uptr<T> { return uptr<T>(obj); }



template <typename T>
#ifdef __GLIBCXX__
using sptr = std::__shared_ptr<T, std::_Lock_policy::_S_single>;
#else
using sptr = std::shared_ptr<T>;
#endif

template <typename T>
fun make_sptr(T* obj) -> sptr<T> { return sptr<T>(obj); }


template <typename T>
#ifdef __GLIBCXX__
using wptr = std::__weak_ptr<T, std::_Lock_policy::_S_single>;
#else
using wptr = std::weak_ptr<T>;
#endif
