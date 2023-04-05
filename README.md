# peak_mem_alloc

An instrumenting middleware for global allocators in Rust, useful to find the peak memory consumed by a function.

## Example

```rust
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

``` 

## Custom allocators

Currenty wrapping a custom allocator requires the use of the nightly compiler
and compiling with the "nightly" feature due to the soon to stabilize use of
the unstable `const_fn_trait_bound` and the fact that the internals of the
instrumenting type are not public. If that's fine with you, a custom allocator
can be wrapped as follows:

```rust
#[global_allocator]
static GLOBAL: PeakAlloc<System> = PeakAlloc::new(MyCustomAllocator::new());
```
