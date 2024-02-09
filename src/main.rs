#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blight_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use blight_os::task::keyboard;

use blight_os::task::Task;
use blight_os::task::executor::Executor;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blight_os::allocator; // new import
    use blight_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    println!("Hello World{}", "!");

    blight_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    for bus in 0..=255 {
        blight_os::pci::check_bus(bus);
    }

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses())); // new
    executor.run();
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

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
