use crate::{
    pci::headers::{Header, HeaderType, BAR},
    println,
};

use x86_64::instructions::port::{Port, PortWriteOnly};

#[derive(Debug)]
pub enum Rtl8139Error {
    InvalidHeader,
}

pub fn enable(header: &Header) -> Result<(), Rtl8139Error> {
    if header.vendor_id != 0x10EC {
        println!("Vendor ID: {}", header.vendor_id);
        return Err(Rtl8139Error::InvalidHeader);
    }

    if header.device_id != 0x8139 {
        println!("Device ID: {}", header.device_id);
        return Err(Rtl8139Error::InvalidHeader);
    }

    let mut command = header.command;

    command |= 0x4;

    let port = Port::<u16>::new(0x4);

    unsafe {
        port.write(command);
    }
}

pub fn turn_on(header: &Header) -> Result<u16, Rtl8139Error> {
    if header.vendor_id != 0x10EC {
        println!("Vendor ID: {}", header.vendor_id);
        return Err(Rtl8139Error::InvalidHeader);
    }

    if header.device_id != 0x8139 {
        println!("Device ID: {}", header.device_id);
        return Err(Rtl8139Error::InvalidHeader);
    }

    match &header.rest_of_header {
        HeaderType::Standard(rest) => {
            let io_base = if let BAR::IO(base) = &rest.base_address_registers[0] {
                base
            } else {
                println!("Invalid BAR");
                return Err(Rtl8139Error::InvalidHeader);
            };

            let mut port = PortWriteOnly::<u8>::new(io_base.address + 0x52);

            unsafe {
                port.write(0x10);
            }

            Ok(io_base.address)
        }
        _ => return Err(Rtl8139Error::InvalidHeader),
    }
}

#[derive(Debug)]
pub struct Rtl8139 {
    io_base: u16,
}

impl Rtl8139 {
    pub fn new(header: Header) -> Result<Rtl8139, Rtl8139Error> {
        if header.vendor_id != 0x10EC {
            println!("Vendor ID: {}", header.vendor_id);
            return Err(Rtl8139Error::InvalidHeader);
        }

        if header.device_id != 0x8139 {
            println!("Device ID: {}", header.device_id);
            return Err(Rtl8139Error::InvalidHeader);
        }

        turn_on(&header)?;

        Ok(Rtl8139 { io_base: 0 })
    }

    pub fn software_reset(&self) {
        let mut port = Port::<u8>::new(self.io_base + 0x37);

        unsafe {
            port.write(0x10);
            while (port.read() & 0x10) != 0 {
                println!("Waiting for reset");
            }
        }
    }
}
