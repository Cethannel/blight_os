pub mod headers;

use x86_64::instructions::port::Port;

use crate::println;

pub fn pci_config_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let mut outport = Port::new(0xCF8);
    let mut inport: x86_64::instructions::port::PortGeneric<
        u32,
        x86_64::instructions::port::ReadWriteAccess,
    > = Port::new(0xCFC);

    let address: u32;
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;

    address = (lbus << 16) | (lslot << 11) | (lfunc << 8) | (offset & 0xFC) as u32 | 0x80000000;

    unsafe {
        outport.write(address);
        ((inport.read() >> ((offset & 2) * 8)) & 0xFFFF) as u16
    }
}

fn get_base_class(bus: u8, slot: u8, function: u8) -> u8 {
    (pci_config_read_word(bus, slot, function, 0x8 + 0x2) >> 8) as u8
}

fn get_sub_class(bus: u8, slot: u8, function: u8) -> u8 {
    (pci_config_read_word(bus, slot, function, 0x8 + 0x2) & 0xFF) as u8
}

fn pci_check_vendor(bus: u8, slot: u8) -> Option<u16> {
    let vendor: u16;
    let device: u16;
    vendor = pci_config_read_word(bus, slot, 0, 0);
    if vendor != 0xFFFF {
        device = pci_config_read_word(bus, slot, 0, 2);
        Some(vendor)
    } else {
        None
    }
}

fn check_device(bus: u8, device: u8) {
    let function: u8 = 0;
    let vendor_id: u16;
    vendor_id = pci_config_read_word(bus, device, function, 0);
    if vendor_id != 0xFFFF {
        check_function(bus, device, function);
        let header = headers::Header::new(bus, device, function);
        println!("PCI: Header : {:?}", header);
        let header_type: u8 = pci_config_read_word(bus, device, function, 0x0E) as u8;
        if (header_type & 0x80) != 0 {
            for function in 1..8 {
                if pci_config_read_word(bus, device, function, 0) != 0xFFFF {
                    check_function(bus, device, function);
                    println!(
                        "PCI: Found multi-function device: bus: {}, device: {}",
                        bus, device
                    );
                }
            }
        }
    }
}

fn get_secondary_bus(bus: u8, device: u8, function: u8) -> u8 {
    (pci_config_read_word(bus, device, function, 0x18) >> 8) as u8
}

pub fn check_bus(bus: u8) {
    for device in 0..32 {
        check_device(bus, device);
    }
}

fn check_function(bus: u8, device: u8, function: u8) {
    let base_class: u8;
    let sub_class: u8;
    let secondary_bus: u8;

    base_class = get_base_class(bus, device, function);
    sub_class = get_sub_class(bus, device, function);
    if (base_class == 0x6) && (sub_class == 0x4) {
        secondary_bus = get_secondary_bus(bus, device, function);
        check_bus(secondary_bus);
        println!(
            "PCI: Found PCI-to-PCI bridge: bus: {}, device: {}, function: {}",
            bus, device, function
        );
    } else {
        println!(
            "PCI: Found device: bus: {}, device: {}, function: {}, base class: {}, sub class: {}",
            bus, device, function, base_class, sub_class
        );
    }
}
