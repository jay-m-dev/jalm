#![cfg_attr(not(any(test, feature = "std")), no_std)]

#[cfg(test)]
extern crate std;

use core::sync::atomic::{AtomicUsize, Ordering};

const ALIGN: usize = 8;

#[cfg(target_arch = "wasm32")]
const PAGE_SIZE: usize = 65536;

#[cfg(not(target_arch = "wasm32"))]
const HEAP_SIZE: usize = 1024 * 1024;

#[cfg(target_arch = "wasm32")]
extern "C" {
    static __heap_base: u8;
}

#[cfg(not(target_arch = "wasm32"))]
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn align_up(value: usize) -> usize {
    (value + (ALIGN - 1)) & !(ALIGN - 1)
}

#[cfg(target_arch = "wasm32")]
fn heap_base() -> usize {
    unsafe { &__heap_base as *const u8 as usize }
}

#[cfg(not(target_arch = "wasm32"))]
fn heap_base() -> usize {
    core::ptr::addr_of!(HEAP) as usize
}

#[cfg(target_arch = "wasm32")]
fn ensure_memory(end: usize) -> bool {
    let needed_pages = (end + PAGE_SIZE - 1) / PAGE_SIZE;
    let current_pages = unsafe { core::arch::wasm32::memory_size(0) as usize };
    if needed_pages <= current_pages {
        return true;
    }
    let grow = (needed_pages - current_pages) as i32;
    let result = unsafe { core::arch::wasm32::memory_grow(0, grow) };
    result != -1
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_memory(end: usize) -> bool {
    end <= heap_base() + HEAP_SIZE
}

#[no_mangle]
pub extern "C" fn jalm_alloc(size: usize) -> *mut u8 {
    let size = align_up(size.max(1));
    let mut current = NEXT.load(Ordering::Relaxed);
    if current == 0 {
        current = heap_base();
    }
    let start = align_up(current);
    let end = match start.checked_add(size) {
        Some(end) => end,
        None => return core::ptr::null_mut(),
    };

    if !ensure_memory(end) {
        return core::ptr::null_mut();
    }

    NEXT.store(end, Ordering::Relaxed);
    start as *mut u8
}

#[no_mangle]
pub extern "C" fn jalm_realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if ptr.is_null() {
        return jalm_alloc(new_size);
    }
    if new_size == 0 {
        return core::ptr::null_mut();
    }

    let new_ptr = jalm_alloc(new_size);
    if new_ptr.is_null() {
        return core::ptr::null_mut();
    }

    let copy_len = core::cmp::min(old_size, new_size);
    unsafe {
        core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_len);
    }
    new_ptr
}

#[no_mangle]
pub extern "C" fn jalm_free(_ptr: *mut u8, _size: usize) {
    // Bump allocator: free is a no-op in v0.
}

#[no_mangle]
pub extern "C" fn jalm_bytes_alloc(len: usize) -> *mut u8 {
    jalm_alloc(len)
}

#[no_mangle]
pub extern "C" fn jalm_bytes_clone(src: *const u8, len: usize) -> *mut u8 {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    let dst = jalm_alloc(len);
    if dst.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len);
    }
    dst
}

#[no_mangle]
pub extern "C" fn jalm_memcpy(dst: *mut u8, src: *const u8, len: usize) -> *mut u8 {
    if dst.is_null() || src.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len);
    }
    dst
}

#[no_mangle]
pub extern "C" fn jalm_memset(dst: *mut u8, value: u8, len: usize) -> *mut u8 {
    if dst.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        core::ptr::write_bytes(dst, value, len);
    }
    dst
}

#[no_mangle]
pub extern "C" fn jalm_panic(_code: u32) -> ! {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        core::arch::wasm32::unreachable();
    }

    #[cfg(not(target_arch = "wasm32"))]
    panic!("jalm_panic");
}

#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        core::arch::wasm32::unreachable();
    }

    #[cfg(not(target_arch = "wasm32"))]
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_heap() {
        NEXT.store(0, Ordering::Relaxed);
        unsafe {
            let ptr = core::ptr::addr_of_mut!(HEAP) as *mut u8;
            core::ptr::write_bytes(ptr, 0, HEAP_SIZE);
        }
    }

    struct TestGuard {
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl TestGuard {
        fn new() -> Self {
            let guard = TEST_LOCK.lock().unwrap();
            reset_heap();
            Self { _guard: guard }
        }
    }

    #[test]
    fn alloc_returns_unique_regions() {
        let _guard = TestGuard::new();
        let a = jalm_alloc(8);
        let b = jalm_alloc(8);
        assert!(!a.is_null());
        assert!(!b.is_null());
        assert!(b as usize > a as usize);
    }

    #[test]
    fn realloc_copies_bytes() {
        let _guard = TestGuard::new();
        let p = jalm_alloc(4);
        unsafe {
            core::ptr::write(p, 1);
            core::ptr::write(p.add(1), 2);
            core::ptr::write(p.add(2), 3);
            core::ptr::write(p.add(3), 4);
        }
        let q = jalm_realloc(p, 4, 8);
        assert!(!q.is_null());
        unsafe {
            assert_eq!(*q, 1);
            assert_eq!(*q.add(1), 2);
            assert_eq!(*q.add(2), 3);
            assert_eq!(*q.add(3), 4);
        }
    }

    #[test]
    fn memcpy_and_memset_work() {
        let _guard = TestGuard::new();
        let src = jalm_alloc(4);
        let dst = jalm_alloc(4);
        unsafe {
            core::ptr::write(src, 9);
            core::ptr::write(src.add(1), 8);
            core::ptr::write(src.add(2), 7);
            core::ptr::write(src.add(3), 6);
        }
        jalm_memcpy(dst, src, 4);
        unsafe {
            assert_eq!(*dst, 9);
            assert_eq!(*dst.add(3), 6);
        }
        jalm_memset(dst, 0xAA, 4);
        unsafe {
            assert_eq!(*dst, 0xAA);
            assert_eq!(*dst.add(2), 0xAA);
        }
    }

    #[test]
    fn bytes_clone_duplicates_data() {
        let _guard = TestGuard::new();
        let src = jalm_alloc(3);
        unsafe {
            core::ptr::write(src, 1);
            core::ptr::write(src.add(1), 2);
            core::ptr::write(src.add(2), 3);
        }
        let dst = jalm_bytes_clone(src, 3);
        assert!(!dst.is_null());
        unsafe {
            assert_eq!(*dst, 1);
            assert_eq!(*dst.add(1), 2);
            assert_eq!(*dst.add(2), 3);
        }
    }
}
