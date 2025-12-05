use alloc::alloc::{Layout, alloc};

// The world's dumbest allocator. Just keep bumping a pointer until we run out
// of memory, in which case we abort. StringCache is responsible for creating
// a new allocator when that's about to happen.
//
// This is now bumping downward rather than up, which simplifies the allocate()
// method and gives a small (5-7%) performance improvement in multithreaded
// benchmarks
//
// See https://fitzgeraldnick.com/2019/11/01/always-bump-downwards.html
pub(crate) struct LeakyBumpAlloc {
    layout: Layout,
    start: *mut u8,
    end: *mut u8,
    ptr: *mut u8,
}

impl LeakyBumpAlloc {
    pub fn new(capacity: usize, alignment: usize) -> LeakyBumpAlloc {
        let layout = Layout::from_size_align(capacity, alignment).unwrap();
        // SAFETY: TODO
        let start = unsafe { alloc(layout) };
        if start.is_null() {
            panic!("oom");
        }
        let end = unsafe { start.add(layout.size()) };
        let ptr = end;
        LeakyBumpAlloc {
            layout,
            start,
            end,
            ptr,
        }
    }

    // Allocates a new chunk. Aborts if out of memory.
    pub unsafe fn allocate(&mut self, num_bytes: usize) -> *mut u8 {
        // Our new ptr will be offset down the heap by num_bytes bytes.
        let ptr = self.ptr as usize;
        let new_ptr = ptr.checked_sub(num_bytes).expect("ptr sub overflowed");
        // Round down to alignment.
        let new_ptr = new_ptr & !(self.layout.align() - 1);
        // Check we have enough capacity.
        let start = self.start as usize;
        if new_ptr < start {
            // We have to abort here rather than panic or the mutex may
            // deadlock.
            libabort::abort();
        }

        // SAFETY: TODO
        self.ptr = unsafe { self.ptr.sub(ptr - new_ptr) };
        self.ptr
    }

    pub fn allocated(&self) -> usize {
        self.end as usize - self.ptr as usize
    }

    pub fn capacity(&self) -> usize {
        self.layout.size()
    }
}
