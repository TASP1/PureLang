; PureLang → LLVM IR
target triple = "x86_64-unknown-linux-gnu"

declare i32 @printf(ptr, ...)
declare ptr @malloc(i64)
declare ptr @strcpy(ptr, ptr)
declare ptr @strcat(ptr, ptr)
declare i64 @strlen(ptr)
declare i32 @snprintf(ptr, i64, ptr, ...)
declare void @free(ptr)

@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@.fmt_concat_sn = private unnamed_addr constant [7 x i8] c"%s%lld\00", align 1

@.str.0 = private unnamed_addr constant [17 x i8] c"Hello, PureLang!\00", align 1
@.str.1 = private unnamed_addr constant [49 x i8] c"Native speed. Absolute safety. Beautiful syntax.\00", align 1

define i32 @main() {
entry:
  %t0 = getelementptr inbounds [17 x i8], ptr @.str.0, i64 0, i64 0
  %t1 = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0
  call i32 (ptr, ...) @printf(ptr %t1, ptr %t0)
  %t2 = getelementptr inbounds [49 x i8], ptr @.str.1, i64 0, i64 0
  %t3 = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0
  call i32 (ptr, ...) @printf(ptr %t3, ptr %t2)
  ret i32 0
}

