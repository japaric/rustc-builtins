use core::intrinsics::likely;

const WORD_SIZE: usize = core::mem::size_of::<usize>();
const WORD_MASK: usize = WORD_SIZE - 1;

const WORD_COPY_THRESHOLD: usize = 2 * WORD_SIZE;

#[inline(always)]
unsafe fn copy_forward_bytes(mut dest: *mut u8, mut src: *const u8, n: usize) {
    let dest_end = dest.add(n);
    while dest < dest_end {
        *dest = *src;
        dest = dest.add(1);
        src = src.add(1);
    }
}

#[inline(always)]
unsafe fn copy_forward_aligned_words(dest: *mut u8, src: *const u8, n: usize) {
    let mut dest_usize = dest as *mut usize;
    let mut src_usize = src as *mut usize;
    let dest_end = dest.add(n) as *mut usize;

    while dest_usize < dest_end {
        *dest_usize = *src_usize;
        dest_usize = dest_usize.add(1);
        src_usize = src_usize.add(1);
    }
}

#[inline(always)]
unsafe fn copy_forward_misaligned_words(dest: *mut u8, src: *const u8, n: usize) {
    let mut dest_usize = dest as *mut usize;
    let dest_end = dest.add(n) as *mut usize;

    // Calculate the misalignment offset and shift needed to reassemble value.
    let offset = src as usize & WORD_MASK;
    let shift = offset * 8;

    // Realign src
    let mut src_aligned = (src as usize & !WORD_MASK) as *mut usize;
    // XXX: Could this possibly be UB?
    let mut prev_word = *src_aligned;

    while dest_usize < dest_end {
        src_aligned = src_aligned.add(1);
        let cur_word = *src_aligned;
        let resembled = prev_word >> shift | cur_word << (WORD_SIZE * 8 - shift);
        prev_word = cur_word;

        *dest_usize = resembled;
        dest_usize = dest_usize.add(1);
    }
}

#[inline(always)]
pub unsafe fn copy_forward(mut dest: *mut u8, mut src: *const u8, mut n: usize) {
    if likely(n >= WORD_COPY_THRESHOLD) {
        // Align dest
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        let dest_misalignment = (dest as usize).wrapping_neg() & WORD_MASK;
        copy_forward_bytes(dest, src, dest_misalignment);
        dest = dest.add(dest_misalignment);
        src = src.add(dest_misalignment);
        n -= dest_misalignment;

        let n_words = n & !WORD_MASK;
        let src_misalignment = src as usize & WORD_MASK;
        if likely(src_misalignment == 0) {
            copy_forward_aligned_words(dest, src, n_words);
        } else {
            copy_forward_misaligned_words(dest, src, n_words);
        }
        dest = dest.add(n_words);
        src = src.add(n_words);
        n -= n_words;
    }
    copy_forward_bytes(dest, src, n);
}

// The following backward copy helper functions except the public-facing copy_backward
// uses the pointers past the end as their inputs instead of pointers to the start!

#[inline(always)]
unsafe fn copy_backward_bytes(mut dest: *mut u8, mut src: *const u8, n: usize) {
    let dest_start = dest.sub(n);
    while dest_start < dest {
        dest = dest.sub(1);
        src = src.sub(1);
        *dest = *src;
    }
}

#[inline(always)]
unsafe fn copy_backward_aligned_words(dest: *mut u8, src: *const u8, n: usize) {
    let mut dest_usize = dest as *mut usize;
    let mut src_usize = src as *mut usize;
    let dest_start = dest.sub(n) as *mut usize;

    while dest_start < dest_usize {
        dest_usize = dest_usize.sub(1);
        src_usize = src_usize.sub(1);
        *dest_usize = *src_usize;
    }
}

#[inline(always)]
unsafe fn copy_backward_misaligned_words(dest: *mut u8, src: *const u8, n: usize) {
    let mut dest_usize = dest as *mut usize;
    let dest_start = dest.sub(n) as *mut usize;

    // Calculate the misalignment offset and shift needed to reassemble value.
    let offset = src as usize & WORD_MASK;
    let shift = offset * 8;

    // Realign src_aligned
    let mut src_aligned = (src as usize & !WORD_MASK) as *mut usize;
    // XXX: Could this possibly be UB?
    let mut prev_word = *src_aligned;

    while dest_start < dest_usize {
        src_aligned = src_aligned.sub(1);
        let cur_word = *src_aligned;
        let resembled = prev_word << (WORD_SIZE * 8 - shift) | cur_word >> shift;
        prev_word = cur_word;

        dest_usize = dest_usize.sub(1);
        *dest_usize = resembled;
    }
}

#[inline(always)]
pub unsafe fn copy_backward(dest: *mut u8, src: *const u8, mut n: usize) {
    let mut dest = dest.add(n);
    let mut src = src.add(n);

    if likely(n >= WORD_COPY_THRESHOLD) {
        // Align dest
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        let dest_misalignment = dest as usize & WORD_MASK;
        copy_backward_bytes(dest, src, dest_misalignment);
        dest = dest.sub(dest_misalignment);
        src = src.sub(dest_misalignment);
        n -= dest_misalignment;

        let n_words = n & !WORD_MASK;
        let src_misalignment = src as usize & WORD_MASK;
        if likely(src_misalignment == 0) {
            copy_backward_aligned_words(dest, src, n_words);
        } else {
            copy_backward_misaligned_words(dest, src, n_words);
        }
        dest = dest.sub(n_words);
        src = src.sub(n_words);
        n -= n_words;
    }
    copy_backward_bytes(dest, src, n);
}

#[inline(always)]
pub unsafe fn set_bytes_bytes(mut s: *mut u8, c: u8, n: usize) {
    let end = s.add(n);
    while s < end {
        *s = c;
        s = s.add(1);
    }
}

#[inline(always)]
pub unsafe fn set_bytes_words(s: *mut u8, c: u8, n: usize) {
    let mut broadcast = c as usize;
    broadcast |= broadcast << 8;
    #[cfg(not(target_pointer_width = "16"))]
    {
        broadcast |= broadcast << 16;
        #[cfg(not(target_pointer_width = "32"))]
        {
            broadcast |= broadcast << 32;
            #[cfg(not(target_pointer_width = "64"))]
            {
                broadcast |= broadcast << 64;
            }
        }
    }

    let mut s_usize = s as *mut usize;
    let end = s.add(n) as *mut usize;

    while s_usize < end {
        *s_usize = broadcast;
        s_usize = s_usize.add(1);
    }
}

#[inline(always)]
pub unsafe fn set_bytes(mut s: *mut u8, c: u8, mut n: usize) {
    if likely(n >= WORD_COPY_THRESHOLD) {
        // Bec// Align dest
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        let misalignment = (s as usize).wrapping_neg() & WORD_MASK;
        set_bytes_bytes(s, c, misalignment);
        s = s.add(misalignment);
        n -= misalignment;

        let n_words = n & !WORD_MASK;
        set_bytes_words(s, c, n_words);
        s = s.add(n_words);
        n -= n_words;
    }
    set_bytes_bytes(s, c, n);
}
