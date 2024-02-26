use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ptr;

use bytes::Buf;

const MIN_CHUNK_SIZE: usize = 2 * mem::size_of::<&[u8]>();

pub struct ReverseBuf {
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

impl ReverseBuf {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            front: 0,
            capacity: 0,
            planned_capacity: 0,
            _phantom_data: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.capacity - self.front
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn unused_capacity(&self) -> usize {
        self.front
    }

    #[inline(always)]
    pub fn plan_reservation(&mut self, additional: usize) {
        let Some(more_needed) = additional.checked_sub(self.unused_capacity()) else {
            return; // There is already enough capacity for `additional` more bytes.
        };
        if self.planned_capacity.unsigned_abs() > more_needed {
            return; // Next planned allocation is already greater than the requested amount.
        }
        self.planned_capacity = -(more_needed as isize);
    }

    #[inline]
    pub fn plan_reservation_exact(&mut self, additional: usize) {
        let Some(more_needed) = additional.checked_sub(self.unused_capacity()) else {
            return;
        };
        self.planned_capacity = (self.capacity + more_needed) as isize;
    }

    #[inline]
    fn front_chunk_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.chunks.last_mut().map(Box::as_mut).unwrap_or(&mut [])
    }

    #[inline]
    pub fn prepend<B: Buf>(&mut self, mut data: B) {
        let copy_data = |data: &mut B, mut dest_chunk: &mut [MaybeUninit<u8>]| {
            while !dest_chunk.is_empty() {
                let src = data.chunk();
                // SAFETY: we are just initializing dest_chunk with bytes from `data`.
                unsafe {
                    ptr::copy_nonoverlapping(
                        src.as_ptr(),
                        dest_chunk.as_mut_ptr() as *mut u8,
                        src.len(),
                    );
                }
                dest_chunk = &mut dest_chunk[src.len()..];
                data.advance(src.len());
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
                [
                    MIN_CHUNK_SIZE,
                    self.capacity,
                    self.planned_capacity.unsigned_abs(),
                ]
                .into_iter()
                .min()
                .unwrap()
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
            debug_assert!(old_front < self.front_chunk_mut().len());
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

impl Default for ReverseBuf {
    fn default() -> Self {
        Self::new()
    }
}

// TODO(widders): clone

// impl Buf for ReverseBuf<u8> {
//     #[inline]
//     fn remaining(&self) -> usize {
//         self.len()
//     }
//
//     #[inline(always)]
//     fn chunk(&self) -> &[u8] {
//         let Some(front_chunk) = self.chunks.last() else {
//             return &[];
//         };
//         debug_assert!(self.front < front_chunk.len());
//         unsafe { transmute(front_chunk.get_unchecked(self.front..)) }
//     }
//
//     #[inline]
//     fn advance(&mut self, cnt: usize) {
//         if cnt == 0 {
//             return;
//         }
//         if cnt > self.len() {
//             panic!("advanced past end");
//         };
//         debug_assert!(!self.chunks.is_empty());
//         // Front chunk will be a reference to the current front chunk.
//         let mut front_chunk = unsafe { self.chunks.last_mut().unwrap_unchecked() };
//         // Live front will be the index of the first live item in the current front chunk.
//         let mut live_front = self.front;
//         // New front is how far from the front of the front chunk the real front of the buffer is.
//         // We will drop chunks and subtract their length from this value until it is an index in the
//         // new front chunk.
//         let mut new_front = self.front + cnt;
//         let mut new_capacity = self.capacity;
//
//         // Drop chunks for as long as they are obsoleted
//         while new_front >= front_chunk.len() {
//             let mut to_drop = unsafe { self.chunks.pop().unwrap_unchecked() };
//             unsafe {
//                 for item in to_drop.get_unchecked_mut(live_front..) {
//                     drop_in_place(item.as_mut_ptr())
//                 }
//             }
//             new_front -= to_drop.len();
//             new_capacity -= to_drop.len();
//             drop(to_drop);
//             live_front = 0;
//             // Get the new front chunk
//             front_chunk = match self.chunks.last_mut() {
//                 None => {
//                     // We've deleted all our chunks and should be empty now.
//                     debug_assert_eq!(new_front, 0);
//                     break;
//                 }
//                 Some(front_chunk) => front_chunk,
//             };
//         }
//
//         unsafe { for item in front_chunk.get_unchecked_mut(live_front..new_front) {} }
//
//         // TODO: this
//         self.front = live_front;
//         debug_assert!(
//             self.chunks
//                 .last()
//                 .map(|front_chunk| front_chunk.len() - 1)
//                 .unwrap_or(0)
//                 <= self.front
//         );
//     }
// }
