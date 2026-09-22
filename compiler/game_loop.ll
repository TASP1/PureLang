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

@.str.0 = private unnamed_addr constant [7 x i8] c"Player\00", align 1
@.str.1 = private unnamed_addr constant [10 x i8] c"Welcome, \00", align 1
@.str.2 = private unnamed_addr constant [9 x i8] c"Health: \00", align 1
@.str.3 = private unnamed_addr constant [10 x i8] c"Game Over\00", align 1
@.str.4 = private unnamed_addr constant [14 x i8] c"You survived!\00", align 1

define i32 @main() {
entry:
  %health.addr = alloca i64, align 8
  store i64 100, ptr %health.addr, align 8
  %t0 = getelementptr inbounds [7 x i8], ptr @.str.0, i64 0, i64 0
  %name.addr = alloca ptr, align 8
  store ptr %t0, ptr %name.addr, align 8
  %t1 = load ptr, ptr %name.addr, align 8
  %t2 = getelementptr inbounds [10 x i8], ptr @.str.1, i64 0, i64 0
  %t3 = load ptr, ptr %name.addr, align 8
  %t4 = call i64 @strlen(ptr %t2)
  %t5 = call i64 @strlen(ptr %t3)
  %t6 = add i64 %t4, %t5
  %t7 = add i64 %t6, 1
  %t8 = call ptr @malloc(i64 %t7)
  call ptr @strcpy(ptr %t8, ptr %t2)
  call ptr @strcat(ptr %t8, ptr %t3)
  %t9 = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0
  call i32 (ptr, ...) @printf(ptr %t9, ptr %t8)
  %i.addr = alloca i64, align 8
  store i64 1, ptr %i.addr, align 8
  br label %loop.cond.0
loop.cond.0:
  %t10 = load i64, ptr %i.addr, align 8
  %t11 = icmp slt i64 %t10, 5
  br i1 %t11, label %loop.body.1, label %loop.end.2
loop.body.1:
  %t12 = load i64, ptr %health.addr, align 8
  %t13 = sub i64 %t12, 10
  store i64 %t13, ptr %health.addr, align 8
  %t14 = load i64, ptr %health.addr, align 8
  %t15 = getelementptr inbounds [9 x i8], ptr @.str.2, i64 0, i64 0
  %t16 = getelementptr inbounds [7 x i8], ptr @.fmt_concat_sn, i64 0, i64 0
  %t17 = call ptr @malloc(i64 256)
  call i32 (ptr, i64, ptr, ...) @snprintf(ptr %t17, i64 256, ptr %t16, ptr %t15, i64 %t14)
  %t18 = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0
  call i32 (ptr, ...) @printf(ptr %t18, ptr %t17)
  call void @free(ptr %t17)
  %t19 = load i64, ptr %health.addr, align 8
  %t20 = icmp sle i64 %t19, 0
  br i1 %t20, label %then.3, label %endif.5
then.3:
  %t21 = getelementptr inbounds [10 x i8], ptr @.str.3, i64 0, i64 0
  %t22 = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0
  call i32 (ptr, ...) @printf(ptr %t22, ptr %t21)
  ret i32 0
after.ret.6:
  br label %endif.5
endif.5:
  %t23 = load i64, ptr %i.addr, align 8
  %t24 = add i64 %t23, 1
  store i64 %t24, ptr %i.addr, align 8
  br label %loop.cond.0
loop.end.2:
  %t25 = getelementptr inbounds [14 x i8], ptr @.str.4, i64 0, i64 0
  %t26 = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0
  call i32 (ptr, ...) @printf(ptr %t26, ptr %t25)
  ret i32 0
}

