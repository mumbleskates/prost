use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::{max, min};
use core::marker::PhantomData;
use core::mem::{self, transmute, MaybeUninit};
use core::ptr;

use bytes::Buf;

const MIN_CHUNK_SIZE: usize = 2 * mem::size_of::<&[u8]>();

/// A prepend-only byte buffer.
///
/// It is not guaranteed to be efficient to interleave reads via `bytes::Buf` and writes via
/// `ReverseBuf::prepend`.
pub trait ReverseBuf: Buf {
    /// Prepends bytes to the buffer. These bytes will still be in the order they appear in the
    /// provided `Buf` when they are read back, but they will appear immediately before any bytes
    /// already written to the buffer.
    fn prepend<B: Buf>(&mut self, data: B);

    // --- Provided: ---

    #[inline]
    fn prepend_slice(&mut self, data: &[u8]) {
        self.prepend(data)
    }

    #[inline]
    fn prepend_u8(&mut self, n: u8) {
        let src = [n];
        self.prepend(src.as_slice());
    }

    #[inline]
    fn prepend_i8(&mut self, n: i8) {
        let src = [n as u8];
        self.prepend(src.as_slice());
    }

    #[inline]
    fn prepend_u16_le(&mut self, n: u16) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_i16_le(&mut self, n: i16) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_u32_le(&mut self, n: u32) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_i32_le(&mut self, n: i32) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_u64_le(&mut self, n: u64) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_i64_le(&mut self, n: i64) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_f32_le(&mut self, n: f32) {
        self.prepend_slice(&n.to_le_bytes())
    }

    #[inline]
    fn prepend_f64_le(&mut self, n: f64) {
        self.prepend_slice(&n.to_le_bytes())
    }
}

/// A `bytes`-compatible, exponentially-growing, prepend-only byte buffer.
///
/// `ReverseBuf` is rope-like in that it stores its data non-contiguously, but does not (yet)
/// support any rope-like operations.
#[derive(Clone)]
pub struct ReverseBuffer {
    /// Chunks of owned items in reverse order.
    chunks: Vec<Box<[MaybeUninit<u8>]>>,
    /// Index of the first item in the front chunk (at the end of `self.chunks`). Invariant: Always
    /// a valid index in that chunk when any chunk exists.
    front: usize,
    /// Total size of owned bytes in the chunks, including the uninitialized items at the front of
    /// the front chunk.
    capacity: usize,
    /// Advisory size value for when the next chunk is allocated. If this value is positive it is an
    /// exact size for the next allocation(s); otherwise it is a negated minimum added capacity that
    /// was requested.
    planned_capacity: isize,
    _phantom_data: PhantomData<u8>,
}

impl ReverseBuffer {
    /// Creates a new empty buffer.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            front: 0,
            capacity: 0,
            planned_capacity: 0,
            _phantom_data: PhantomData,
        }
    }

    /// Returns the number of bytes written into this buffer so far.
    #[inline]
    pub fn len(&self) -> usize {
        debug_assert!(self.capacity > self.front);
        self.capacity - self.front
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        // Front should be zero any time the buf is empty
        debug_assert!(!(self.chunks.is_empty() && self.front > 0));
        // If there are no chunks capacity should also be zero, and chunks should always have
        // nonzero size.
        debug_assert_eq!(self.chunks.is_empty(), self.capacity == 0);
        self.chunks.is_empty()
    }

    /// Returns the number of bytes this buffer currently has allocated capacity for.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Ensures that the buffer will, upon its next allocation, reserve at least enough space to fit
    /// this many more bytes than are currently in the buffer.
    #[inline(always)]
    pub fn plan_reservation(&mut self, additional: usize) {
        let Some(more_needed) = additional.checked_sub(self.front) else {
            return; // There is already enough capacity for `additional` more bytes.
        };
        if self.planned_capacity.unsigned_abs() > more_needed {
            return; // Next planned allocation is already greater than the requested amount.
        }
        self.planned_capacity = -(more_needed as isize);
    }

    /// Ensures that the buffer will, upon its next allocation, reserve enough space to fit this
    /// many more bytes than are in the buffer at present. If there is already enough additional
    /// capacity to fit this many more bytes, this method has no effect.
    ///
    /// If this method is repeatedly called interleaved with calls to `prepend` that trigger new
    /// allocations, the buffer may become very fragmented as this method can be used to control the
    /// exact sizes of all its allocations. Use sparingly.
    #[inline]
    pub fn plan_reservation_exact(&mut self, additional: usize) {
        let Some(more_needed) = additional.checked_sub(self.front) else {
            return;
        };
        self.planned_capacity = (self.capacity + more_needed) as isize;
    }

    /// Returns the slice of bytes ordered at the front of the buffer.
    #[inline]
    fn front_chunk_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.chunks.last_mut().map(Box::as_mut).unwrap_or(&mut [])
    }

    /// Returns a reader that references this buf's value, which implements `bytes::Buf` without
    /// draining bytes from the buffer.
    pub fn reader(&self) -> ReverseBufferReader {
        ReverseBufferReader {
            chunks: self.chunks.as_slice(),
            front: self.front,
            capacity: self.capacity,
        }
    }
}

impl ReverseBuf for ReverseBuffer {
    #[inline]
    fn prepend<B: Buf>(&mut self, mut data: B) {
        let copy_data = |data: &mut B, mut dest_chunk: &mut [MaybeUninit<u8>]| {
            while !dest_chunk.is_empty() {
                let src = data.chunk();
                let copy_size = min(src.len(), dest_chunk.len());
                // SAFETY: we are initializing dest_chunk with bytes from `data`.
                unsafe {
                    ptr::copy_nonoverlapping(
                        src.as_ptr(),
                        dest_chunk.as_mut_ptr() as *mut u8,
                        copy_size,
                    );
                }
                dest_chunk = &mut dest_chunk[copy_size..];
                data.advance(copy_size);
            }
        };

        let prepending_len = data.remaining();
        if prepending_len == 0 {
            return;
        }
        if prepending_len <= self.front {
            // The data fits in our current front chunk; copy it into there and update the front
            // index.
            let new_front = self.front - prepending_len;
            let dest_range = new_front..self.front;
            // SAFETY: We have a nonzero `front`, therefore we must have a front chunk.
            copy_data(&mut data, &mut self.front_chunk_mut()[dest_range]);
            // Copy each chunk of `data` into this uninitialized destination.
            debug_assert_eq!(data.remaining(), 0);
            self.front = new_front; // Those bytes are now initialized.
        } else {
            self.plan_reservation(prepending_len);

            let new_chunk_size = if self.planned_capacity > 0 {
                // We planned an explicit exact size for the next allocation.
                self.planned_capacity as usize
            } else {
                // We planned a minimum size for the new chunk. Choose the actual size for that
                // allocation, planning to at least double in size.
                max(
                    max(MIN_CHUNK_SIZE, self.capacity),
                    self.planned_capacity.unsigned_abs(),
                )
            };

            // Create the ownership of the new chunk
            let mut new_chunk: Box<[MaybeUninit<u8>]>;
            unsafe {
                let new_allocation =
                    alloc::alloc::alloc(alloc::alloc::Layout::array::<u8>(new_chunk_size).unwrap())
                        as *mut MaybeUninit<u8>;
                new_chunk = Box::from_raw(ptr::slice_from_raw_parts_mut(
                    new_allocation,
                    new_chunk_size,
                ));
            };

            // Copy bytes from the provided buffer into our two slices, the early half at the end of
            // the new chunk we just allocated, and the rest at the beginning of the "current" front
            // chunk.
            let old_front = self.front;
            let new_front = new_chunk_size + self.front - prepending_len;
            debug_assert!(new_front < new_chunk.len());
            copy_data(&mut data, unsafe {
                new_chunk.get_unchecked_mut(new_front..)
            });
            debug_assert!(self
                .chunks
                .last()
                .map_or(true, |old_front_chunk| old_front < old_front_chunk.len()));
            copy_data(&mut data, unsafe {
                self.front_chunk_mut().get_unchecked_mut(..old_front)
            });
            debug_assert_eq!(data.remaining(), 0);
            // Data is all written; update our state.
            self.chunks.push(new_chunk); // new_chunk becomes the new front chunk
            self.front = new_front; // front now points to the first initialized index there
            self.capacity += new_chunk_size; // update our capacity
            self.planned_capacity = 0; // reset planned capacity since we already allocated
        }
    }
}

impl Default for ReverseBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// The implementation of `bytes::Buf` for `ReverseBuf` drains bytes from the buffer as they are
/// advanced past. Calls to `bytes::Buf::advance` undo any planned reservations.
impl Buf for ReverseBuffer {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn chunk(&self) -> &[u8] {
        let Some(front_chunk) = self.chunks.last() else {
            return &[];
        };
        debug_assert!(self.front < front_chunk.len());
        // SAFETY: front is always a valid index in the front chunk, and the bytes at and
        // after that index are always initialized.
        unsafe { transmute(front_chunk.get_unchecked(self.front..)) }
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        if cnt == 0 {
            return;
        }
        if cnt > self.len() {
            panic!("advanced past end");
        };
        // Un-plan any particular growth.
        self.planned_capacity = 0;
        // `front` becomes the number of bytes from the front that the new front of the buffer will
        // be. This temporarily breaks our invariant that `front` must always be a valid index in
        // the front chunk, so we will be removing chunks and subtracting from front until it fits.
        self.front += cnt;
        // Pop chunks off the front of the buffer until the new front doesn't overflow the front
        // chunk
        while let Some(front_chunk) = self.chunks.last() {
            let front_chunk_size = front_chunk.len();
            if self.front < front_chunk_size {
                break;
            }
            drop(self.chunks.pop());
            self.capacity -= front_chunk_size;
            self.front -= front_chunk_size;
        }
    }
}

/// Non-draining reader-by-reference for `ReverseBuf`, implementing `bytes::Buf`.
pub struct ReverseBufferReader<'a> {
    /// Buffer being read
    chunks: &'a [Box<[MaybeUninit<u8>]>],
    /// Index of the front byte in the front chunk (the last in the slice). If chunks is non-empty,
    /// front is always a valid index inside it.
    front: usize,
    /// Total size of all the boxes covered by chunks
    capacity: usize,
}

impl Buf for ReverseBufferReader<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.capacity - self.front
    }

    #[inline(always)]
    fn chunk(&self) -> &[u8] {
        let Some(front_chunk) = self.chunks.last() else {
            return &[];
        };
        debug_assert!(self.front < front_chunk.len());
        // SAFETY: front is always a valid index in the front chunk, and the bytes at and
        // after that index are always initialized.
        unsafe { transmute(front_chunk.get_unchecked(self.front..)) }
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        if cnt == 0 {
            return;
        }
        if cnt > self.remaining() {
            panic!("advanced past end");
        };
        // `front` becomes the number of bytes from the front that the new front of the buffer will
        // be. This temporarily breaks our invariant that `front` must always be a valid index in
        // the front chunk, so we will be removing chunks and subtracting from front until it fits.
        self.front += cnt;
        // Pop chunks off until the new front doesn't overflow the front chunk
        while let Some(front_chunk) = self.chunks.last() {
            if self.front < front_chunk.len() {
                break;
            }
            // Snip off the first chunk from the end of the slice (chunks are in reverse order,
            // remember)
            self.chunks = &self.chunks[..self.chunks.len() - 1];
            self.front -= front_chunk.len();
            self.capacity -= front_chunk.len();
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ReverseBuf, ReverseBuffer};
    use alloc::vec::Vec;
    use bytes::{Buf, BufMut};

    fn compare_buf(buf: impl Buf, expected: &[u8]) {
        let mut read = Vec::new();
        read.put(buf);
        assert_eq!(read, expected);
    }

    fn check_read(buf: ReverseBuffer, expected: &[u8]) {
        assert_eq!(buf.len(), expected.len());
        assert_eq!(buf.is_empty(), buf.len() == 0);
        compare_buf(buf.reader(), expected);
        compare_buf(buf, expected);
    }

    #[test]
    fn fresh() {
        check_read(ReverseBuffer::new(), b"");
    }

    #[test]
    fn fresh_with_plan_still_empty() {
        let mut buf = ReverseBuffer::new();
        buf.plan_reservation(100);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        check_read(buf, b"");
    }

    #[test]
    fn build_and_read() {
        let mut buf = ReverseBuffer::new();
        buf.prepend(b"!".as_slice());
        buf.prepend(b"world".as_slice());
        buf.prepend(b"hello ".as_slice());
        check_read(buf, b"hello world!");
    }

    #[test]
    fn build_bigger_and_read() {
        let mut buf = ReverseBuffer::new();
        buf.prepend(b"!".as_slice());
        buf.prepend(b"world".as_slice());
        buf.prepend(b"hello ".as_slice());
        let mut buf2 = ReverseBuffer::new();
        buf2.prepend(buf.reader()); // 1
        buf.prepend(buf2.reader()); // 2
        buf2.prepend(buf.reader()); // 3
        buf.prepend(buf2.reader()); // 5
        buf2.prepend(buf.reader()); // 8
        buf.prepend(buf2.reader()); // 13
        assert_eq!(buf.chunks.len(), 3);
        check_read(
            buf,
            b"hello world!hello world!hello world!hello world!hello world!hello world!\
            hello world!hello world!hello world!hello world!hello world!hello world!hello world!",
        );
    }

    #[test]
    fn build_with_planned_reservation() {
        let mut buf = ReverseBuffer::new();
        buf.prepend(b"!".as_slice());
        buf.prepend(b"world".as_slice());
        buf.prepend(b"hello ".as_slice());
        buf.plan_reservation(b"hello world!".len() * 12);
        let mut buf2 = ReverseBuffer::new();
        buf2.prepend(buf.reader()); // 1
        buf.prepend(buf2.reader()); // 2
        buf2.prepend(buf.reader()); // 3
        buf.prepend(buf2.reader()); // 5
        buf2.prepend(buf.reader()); // 8
        buf.prepend(buf2.reader()); // 13
                                    // Only one additional chunk was allocated
        assert_eq!(buf.chunks.len(), 2);
        // No extra capacity exists in the buffer at this point
        assert_eq!(buf.capacity() - buf.len(), 0);
        check_read(
            buf,
            b"hello world!hello world!hello world!hello world!hello world!hello world!\
            hello world!hello world!hello world!hello world!hello world!hello world!hello world!",
        );
    }

    #[test]
    fn build_with_initial_planned_reservation() {
        let mut buf = ReverseBuffer::new();
        buf.plan_reservation(b"hello world!".len() * 13);
        buf.prepend(b"!".as_slice());
        buf.prepend(b"world".as_slice());
        buf.prepend(b"hello ".as_slice());
        let mut buf2 = ReverseBuffer::new();
        buf2.prepend(buf.reader()); // 1
        buf.prepend(buf2.reader()); // 2
        buf2.prepend(buf.reader()); // 3
        buf.prepend(buf2.reader()); // 5
        buf2.prepend(buf.reader()); // 8
        buf.prepend(buf2.reader()); // 13
        assert_eq!(buf.chunks.len(), 1); // Only one chunk was allocated in total
        assert_eq!(buf.capacity() - buf.len(), 0); // No extra capacity exists in the buffer
        check_read(
            buf,
            b"hello world!hello world!hello world!hello world!hello world!hello world!\
            hello world!hello world!hello world!hello world!hello world!hello world!hello world!",
        );
    }

    #[test]
    fn build_with_exact_planned_reservation() {
        let mut buf = ReverseBuffer::new();
        buf.plan_reservation_exact(b"hello world!".len());
        buf.prepend(b"!".as_slice());
        buf.prepend(b"world".as_slice());
        buf.prepend(b"hello ".as_slice());
        assert_eq!(buf.chunks.len(), 1); // Only one chunk was allocated in total
        assert_eq!(buf.capacity() - buf.len(), 0); // No extra capacity exists in the buffer
        check_read(buf, b"hello world!");
    }
}
