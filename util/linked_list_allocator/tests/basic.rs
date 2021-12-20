use linked_list_allocator::LinkedListAllocator;
use std::alloc::{GlobalAlloc, Layout};

#[test]
fn simple_alloc() {
    let mut global_mem = vec![0u8; 1000];
    let global_mem_ptr = global_mem.as_ptr();
    let mut allocator = LinkedListAllocator::new();
    allocator.init(&mut global_mem);

    let ptr = unsafe { allocator.alloc(Layout::array::<u32>(10).unwrap()) };
    assert!(!ptr.is_null());
    let offset = ptr as usize - global_mem_ptr as usize;
    assert_eq!(offset, 4);
    assert_eq!(&global_mem[..2], &[40, 0]);
    assert_eq!(global_mem[2] & 0b1000_0000, 0b1000_0000);

    let slice = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u32, 10) };
    assert_eq!(&global_mem[4..8], &[0, 0, 0, 0]);
    slice[0] = 5;
    dbg!(slice);
    assert_eq!(&global_mem[4..8], &[5, 0, 0, 0]);

    let second_ptr = unsafe { allocator.alloc(Layout::array::<u32>(8).unwrap()) };
    assert!(!second_ptr.is_null());
    let offset = second_ptr as usize - global_mem_ptr as usize;
    assert_eq!(offset, 48);
    assert_eq!(&global_mem[44..46], &[32, 0]);
    assert_eq!(global_mem[46] & 0b1000_0000, 0b1000_0000);

    unsafe { allocator.dealloc(second_ptr, Layout::array::<u32>(8).unwrap()) }
    assert_eq!(&global_mem[44..46], &[0, 0]);
    assert_eq!(global_mem[46] & 0b1000_0000, 0);

    unsafe { allocator.dealloc(ptr, Layout::array::<u32>(10).unwrap()) }
    assert_eq!(&global_mem[..2], &[0, 0]);
    assert_eq!(global_mem[2] & 0b1000_0000, 0);
}

#[test]
fn reclaim_multiple() {
    let mut global_mem = vec![0u8; 100];
    let global_mem_ptr = global_mem.as_ptr();
    let mut allocator = LinkedListAllocator::new();
    allocator.init(&mut global_mem);

    let layout = Layout::new::<u32>();

    let ptrs = [
        unsafe { allocator.alloc(layout) },
        unsafe { allocator.alloc(layout) },
        unsafe { allocator.alloc(layout) },
        unsafe { allocator.alloc(layout) },
        unsafe { allocator.alloc(layout) },
    ];

    assert_eq!(ptrs[0] as usize - global_mem_ptr as usize, 4);
    assert_eq!(ptrs[1] as usize - global_mem_ptr as usize, 12);
    assert_eq!(ptrs[2] as usize - global_mem_ptr as usize, 20);
    assert_eq!(ptrs[3] as usize - global_mem_ptr as usize, 28);
    assert_eq!(ptrs[4] as usize - global_mem_ptr as usize, 36);

    for offset in [0, 8, 16, 24, 32] {
        assert_eq!(global_mem[offset], 4);
        assert_eq!(global_mem[offset + 1], 0);
        assert_eq!(global_mem[offset + 2] & 0b1000_0000, 0b1000_0000);
    }

    // Dealloc pointers 1, 2 and 3
    for ptr in ptrs[1..4].iter().copied() {
        let offset = (ptr as usize - global_mem_ptr as usize) - 4;

        assert_eq!(global_mem[offset], 4);
        assert_eq!(global_mem[offset + 1], 0);
        assert_eq!(global_mem[offset + 2] & 0b1000_0000, 0b1000_0000);
        unsafe { allocator.dealloc(ptr, layout) };
        assert_eq!(global_mem[offset], 4);
        assert_eq!(global_mem[offset + 1], 0);
        assert_eq!(global_mem[offset + 2] & 0b1000_0000, 0);
    }

    // assert len of first alloc and second
    assert_eq!(global_mem[0], 4);
    assert_eq!(global_mem[1], 0);
    assert_eq!(global_mem[8], 4);
    assert_eq!(global_mem[9], 0);
    // assert len of last alloc
    assert_eq!(global_mem[32], 4);
    assert_eq!(global_mem[33], 0);
    assert_eq!(global_mem[34] & 0b1000_0000, 0b1000_0000); // last alloc is still present
                                                           // dealloc the last one, should dealloc everything but the first one
    unsafe { allocator.dealloc(ptrs[4], layout) };
    assert_eq!(global_mem[34] & 0b1000_0000, 0); // last alloc is cleared
                                                 // second alloc should also be cleared
    assert_eq!(global_mem[8], 0);
    assert_eq!(global_mem[9], 0);
    // first alloc should be fine
    assert_eq!(global_mem[0], 4);
    assert_eq!(global_mem[1], 0);
    assert_eq!(global_mem[2] & 0b1000_0000, 0b1000_0000);

    // Next alloc should be at the 2nd index
    let ptr = unsafe { allocator.alloc(Layout::new::<u128>()) };
    assert_eq!(ptr as usize - global_mem_ptr as usize, 12);
    assert_eq!(global_mem[8], 16);
    assert_eq!(global_mem[9], 0);
    assert_eq!(global_mem[10] & 0b1000_0000, 0b1000_0000);
}

#[test]
fn test_oom() {
    let mut global_mem = vec![0u8; 100];
    let global_mem_ptr = global_mem.as_ptr();
    let mut allocator = LinkedListAllocator::new();
    allocator.init(&mut global_mem);

    // Does not fit, should return a nullptr
    let layout = Layout::array::<u32>(100).unwrap();

    let ptr = unsafe { allocator.alloc(layout) };
    assert!(ptr.is_null());

    // still works
    let ptr = unsafe { allocator.alloc(Layout::new::<u32>()) };
    assert!(!ptr.is_null());
    let offset = ptr as usize - global_mem_ptr as usize;
    assert_eq!(offset, 4);

    unsafe { allocator.dealloc(ptr, Layout::new::<u32>()) };

    // fits only 1 of these
    let layout = Layout::array::<u32>(50).unwrap();

    let ptr = unsafe { allocator.alloc(layout) };
    assert!(!ptr.is_null());
    let offset = ptr as usize - global_mem_ptr as usize;
    assert_eq!(offset, 4);

    // 2nd one should fail
    let ptr = unsafe { allocator.alloc(layout) };
    assert!(ptr.is_null());
}
