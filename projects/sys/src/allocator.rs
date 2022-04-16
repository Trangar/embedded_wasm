use alloc::alloc::GlobalAlloc;
use core::alloc::Layout;

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        crate::ffi::alloc(layout.size(), layout.align())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        crate::ffi::dealloc(ptr, layout.size(), layout.align())
    }
}

#[cfg(target_family = "wasm")]
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

#[cfg(target_family = "wasm")]
#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("Out of memory");
}
