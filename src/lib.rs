//!
//! An instrumenting allocator wrapper to compute (scoped) peak memory consumption.
//!
//! ## Example
//!
//! ```
//! extern crate peakmem_alloc;
//!
//! use peakmem_alloc::{PeakAlloc, INSTRUMENTED_SYSTEM};
//! use std::alloc::System;
//!
//! #[global_allocator]
//! static GLOBAL: &PeakAlloc<System> = &INSTRUMENTED_SYSTEM;
//!
//! fn main() {
//!    GLOBAL.reset_peak_memory();
//!    let _x: Vec<u8> = Vec::with_capacity(1_024);
//!    println!(
//!        "Peak Memory used by function : {:#?}",
//!        GLOBAL.get_peak_memory()
//!    );
//! }
//! ```

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_imports,
    unused_qualifications,
    missing_docs
)]
#![cfg_attr(doc_cfg, feature(allocator_api))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
};

/// An allocator middleware which keeps track of peak memory consumption.
#[derive(Default, Debug)]
pub struct PeakAlloc<T: GlobalAlloc> {
    peak_bytes_allocated_tracker: AtomicIsize,
    peak_bytes_allocated: AtomicUsize,
    inner: T,
}

/// An instrumented instance of the system allocator.
pub static INSTRUMENTED_SYSTEM: PeakAlloc<System> = PeakAlloc {
    peak_bytes_allocated_tracker: AtomicIsize::new(0),
    peak_bytes_allocated: AtomicUsize::new(0),
    inner: System,
};

impl PeakAlloc<System> {
    /// Provides access to an instrumented instance of the system allocator.
    pub const fn system() -> Self {
        PeakAlloc {
            peak_bytes_allocated_tracker: AtomicIsize::new(0),
            peak_bytes_allocated: AtomicUsize::new(0),
            inner: System,
        }
    }
}

impl<T: GlobalAlloc> PeakAlloc<T> {
    /// Provides access to an instrumented instance of the given global
    /// allocator.
    pub const fn new(inner: T) -> Self {
        PeakAlloc {
            peak_bytes_allocated_tracker: AtomicIsize::new(0),
            peak_bytes_allocated: AtomicUsize::new(0),
            inner,
        }
    }

    /// Resets the peak memory to 0
    pub fn reset_peak_memory(&self) {
        self.peak_bytes_allocated.store(0, Ordering::SeqCst);
        self.peak_bytes_allocated_tracker.store(0, Ordering::SeqCst);
    }

    /// Get the peak memory consumption
    pub fn get_peak_memory(&self) -> usize {
        self.peak_bytes_allocated.load(Ordering::SeqCst)
    }

    #[inline]
    fn track_alloc(&self, bytes: usize) {
        let prev = self
            .peak_bytes_allocated_tracker
            .fetch_add(bytes as isize, Ordering::SeqCst);
        let current_peak = (prev + bytes as isize).max(0) as usize;
        self.peak_bytes_allocated
            .fetch_max(current_peak, Ordering::SeqCst);
    }

    #[inline]
    fn track_dealloc(&self, bytes: usize) {
        self.peak_bytes_allocated_tracker
            .fetch_sub(bytes as isize, Ordering::SeqCst);
    }
}

unsafe impl<'a, T: GlobalAlloc + 'a> GlobalAlloc for &'a PeakAlloc<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self).alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self).dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (*self).alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        (*self).realloc(ptr, layout, new_size)
    }
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for PeakAlloc<T> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.track_alloc(layout.size());
        self.inner.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.track_dealloc(layout.size());
        self.inner.dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.track_alloc(layout.size());
        self.inner.alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > layout.size() {
            let difference = new_size - layout.size();
            self.track_alloc(difference);
        } else if new_size < layout.size() {
            let difference = layout.size() - new_size;
            self.track_dealloc(difference);
        }
        self.inner.realloc(ptr, layout, new_size)
    }
}
