use crate::{
    memory,
    pci::{
        self,
        headers::{Header, HeaderType, IOBar, BAR},
        pci_config_write_word,
    },
    println,
};

const BUFFER_SIZE: usize = 8192 + 16;

static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

use alloc::vec;
use alloc::vec::Vec;
use x86_64::{
    instructions::port::{Port, PortWriteOnly},
    VirtAddr,
};

#[derive(Debug)]
pub enum Rtl8139Error {
    InvalidHeader,
}

#[derive(Debug)]
pub struct Rtl8139 {
    bar: IOBar,
    eeprom_exists: bool,
    mac: [u8; 6],
    rx_buffer: Vec<u8>,
    tx_cur: u32,
}

const RTL8139_VENDOR_ID: u16 = 0x10EC;
const RTL8139_DEVICE_ID: u16 = 0x8139;

impl Rtl8139 {
    pub fn new(phys_mem_offset: VirtAddr) -> Result<Rtl8139, Rtl8139Error> {
        let header = pci::get_device(RTL8139_VENDOR_ID, RTL8139_DEVICE_ID).ok_or(Rtl8139Error::InvalidHeader)?;

        if header.vendor_id != 0x10EC {
            println!("Vendor ID: {}", header.vendor_id);
            return Err(Rtl8139Error::InvalidHeader);
        }

        if header.device_id != 0x8139 {
            println!("Device ID: {}", header.device_id);
            return Err(Rtl8139Error::InvalidHeader);
        }

        let io_base = if let BAR::IO(base) = &header
            .rest_of_header
            .to_standard()
            .map_err(|_| Rtl8139Error::InvalidHeader)?
            .base_address_registers[0]
        {
            base
        } else {
            println!("Invalid BAR");
            return Err(Rtl8139Error::InvalidHeader);
        };

        let bar = io_base;

        let mem_base = io_base.address & (!0x3);
        let io_base: u16 = (io_base.address & (!0xf)) as u16;

        println!("IO Base: {:#X}", io_base);
        println!("Mem Base: {:#X}", mem_base);
        println!("IO_BASE: {:#X}", bar.address);

        let mut pci_command_reg = header.command;

        /*
        if (pci_command_reg & (1 << 2)) == 0 {
            pci_command_reg |= 1 << 2;
            pci::pci_config_write_word(
                header.bus,
                header.device,
                header.function,
                0x4,
                pci_command_reg,
            );
        }
        */
        pci_command_reg |= 0b100;
        pci_config_write_word(header.bus, header.device, header.function, 0x4, pci_command_reg);

        let mut outport = Port::<u8>::new(io_base as u16 + 0x52);

        unsafe {
            outport.write(0x0);
        }

        let mut outport = Port::<u8>::new(io_base as u16 + 0x37);

        unsafe {
            outport.write(0x10);
            while (outport.read() & 0x10) != 0 {
                x86_64::instructions::hlt();
            }
        }

        println!("Reset complete");

        let rx_buffer = vec![0; 8192 + 16 + 1500];

        let mut outport = Port::<u32>::new(io_base as u16 + 0x30);

        unsafe {
            outport.write(
                memory::translate_addr(VirtAddr::new(rx_buffer.as_ptr() as u64), phys_mem_offset)
                    .ok_or(Rtl8139Error::InvalidHeader)?
                    .as_u64() as u32,
            );
        }

        // Set TOK and ROK
        let mut outport = Port::<u8>::new(io_base as u16 + 0x3C);

        unsafe {
            outport.write(0x0005);
        }

        let mut outport = Port::<u32>::new(io_base as u16 + 0x44);

        unsafe {
            outport.write(0xf | (1 << 7));
        }

        let mut outport = Port::<u8>::new(io_base as u16 + 0x37);

        unsafe {
            outport.write(0x0C);
        }

        let mut out = Rtl8139 {
            bar: bar.clone(),
            eeprom_exists: false,
            mac: [0; 6],
            rx_buffer,
            tx_cur: 0,
        };

        out.set_mac_address();

        Ok(out)
    }

    pub fn set_mac_address(&mut self) {
        let mut mac_part1_port = Port::<u32>::new(self.bar.address as u16 + 0);
        let mut mac_part2_port = Port::<u16>::new(self.bar.address as u16 + 4);

        let mac_part1 = unsafe { mac_part1_port.read() };
        let mac_part2 = unsafe { mac_part2_port.read() };

        self.mac[0] = (mac_part1 >> 0) as u8;
        self.mac[1] = (mac_part1 >> 8) as u8;
        self.mac[2] = (mac_part1 >> 16) as u8;
        self.mac[3] = (mac_part1 >> 24) as u8;
        self.mac[4] = (mac_part2 >> 0) as u8;
        self.mac[5] = (mac_part2 >> 8) as u8;
    }
}
