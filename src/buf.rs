use alloc::boxed::Box;
use alloc::vec;
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
        self.prepend_slice(&src);
    }

    #[inline]
    fn prepend_i8(&mut self, n: i8) {
        let src = [n as u8];
        self.prepend_slice(&src);
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
    planned_allocation: usize,
    planned_exact: bool,
    keep_back: bool,
    _phantom_data: PhantomData<u8>,
}

impl ReverseBuffer {
    /// Creates a new empty buffer. This buffer will not allocate until data is added.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            front: 0,
            capacity: 0,
            planned_allocation: 0,
            planned_exact: false,
            keep_back: false,
            _phantom_data: PhantomData,
        }
    }

    /// Creates a new buffer with a given base capacity. If the capacity is nonzero, it will always
    /// retain at least that much capacity even when fully read or cleared.
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }
        Self {
            chunks: vec![Self::allocate_chunk(capacity)],
            front: capacity,
            capacity,
            planned_allocation: 0,
            planned_exact: false,
            keep_back: true,
            _phantom_data: PhantomData,
        }
    }

    /// Returns the number of bytes written into this buffer so far.
    #[inline]
    pub fn len(&self) -> usize {
        // Front should be zero any time the buf is empty
        debug_assert!(!(self.chunks.is_empty() && self.front > 0));
        // If there are no chunks capacity should also be zero, and chunks should always have
        // nonzero size.
        debug_assert_eq!(self.chunks.is_empty(), self.capacity == 0);
        self.capacity - self.front
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the data from the buffer, including any additional allocations.
    pub fn clear(&mut self) {
        if self.keep_back {
            self.chunks.truncate(1);
            self.front = self.chunks[0].len();
            self.capacity = self.front;
            (self.planned_allocation, self.planned_exact) = (0, false);
        } else {
            *self = Self::new()
        }
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
        if self.planned_allocation > more_needed {
            return; // Next planned allocation is already greater than the requested amount.
        }
        (self.planned_allocation, self.planned_exact) = (more_needed, false);
    }

    /// Ensures that the buffer will, upon its next allocation, reserve enough space to fit this
    /// many more bytes than are in the buffer at present. If there is already enough additional
    /// capacity to fit this many more bytes, this method has no effect. If the requested capacity
    /// is not already met and there is already a set plan for the size of the next allocation, it
    /// will be overridden by this request.
    ///
    /// If this method is repeatedly called interleaved with calls to `prepend` that trigger new
    /// allocations, the buffer may become very fragmented as this method can be used to control the
    /// exact sizes of all its allocations. Use sparingly.
    #[inline]
    pub fn plan_reservation_exact(&mut self, additional: usize) {
        let Some(more_needed) = additional.checked_sub(self.front) else {
            return;
        };
        (self.planned_allocation, self.planned_exact) = (self.capacity + more_needed, true);
    }

    /// Returns the slice of bytes ordered at the front of the buffer.
    #[inline]
    fn front_chunk_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.chunks.last_mut().map(Box::as_mut).unwrap_or(&mut [])
    }

    #[inline]
    fn allocate_chunk(new_chunk_size: usize) -> Box<[MaybeUninit<u8>]> {
        debug_assert!(new_chunk_size > 0);
        unsafe {
            let new_allocation =
                alloc::alloc::alloc(alloc::alloc::Layout::array::<u8>(new_chunk_size).unwrap())
                    as *mut MaybeUninit<u8>;
            Box::from_raw(ptr::slice_from_raw_parts_mut(
                new_allocation,
                new_chunk_size,
            ))
        }
    }

    #[inline(never)]
    #[cold]
    fn grow_and_copy_buf<B: Buf>(&mut self, mut data: B, prepending_len: usize) {
        self.plan_reservation(prepending_len);

        let new_chunk_size = if self.planned_exact {
            // We planned an explicit exact size for the next allocation.
            debug_assert_ne!(self.planned_allocation, 0);
            self.planned_allocation
        } else {
            // We planned a minimum size for the new chunk. Choose the actual size for that
            // allocation, planning to at least double in size.
            max(max(MIN_CHUNK_SIZE, self.capacity), self.planned_allocation)
        };

        // Create the ownership of the new chunk
        let mut new_chunk = Self::allocate_chunk(new_chunk_size);

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
        self.capacity += new_chunk_size;
        // Reset the planned allocation since we already allocated
        (self.planned_allocation, self.planned_exact) = (0, false);
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

/// Copies bytes out of a `bytes::Buf` directly into a slice of uninitialized bytes, filling it. The
/// source must have enough bytes to fill the destination.
#[inline(always)]
fn copy_data<B: Buf>(data: &mut B, mut dest_chunk: &mut [MaybeUninit<u8>]) {
    debug_assert!(data.remaining() >= dest_chunk.len());
    while !dest_chunk.is_empty() {
        let src = data.chunk();
        let copy_size = min(src.len(), dest_chunk.len());
        // SAFETY: we are initializing dest_chunk with bytes from `data`.
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dest_chunk.as_mut_ptr() as *mut u8, copy_size);
        }
        dest_chunk = &mut dest_chunk[copy_size..];
        data.advance(copy_size);
    }
}

impl ReverseBuf for ReverseBuffer {
    #[inline(always)]
    fn prepend<B: Buf>(&mut self, mut data: B) {
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
            self.grow_and_copy_buf(data, prepending_len);
        }
    }
    // TODO(widders): try specializing provided methods and cheaper copies here
}

impl Default for ReverseBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// The implementation of `bytes::Buf` for `ReverseBuf` drains bytes from the buffer as they are
/// advanced past.
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
            // If we have exactly one chunk left and we are retaining the back chunk, don't drop it.
            if self.keep_back && self.chunks.len() == 1 {
                debug_assert!(self.capacity == self.front);
                break;
            }
            drop(self.chunks.pop());
            self.capacity -= front_chunk_size;
            self.front -= front_chunk_size;
            // Now that the buffer has shrunk, unplan any future allocations.
            (self.planned_allocation, self.planned_exact) = (0, false);
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
        buf.prepend_slice(b"!");
        buf.prepend_slice(b"world");
        buf.prepend_slice(b"hello ");
        check_read(buf, b"hello world!");
    }

    #[test]
    fn build_bigger_and_read() {
        let mut buf = ReverseBuffer::new();
        buf.prepend_slice(b"!");
        buf.prepend_slice(b"world");
        buf.prepend_slice(b"hello ");
        let mut buf2 = ReverseBuffer::new();
        buf2.prepend(buf.reader()); // 1
        buf.prepend(buf2.reader()); // 2
        buf2.prepend(buf.reader()); // 3
        buf.prepend(buf2.reader()); // 5
        buf2.prepend(buf.reader()); // 8
        buf.prepend(buf2.reader()); // 13
        assert_eq!(buf.chunks.len(), 3);
        check_read(
            buf.clone(),
            b"hello world!hello world!hello world!hello world!hello world!hello world!\
            hello world!hello world!hello world!hello world!hello world!hello world!hello world!",
        );
        check_read(
            buf,
            b"hello world!hello world!hello world!hello world!hello world!hello world!\
            hello world!hello world!hello world!hello world!hello world!hello world!hello world!",
        );
    }

    #[test]
    fn build_with_planned_reservation() {
        let mut buf = ReverseBuffer::new();
        buf.prepend_slice(b"!");
        buf.prepend_slice(b"world");
        buf.prepend_slice(b"hello ");
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
        buf.prepend_slice(b"!");
        buf.prepend_slice(b"world");
        buf.prepend_slice(b"hello ");
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
        buf.prepend_slice(b"!");
        buf.prepend_slice(b"world");
        buf.prepend_slice(b"hello ");
        assert_eq!(buf.chunks.len(), 1); // Only one chunk was allocated in total
        assert_eq!(buf.capacity() - buf.len(), 0); // No extra capacity exists in the buffer
        check_read(buf, b"hello world!");
    }

    #[test]
    fn build_with_capacity() {
        let mut buf = ReverseBuffer::with_capacity(b"hello world!".len());
        assert_eq!(buf.capacity(), b"hello world!".len());
        assert!(buf.is_empty());
        assert_eq!(buf.chunks.len(), 1);
        buf.prepend_slice(b"!");
        buf.prepend_slice(b"world");
        buf.prepend_slice(b"hello ");
        assert_eq!(buf.capacity(), b"hello world!".len());
        assert_eq!(buf.len(), buf.capacity());
        assert_eq!(buf.chunks.len(), 1);
        buf.prepend_slice(b"12345");
        assert_eq!(buf.chunks.len(), 2);
        assert!(buf.capacity() > buf.len());
        check_read(buf, b"12345hello world!");
    }

    #[test]
    fn single_prepend_allocates_once() {
        let mut buf = ReverseBuffer::with_capacity(4);
        buf.prepend_slice(b"aa");
        buf.prepend_slice(&[0; 100]);
        assert_eq!(buf.len(), 102);
        assert_eq!(buf.capacity(), buf.len());
        assert_eq!(buf.chunks.len(), 2);
        buf.clear();
        assert_eq!(buf.chunks.len(), 1);
        assert_eq!(buf.capacity(), 4);
        assert!(buf.is_empty());
    }
}
