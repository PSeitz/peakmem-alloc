extern crate peak_mem_alloc;

use peak_mem_alloc::{PeakAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &PeakAlloc<System> = &INSTRUMENTED_SYSTEM;

#[test]
fn example_using_region() {
    GLOBAL.reset_peak_memory();
    let _x: Vec<u8> = Vec::with_capacity(1_024);
    println!(
        "Peak Memory used by function : {:#?}",
        GLOBAL.get_peak_memory()
    );
}
