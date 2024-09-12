; ModuleID = 'main.8db5165aed74ea44-cgu.0'
source_filename = "main.8db5165aed74ea44-cgu.0"
target datalayout = "e-P1-p:16:8-i8:8-i16:8-i32:8-i64:8-f32:8-f64:8-n8-a:8"
target triple = "avr-unknown-unknown"

; Function Attrs: nounwind
define dso_local void @main() unnamed_addr addrspace(1) #0 {
start:
  tail call addrspace(1) void @"color __sdcccall(0)"(i8 noundef 3, i8 noundef 0, i8 noundef 0) #3
  br label %bb3.preheader

bb3.preheader:                                    ; preds = %start, %bb3.preheader
  %x.sroa.0.06 = phi i8 [ 20, %start ], [ %0, %bb3.preheader ]
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 20, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 30, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 40, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 50, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 60, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 70, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 80, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 90, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 100, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 110, i8 noundef 15, i8 noundef 0) #3
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 120, i8 noundef 15, i8 noundef 0) #3
  %0 = add nuw i8 %x.sroa.0.06, 10
  %_4 = icmp ult i8 %x.sroa.0.06, -121
  br i1 %_4, label %bb3.preheader, label %bb7

bb7:                                              ; preds = %bb3.preheader
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local noundef i32 @add(i32 noundef %left, i32 noundef %right) unnamed_addr addrspace(1) #1 {
start:
  %_0 = mul i32 %right, %left
  ret i32 %_0
}

; Function Attrs: nofree noreturn nosync nounwind memory(none)
define hidden void @rust_begin_unwind(ptr noalias nocapture noundef readonly align 1 dereferenceable(16) %info) unnamed_addr addrspace(1) #2 {
start:
  tail call addrspace(1) void @rust_begin_unwind(ptr noalias noundef nonnull readonly align 1 dereferenceable(16) %info) #4
  unreachable
}

; Function Attrs: nounwind
declare dso_local void @"color __sdcccall(0)"(i8 noundef, i8 noundef, i8 noundef) unnamed_addr addrspace(1) #0

; Function Attrs: nounwind
declare dso_local void @"circle __sdcccall(0)"(i8 noundef, i8 noundef, i8 noundef, i8 noundef) unnamed_addr addrspace(1) #0

attributes #0 = { nounwind "target-cpu"="atmega328" }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) "target-cpu"="atmega328" }
attributes #2 = { nofree noreturn nosync nounwind memory(none) "target-cpu"="atmega328" }
attributes #3 = { nounwind }
attributes #4 = { noreturn nounwind }

!llvm.ident = !{!0}

!0 = !{!"rustc version 1.83.0-nightly (c2f74c3f9 2024-09-09)"}
