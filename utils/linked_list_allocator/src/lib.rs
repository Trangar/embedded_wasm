#![no_std]

extern crate alloc;

use core::{
    alloc::{GlobalAlloc, Layout},
    hint::unreachable_unchecked,
};

// #[global_allocator]
// pub static ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

pub struct LinkedListAllocator {
    start: *mut u8,
    len: usize,
}

unsafe impl Sync for LinkedListAllocator {}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            start: core::ptr::null_mut(),
            len: 0,
        }
    }

    pub fn init(&mut self, mem: &mut [u8]) {
        self.start = mem.as_mut_ptr();
        self.len = mem.len();
    }

    // unsafe fn ptr(&self, idx: usize, len: usize) -> Option<*const [u8]> {
    //     let end = idx + len;
    //     if end > self.len {
    //         None
    //     } else {
    //         Some(core::ptr::slice_from_raw_parts(self.start.add(idx), len))
    //     }
    // }

    unsafe fn ptr_mut(&self, idx: usize, len: usize) -> Option<*mut [u8]> {
        let end = idx + len;
        if end > self.len {
            None
        } else {
            Some(core::ptr::slice_from_raw_parts_mut(
                self.start.add(idx),
                len,
            ))
        }
    }

    // unsafe fn obj_at<T>(&self, idx: usize) -> Option<&'static T> {
    //     let ptr = self.ptr(idx, core::mem::size_of::<T>())?;
    //     Some(&*ptr.cast())
    // }
    unsafe fn mut_obj_at<T>(&self, idx: usize) -> Option<&'static mut T> {
        let ptr = self.ptr_mut(idx, core::mem::size_of::<T>())?;
        Some(&mut *ptr.cast())
    }

    fn ptr_to_idx<T>(&self, ptr: *const T) -> usize {
        (ptr as usize) - (self.start as usize)
    }

    fn first_header(&self) -> &'static mut Header {
        unsafe { self.mut_obj_at(0) }.unwrap()
    }
    fn header_at(&self, idx: usize) -> Option<&'static mut Header> {
        unsafe { self.mut_obj_at(idx) }
    }

    fn read_next_header(&self, header: &mut Header) -> Option<&'static mut Header> {
        let idx = self.ptr_to_idx(header) + header.len() as usize + core::mem::size_of::<Header>();
        unsafe { self.mut_obj_at(idx) }
    }

    fn clear_empty_entries(&self) {
        enum Result {
            Occupied,
            Unoccupied,
        }
        fn inner(ll: &LinkedListAllocator, header: &mut Header) -> Result {
            if header.occupied() {
                if let Some(next) = ll.read_next_header(header) {
                    inner(ll, next);
                }
                Result::Occupied
            } else {
                let mut next_occupied = None;
                let mut next = ll.read_next_header(header);
                while let Some(next_header) = next {
                    if next_header.occupied() {
                        next_occupied = Some(next_header);
                        break;
                    }
                    next = ll.read_next_header(next_header);
                }
                if let Some(next) = next_occupied {
                    inner(ll, next);
                    Result::Occupied
                } else {
                    header.set_len(0);
                    Result::Unoccupied
                }
            }
        }
        inner(self, self.first_header());
    }
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() < u16::MAX as usize);
        let mut header = self.first_header();
        loop {
            if !header.occupied() {
                let mut occupied = false;
                if header.len() == 0 {
                    header.set_len(layout.size() as u16);
                    occupied = true;
                    // make sure the next header is emptied
                    if let Some(next) = self.read_next_header(header) {
                        next.clear();
                    } else {
                        // We're out of memory
                        return core::ptr::null_mut();
                    }
                }
                if layout.size() <= header.len() as usize {
                    occupied = true;
                }

                if occupied {
                    header.set_occupied(true);
                    return header.data_ptr();
                }
            }
            header = match self.read_next_header(header) {
                Some(header) => header,
                None => return core::ptr::null_mut(),
            };
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: core::alloc::Layout) {
        let idx = self.ptr_to_idx(ptr) - core::mem::size_of::<Header>();
        if let Some(header) = self.header_at(idx) {
            header.set_occupied(false);

            if let Some(next) = self.read_next_header(header) {
                if next.len() == 0 {
                    self.clear_empty_entries();
                }
            }
        }
    }
}

#[repr(transparent)]
struct Header([u8; 4]);

impl Header {
    const OCCUPIED: u8 = 0b1000_0000;
    fn len(&self) -> u16 {
        u16::from_le_bytes(match self.0[..2].try_into() {
            Ok(b) => b,
            Err(_) => unsafe { unreachable_unchecked() },
        })
    }
    fn occupied(&self) -> bool {
        self.0[2] & Self::OCCUPIED > 0
    }

    // fn ptr(&mut self, len: usize) -> NonNull<[u8]> {
    //     unsafe {
    //         let ptr = self as *mut Header as *mut u8;
    //         let ptr = ptr.add(core::mem::size_of::<Header>());
    //         let slice = core::ptr::slice_from_raw_parts_mut(ptr, len);
    //         NonNull::new_unchecked(slice)
    //     }
    // }

    fn data_ptr(&mut self) -> *mut u8 {
        unsafe { (self as *mut Header as *mut u8).add(core::mem::size_of::<Header>()) }
    }

    fn set_occupied(&mut self, occupied: bool) {
        if occupied {
            self.0[2] |= Self::OCCUPIED;
        } else {
            self.0[2] &= !Self::OCCUPIED;
        }
    }

    fn set_len(&mut self, len: u16) {
        self.0[..2].copy_from_slice(&len.to_le_bytes());
    }

    fn clear(&mut self) {
        for b in &mut self.0 {
            *b = 0;
        }
    }
}
