/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/



#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>


#ifdef __cplusplus
extern "C" {
#endif


// Int
typedef  int8_t   i8;
typedef  int16_t  i16;
typedef  int32_t  i32;
typedef  int64_t  i64;
typedef  signed __int128 i128;



// UInt
typedef  uint8_t   u8;
typedef  uint16_t  u16;
typedef  uint32_t  u32;
typedef  uint64_t  u64;
typedef  unsigned __int128 u128;  


// Char
typedef  char  c8;


// System
#if INTPTR_MAX == INT64_MAX
  typedef u64 u0;
  typedef i64 i0;
#else
  typedef u32 u0;
  typedef i32 i0;
#endif

typedef  u0  handle;
typedef  u0  ohid;


// Float
typedef  __bf16  bf16;
typedef  __fp16  f16;
typedef  float   f32;
typedef  double  f64;
typedef  __float128  f128;

#if INTPTR_MAX == INT64_MAX
  typedef f64 f0;
#else
  typedef f32 f0;
#endif



// Point
typedef  void*  point;
#define Nil  nullptr

typedef struct
{
  point Point;
  u0    Size;
} data_;

typedef struct
{
  point Point;
  u64   Size;
} data_64;

typedef struct
{
  point Point;
  u32   Size;
} data_32;


#ifdef __cplusplus
}
#endif
