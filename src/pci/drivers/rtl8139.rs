use crate::{
    interrupts::{self, IDT},
    memory,
    pci::{
        self,
        headers::{Header, HeaderType, IOBar, BAR},
        pci_config_write_word,
    },
    print, println,
};

const BUFFER_SIZE: usize = 8192 + 16;

static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

use x86_64::{
    instructions::port::{Port, PortWriteOnly},
    structures::idt::InterruptStackFrame,
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
    pub mac: [u8; 6],
    tx_cur: u32,
}

const RTL8139_VENDOR_ID: u16 = 0x10EC;
const RTL8139_DEVICE_ID: u16 = 0x8139;

impl Rtl8139 {
    pub fn new(phys_mem_offset: VirtAddr) -> Result<Rtl8139, Rtl8139Error> {
        let header = pci::get_device(RTL8139_VENDOR_ID, RTL8139_DEVICE_ID)
            .ok_or(Rtl8139Error::InvalidHeader)?;

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

        let io_base: u16 = io_base.address as u16;

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
        pci_command_reg |= 0b101;
        pci_config_write_word(
            header.bus,
            header.device,
            header.function,
            0x4,
            pci_command_reg,
        );

        let mut outport = Port::<u8>::new(io_base as u16 + 0x52);

        unsafe {
            outport.write(0x0);
        }

        let mut outport = Port::<u8>::new(io_base as u16 + 0x37);

        println!("Resetting...");

        unsafe {
            outport.write(0x10);
            let mut out = outport.read();
            while (out & 0x10) != 0 {
                if out != 0xFF {
                    println!("Error: {:#X}", out);
                }
                out = outport.read();
            }
        }

        println!("Reset complete");

        let mut outport = Port::<u32>::new(io_base as u16 + 0x30);

        unsafe {
            outport.write(
                memory::translate_addr(VirtAddr::new(BUFFER.as_ptr() as u64), phys_mem_offset)
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

        let mut outport = Port::<u16>::new(io_base as u16 + 0x3E);

        unsafe {
            outport.write(interrupts::InterruptIndex::RTL8139.as_usize() as u16);
        }

        let mut out = Rtl8139 {
            bar: bar.clone(),
            eeprom_exists: false,
            mac: [0; 6],
            tx_cur: 0,
        };

        out.set_mac_address();

        Ok(out)
    }

    pub fn set_mac_address(&mut self) {
        self.mac = self.get_mac_address();
    }

    fn get_mac_address(&self) -> [u8; 6] {
        let mut mac = [0; 6];
        let mut mac_part1_port = Port::<u32>::new(self.bar.address as u16 + 0);
        let mut mac_part2_port = Port::<u16>::new(self.bar.address as u16 + 4);

        let mac_part1 = unsafe { mac_part1_port.read() };
        let mac_part2 = unsafe { mac_part2_port.read() };

        mac[0] = (mac_part1 >> 0) as u8;
        mac[1] = (mac_part1 >> 8) as u8;
        mac[2] = (mac_part1 >> 16) as u8;
        mac[3] = (mac_part1 >> 24) as u8;
        mac[4] = (mac_part2 >> 0) as u8;
        mac[5] = (mac_part2 >> 8) as u8;

        mac
    }

    pub fn send_packet(&mut self, packet: &[u8]) {
        let mut outport = Port::<u32>::new(self.bar.address as u16 + 0x20);

        unsafe {
            outport.write(self.tx_cur);
        }

        let mut tx_cur = self.tx_cur as usize;

        for byte in packet {
            unsafe {
                BUFFER[tx_cur] = *byte;
            }
            tx_cur += 1;
        }

        self.tx_cur = (tx_cur as u32) + 4;

        let mut outport = Port::<u32>::new(self.bar.address as u16 + 0x10);

        unsafe {
            outport.write(0x1);
        }
    }
}

pub extern "x86-interrupt" fn rtl8139_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut outport = Port::<u32>::new(0x20);
    println!("RTL8139 interrupt");

    unsafe {
        outport.write(0x20);
    }
}
