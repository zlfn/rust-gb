; ModuleID = 'main.8db5165aed74ea44-cgu.0'
source_filename = "main.8db5165aed74ea44-cgu.0"
target datalayout = "e-P1-p:16:8-i8:8-i16:8-i32:8-i64:8-f32:8-f64:8-n8-a:8"
target triple = "avr-unknown-unknown"

; Function Attrs: nounwind
define dso_local void @main() unnamed_addr addrspace(1) #0 {
start:
  tail call addrspace(1) void @"color __sdcccall(0)"(i8 noundef 3, i8 noundef 0, i8 noundef 0) #12
  br label %bb3.preheader

bb3.preheader:                                    ; preds = %start, %bb3.preheader
  %x.sroa.0.06 = phi i8 [ 20, %start ], [ %0, %bb3.preheader ]
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 20, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 30, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 40, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 50, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 60, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 70, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 80, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 90, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 100, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 110, i8 noundef 15, i8 noundef 0) #12
  tail call addrspace(1) void @"circle __sdcccall(0)"(i8 noundef %x.sroa.0.06, i8 noundef 120, i8 noundef 15, i8 noundef 0) #12
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
  tail call addrspace(1) void @rust_begin_unwind(ptr noalias noundef nonnull readonly align 1 dereferenceable(16) %info) #13
  unreachable
}

; Function Attrs: mustprogress nofree nounwind willreturn memory(inaccessiblemem: readwrite)
define hidden noalias noundef ptr @__rust_alloc(i16 noundef %size, i16 noundef %align) unnamed_addr addrspace(1) #3 {
start:
  %0 = icmp ne i16 %align, 0
  tail call addrspace(1) void @llvm.assume(i1 %0)
  %1 = icmp ult i16 %align, -32767
  tail call addrspace(1) void @llvm.assume(i1 %1)
  %_4.i = tail call noalias noundef addrspace(1) ptr @malloc(i16 noundef %size) #12
  ret ptr %_4.i
}

; Function Attrs: mustprogress nounwind willreturn memory(argmem: readwrite, inaccessiblemem: readwrite)
define hidden void @__rust_dealloc(ptr nocapture noundef %ptr, i16 noundef %size, i16 noundef %align) unnamed_addr addrspace(1) #4 {
start:
  %0 = icmp ne i16 %align, 0
  tail call addrspace(1) void @llvm.assume(i1 %0)
  %1 = icmp ult i16 %align, -32767
  tail call addrspace(1) void @llvm.assume(i1 %1)
  tail call addrspace(1) void @free(ptr noundef %ptr) #12
  ret void
}

; Function Attrs: mustprogress nounwind willreturn
define hidden noalias noundef ptr @__rust_realloc(ptr nocapture noundef %ptr, i16 noundef %size, i16 noundef %align, i16 noundef %new_size) unnamed_addr addrspace(1) #5 {
start:
  %0 = icmp ne i16 %align, 0
  tail call addrspace(1) void @llvm.assume(i1 %0)
  %1 = icmp ult i16 %align, -32767
  tail call addrspace(1) void @llvm.assume(i1 %1)
  %_4.i.i = tail call noalias noundef addrspace(1) ptr @malloc(i16 noundef %new_size) #12
  %2 = icmp eq ptr %_4.i.i, null
  br i1 %2, label %"_ZN68_$LT$main..LibcAlloc$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7realloc17h3ef0212bdae7f97aE.exit", label %bb3.i

bb3.i:                                            ; preds = %start
  %size.sroa.0.0.i = tail call addrspace(1) i16 @llvm.umin.i16(i16 %size, i16 %new_size)
  tail call addrspace(1) void @llvm.memcpy.p0.p0.i16(ptr nonnull align 1 %_4.i.i, ptr align 1 %ptr, i16 %size.sroa.0.0.i, i1 false)
  tail call addrspace(1) void @free(ptr noundef %ptr) #12
  br label %"_ZN68_$LT$main..LibcAlloc$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7realloc17h3ef0212bdae7f97aE.exit"

"_ZN68_$LT$main..LibcAlloc$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7realloc17h3ef0212bdae7f97aE.exit": ; preds = %start, %bb3.i
  ret ptr %_4.i.i
}

; Function Attrs: mustprogress nofree nounwind willreturn memory(inaccessiblemem: readwrite)
define hidden noalias noundef ptr @__rust_alloc_zeroed(i16 noundef %size, i16 noundef %align) unnamed_addr addrspace(1) #3 {
start:
  %0 = icmp ne i16 %align, 0
  tail call addrspace(1) void @llvm.assume(i1 %0)
  %1 = icmp ult i16 %align, -32767
  tail call addrspace(1) void @llvm.assume(i1 %1)
  %calloc.i = tail call noalias noundef addrspace(1) ptr @calloc(i16 1, i16 %size)
  ret ptr %calloc.i
}

; Function Attrs: mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite)
declare dso_local noalias noundef ptr @malloc(i16 noundef) unnamed_addr addrspace(1) #6

; Function Attrs: mustprogress nounwind willreturn allockind("free") memory(argmem: readwrite, inaccessiblemem: readwrite)
declare dso_local void @free(ptr allocptr nocapture noundef) unnamed_addr addrspace(1) #7

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(inaccessiblemem: write)
declare void @llvm.assume(i1 noundef) addrspace(1) #8

; Function Attrs: mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i16(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i16, i1 immarg) addrspace(1) #9

; Function Attrs: nounwind
declare dso_local void @"color __sdcccall(0)"(i8 noundef, i8 noundef, i8 noundef) unnamed_addr addrspace(1) #0

; Function Attrs: nounwind
declare dso_local void @"circle __sdcccall(0)"(i8 noundef, i8 noundef, i8 noundef, i8 noundef) unnamed_addr addrspace(1) #0

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i16 @llvm.umin.i16(i16, i16) addrspace(1) #10

; Function Attrs: nofree nounwind willreturn allockind("alloc,zeroed") allocsize(0,1) memory(inaccessiblemem: readwrite)
declare noalias noundef ptr @calloc(i16 noundef, i16 noundef) local_unnamed_addr addrspace(1) #11

attributes #0 = { nounwind "target-cpu"="atmega328" }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) "target-cpu"="atmega328" }
attributes #2 = { nofree noreturn nosync nounwind memory(none) "target-cpu"="atmega328" }
attributes #3 = { mustprogress nofree nounwind willreturn memory(inaccessiblemem: readwrite) "target-cpu"="atmega328" }
attributes #4 = { mustprogress nounwind willreturn memory(argmem: readwrite, inaccessiblemem: readwrite) "target-cpu"="atmega328" }
attributes #5 = { mustprogress nounwind willreturn "target-cpu"="atmega328" }
attributes #6 = { mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" "target-cpu"="atmega328" }
attributes #7 = { mustprogress nounwind willreturn allockind("free") memory(argmem: readwrite, inaccessiblemem: readwrite) "alloc-family"="malloc" "target-cpu"="atmega328" }
attributes #8 = { mustprogress nocallback nofree nosync nounwind willreturn memory(inaccessiblemem: write) }
attributes #9 = { mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #10 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #11 = { nofree nounwind willreturn allockind("alloc,zeroed") allocsize(0,1) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" }
attributes #12 = { nounwind }
attributes #13 = { noreturn nounwind }

!llvm.ident = !{!0}

!0 = !{!"rustc version 1.83.0-nightly (c2f74c3f9 2024-09-09)"}
