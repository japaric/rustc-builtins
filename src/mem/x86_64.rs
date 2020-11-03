// On most modern Intel and AMD processors, "rep movsq" and "rep stosq" have
// been enhanced to perform better than an simple qword loop, making them ideal
// for implementing memcpy/memset. Note that "rep cmps" has received no such
// enhancement, so it is not used to implement memcmp.
//
// On certain recent Intel processors, "rep movsb" and "rep stosb" have been
// further enhanced to automatically select the best microarchitectural
// implementation based on length and alignment. See the following features from
// the "Intel® 64 and IA-32 Architectures Optimization Reference Manual":
//  - ERMSB - Enhanced REP MOVSB and STOSB (Ivy Bridge and later)
//  - FSRM - Fast Short REP MOV (Ice Lake and later)
//  - Fast Zero-Length MOVSB (On no current hardware)
//  - Fast Short STOSB (On no current hardware)
// However, to avoid run-time feature detection, we don't use these byte-based
// instructions for most of the copying, preferring the qword variants.

#[inline(always)]
pub unsafe fn copy_forward(dest: *mut u8, src: *const u8, count: usize) {
    let qword_count = count >> 3;
    let byte_count = count & 0b111;
    asm!(
        "rep movsq [rdi], [rsi]",
        "mov ecx, {byte_count:e}",
        "rep movsb [rdi], [rsi]",
        byte_count = in(reg) byte_count,
        inout("rcx") qword_count => _,
        inout("rdi") dest => _,
        inout("rsi") src => _,
        options(nostack, preserves_flags)
    );
}

#[inline(always)]
pub unsafe fn copy_backward(dest: *mut u8, src: *const u8, count: usize) {
    let qword_count = count >> 3;
    let byte_count = count & 0b111;
    asm!(
        "std",
        "rep movsq [rdi], [rsi]",
        "mov ecx, {byte_count:e}",
        "add rdi, 7",
        "add rsi, 7",
        "rep movsb [rdi], [rsi]",
        "cld",
        byte_count = in(reg) byte_count,
        inout("rcx") qword_count => _,
        inout("rdi") dest.offset(count as isize).wrapping_sub(8) => _,
        inout("rsi") src.offset(count as isize).wrapping_sub(8) => _,
        options(nostack)
    );
}

#[inline(always)]
pub unsafe fn set_bytes(dest: *mut u8, c: u8, count: usize) {
    let qword_count = count >> 3;
    let byte_count = count & 0b111;
    asm!(
        "rep stosq [rdi], rax",
        "mov ecx, {byte_count:e}",
        "rep stosb [rdi], al",
        byte_count = in(reg) byte_count,
        inout("rcx") qword_count => _,
        inout("rdi") dest => _,
        in("rax") (c as u64) * 0x0101010101010101,
        options(nostack, preserves_flags)
    );
}
