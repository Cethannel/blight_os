#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blight_os::networking::ethernet;
use blight_os::pci::drivers::rtl8139;
use blight_os::pci::find_network_card;
use blight_os::pci::get_network_card;
use blight_os::println;
use blight_os::task::keyboard;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

use blight_os::task::executor::Executor;
use blight_os::task::Task;

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

    let network_card = get_network_card().unwrap();

    println!("Netcard: {:?}", network_card);

    println!("Vendor ID: {:X}", network_card.vendor_id);
    println!("Device ID: {:X}", network_card.device_id);

    assert_eq!(network_card.vendor_id, 0x10EC);
    assert_eq!(network_card.device_id, 0x8139);

    let mut rtl8139 = rtl8139::Rtl8139::new(phys_mem_offset).unwrap();

    println!("Mac: {:?}", rtl8139.mac);

    let mut packet = ethernet::Packet::new();

    packet.dest = rtl8139.mac;
    packet.src = rtl8139.mac;
    for i in 0..6 {
        packet.data[i] = i as u8;
    }

    packet.len = 6;

    rtl8139.send_packet(&packet.to_slice());

    println!("You can now use the network card");

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
