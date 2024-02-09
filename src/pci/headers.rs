use super::pci_config_read_word;

trait GetHeader {
    fn get_header(bus: u8, device: u8, function: u8) -> Self;
}

#[derive(Debug)]
pub struct Header {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
    pub rest_of_header: HeaderType,
}

impl Header {
    pub fn new(bus: u8, device: u8, function: u8) -> Result<Header, HeaderError> {
        let mut header = Header {
            vendor_id: 0,
            device_id: 0,
            command: 0,
            status: 0,
            revision_id: 0,
            prog_if: 0,
            subclass: 0,
            class: 0,
            cache_line_size: 0,
            latency_timer: 0,
            header_type: 0,
            bist: 0,
            rest_of_header: HeaderType::Standard(StandardHeader {
                base_address_registers: [0; 6],
                cardbus_cis_pointer: 0,
                subsystem_vendor_id: 0,
                subsystem_id: 0,
                expansion_rom_base_address: 0,
                capabilities_pointer: 0,
                interrupt_line: 0,
                interrupt_pin: 0,
                min_grant: 0,
                max_latency: 0,
            }),
        };
        header.vendor_id = pci_config_read_word(bus, device, function, 0);
        header.device_id = pci_config_read_word(bus, device, function, 2);
        header.command = pci_config_read_word(bus, device, function, 4);
        header.status = pci_config_read_word(bus, device, function, 6);
        (header.revision_id, header.prog_if) =
            pci_config_read_word(bus, device, function, 8).split();
        (header.subclass, header.class) = pci_config_read_word(bus, device, function, 0xA).split();
        (header.cache_line_size, header.latency_timer) =
            pci_config_read_word(bus, device, function, 0xC).split();
        (header.header_type, header.bist) =
            pci_config_read_word(bus, device, function, 0xE).split();

        header.rest_of_header = HeaderType::get(bus, device, function)?;
        Ok(header)
    }
}

#[derive(Debug)]
pub enum HeaderType {
    Standard(StandardHeader),
    PciToPciBridge(PciToPciBus),
    CardBusBridge(CardBusBridge),
}

impl HeaderType {
    fn get(bus: u8, device: u8, function: u8) -> Result<Self, HeaderError> {
        let header_type = pci_config_read_word(bus, device, function, 0xE);
        match header_type {
            0x00 => Ok(HeaderType::Standard(StandardHeader::get_header(
                bus, device, function,
            ))),
            0x01 => Err(HeaderError::UnimplementedHeaderType),
            0x02 => Err(HeaderError::UnimplementedHeaderType),
            _ => Err(HeaderError::InvalidHeaderType),
        }
    }
}

#[derive(Debug)]
pub struct StandardHeader {
    pub base_address_registers: [u32; 6],
    pub cardbus_cis_pointer: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_base_address: u32,
    pub capabilities_pointer: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

impl GetHeader for StandardHeader {
    fn get_header(bus: u8, device: u8, function: u8) -> Self {
        let mut header = StandardHeader {
            base_address_registers: [0; 6],
            cardbus_cis_pointer: 0,
            subsystem_vendor_id: 0,
            subsystem_id: 0,
            expansion_rom_base_address: 0,
            capabilities_pointer: 0,
            interrupt_line: 0,
            interrupt_pin: 0,
            min_grant: 0,
            max_latency: 0,
        };

        for i in 0..6 {
            let offset = 0x10 + i * 4;
            let low = pci_config_read_word(bus, device, function, offset);
            let high = pci_config_read_word(bus, device, function, offset + 2);
            header.base_address_registers[i as usize] = (high as u32) << 16 | low as u32;
        }

        header.cardbus_cis_pointer = pci_config_read_word(bus, device, function, 0x28) as u32;
        header.cardbus_cis_pointer |=
            (pci_config_read_word(bus, device, function, 0x2A) as u32) << 16;
        header.subsystem_vendor_id = pci_config_read_word(bus, device, function, 0x2C);
        header.subsystem_id = pci_config_read_word(bus, device, function, 0x2E);
        header.expansion_rom_base_address =
            pci_config_read_word(bus, device, function, 0x30) as u32;
        header.expansion_rom_base_address |=
            (pci_config_read_word(bus, device, function, 0x32) as u32) << 16;
        header.capabilities_pointer = pci_config_read_word(bus, device, function, 0x34) as u8;
        header.interrupt_line = pci_config_read_word(bus, device, function, 0x3C) as u8;
        header.interrupt_pin = pci_config_read_word(bus, device, function, 0x3D) as u8;
        header.min_grant = pci_config_read_word(bus, device, function, 0x3E) as u8;
        header.max_latency = pci_config_read_word(bus, device, function, 0x3F) as u8;

        header
    }
}

#[derive(Debug)]
pub struct PciToPciBus {
    pub base_address_registers: [u32; 2],
    pub primary_bus_number: u8,
    pub secondary_bus_number: u8,
    pub subordinate_bus_number: u8,
    pub secondary_latency_timer: u8,
    pub io_base: u8,
    pub io_limit: u8,
    pub secondary_status: u16,
    pub memory_base: u16,
    pub memory_limit: u16,
    pub prefetchable_memory_base: u16,
    pub prefetchable_memory_limit: u16,
    pub prefetchable_base_upper_32_bits: u32,
    pub prefetchable_limit_upper_32_bits: u32,
    pub io_base_upper_16_bits: u16,
    pub io_limit_upper_16_bits: u16,
    pub capabilities_pointer: u8,
    pub expansion_rom_base_address: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bridge_control: u16,
}

impl GetHeader for PciToPciBus {
    fn get_header(bus: u8, device: u8, function: u8) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct CardBusBridge {}

impl GetHeader for CardBusBridge {
    fn get_header(bus: u8, device: u8, function: u8) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum HeaderError {
    InvalidHeaderType,
    UnimplementedHeaderType,
}

trait SplitBits<T> {
    fn split(&self) -> (T, T);
}

impl SplitBits<u8> for u16 {
    fn split(&self) -> (u8, u8) {
        let low = *self as u8;
        let high = (*self >> 8) as u8;
        (high, low)
    }
}
