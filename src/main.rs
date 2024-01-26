#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blight_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blight_os::memory::BootInfoFrameAllocator;
    println!("Hello World{}", "!");

    blight_os::init();

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    #[cfg(test)]
    test_main();

    blight_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blight_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blight_os::test_panic_handler(info)
}
