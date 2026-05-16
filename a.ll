; ModuleID = 'Test.qw'
source_filename = "Test.qw"

declare i32 @_Q3add(i32, i32)

define i32 @main() {
  %1 = alloca i64, align 8
  store i64 0, ptr %1, align 4
  ret i32 0
}
