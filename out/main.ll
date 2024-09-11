; ModuleID = 'main.8db5165aed74ea44-cgu.0'
source_filename = "main.8db5165aed74ea44-cgu.0"
target datalayout = "e-P1-p:16:8-i8:8-i16:8-i32:8-i64:8-f32:8-f64:8-n8-a:8"
target triple = "avr-unknown-unknown"

; Function Attrs: nounwind
define dso_local noundef i8 @main() unnamed_addr addrspace(1) #0 {
start:
  tail call addrspace(1) void @delay(i16 noundef 10000) #3
  ret i8 42
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local noundef i8 @add(i8 noundef %left, i8 noundef %right) unnamed_addr addrspace(1) #1 {
start:
  %_0 = add i8 %right, %left
  ret i8 %_0
}

; Function Attrs: nofree noreturn nosync nounwind memory(none)
define hidden void @rust_begin_unwind(ptr noalias nocapture noundef readonly align 1 dereferenceable(16) %info) unnamed_addr addrspace(1) #2 {
start:
  tail call addrspace(1) void @rust_begin_unwind(ptr noalias noundef nonnull readonly align 1 dereferenceable(16) %info) #4
  unreachable
}

; Function Attrs: nounwind
declare dso_local void @delay(i16 noundef) unnamed_addr addrspace(1) #0

attributes #0 = { nounwind "target-cpu"="atmega328" }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) "target-cpu"="atmega328" }
attributes #2 = { nofree noreturn nosync nounwind memory(none) "target-cpu"="atmega328" }
attributes #3 = { nounwind }
attributes #4 = { noreturn nounwind }

!llvm.ident = !{!0}

!0 = !{!"rustc version 1.83.0-nightly (c2f74c3f9 2024-09-09)"}
