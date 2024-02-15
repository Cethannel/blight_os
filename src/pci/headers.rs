use super::pci_config_read_word;

#[derive(Debug)]
pub enum BAR {
    Memory(MemoryBar),
    IO(IOBar),
}

#[derive(Debug)]
pub struct MemoryBar {
    pub _type: u8,
    pub prefetchable: bool,
    pub address: u32,
}

#[derive(Debug, Clone)]
pub struct IOBar {
    pub address: u32,
}

impl BAR {
    pub fn new(bus: u8, device: u8, function: u8, index: u8) -> BAR {
        let low = pci_config_read_word(bus, device, function, 0x10 + index * 4);
        let high = pci_config_read_word(bus, device, function, 0x10 + index * 4 + 2);

        let address = (high as u32) << 16 | low as u32;

        if (low & 0x1) == 0 {
            BAR::Memory(MemoryBar {
                _type: (low & 0x6) as u8,
                prefetchable: (low & 0x8) != 0,
                address: address >> 4,
            })
        } else {
            BAR::IO(IOBar {
                address: address >> 2,
            })
        }
    }
}

trait GetHeader {
    fn get_header(bus: u8, device: u8, function: u8) -> Self;
}

#[derive(Debug)]
pub struct Header {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
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
            bus,
            device,
            function,
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
            rest_of_header: HeaderType::Unimplemented,
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
    Unimplemented,
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

    pub fn to_standard(&self) -> Result<&StandardHeader, HeaderError> {
        match self {
            HeaderType::Standard(header) => Ok(header),
            _ => Err(HeaderError::InvalidHeaderType),
        }
    }
}

#[derive(Debug)]
pub struct StandardHeader {
    pub base_address_registers: [BAR; 6],
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
        let bars = [
            BAR::new(bus, device, function, 0),
            BAR::new(bus, device, function, 1),
            BAR::new(bus, device, function, 2),
            BAR::new(bus, device, function, 3),
            BAR::new(bus, device, function, 4),
            BAR::new(bus, device, function, 5),
        ];

        let mut header = StandardHeader {
            base_address_registers: bars,
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
        let mut header = PciToPciBus {
            base_address_registers: [0; 2],
            primary_bus_number: 0,
            secondary_bus_number: 0,
            subordinate_bus_number: 0,
            secondary_latency_timer: 0,
            io_base: 0,
            io_limit: 0,
            secondary_status: 0,
            memory_base: 0,
            memory_limit: 0,
            prefetchable_memory_base: 0,
            prefetchable_memory_limit: 0,
            prefetchable_base_upper_32_bits: 0,
            prefetchable_limit_upper_32_bits: 0,
            io_base_upper_16_bits: 0,
            io_limit_upper_16_bits: 0,
            capabilities_pointer: 0,
            expansion_rom_base_address: 0,
            interrupt_line: 0,
            interrupt_pin: 0,
            bridge_control: 0,
        };

        for i in 0..2 {
            let offset = 0x10 + i * 4;
            let low = pci_config_read_word(bus, device, function, offset);
            let high = pci_config_read_word(bus, device, function, offset + 2);
            header.base_address_registers[i as usize] = (high as u32) << 16 | low as u32;
        }

        (header.primary_bus_number, header.secondary_bus_number) =
            pci_config_read_word(bus, device, function, 0x18).split();
        (
            header.subordinate_bus_number,
            header.secondary_latency_timer,
        ) = pci_config_read_word(bus, device, function, 0x1A).split();

        (header.io_base, header.io_limit) =
            pci_config_read_word(bus, device, function, 0x1C).split();
        header.secondary_status = pci_config_read_word(bus, device, function, 0x1E);

        header.memory_base = pci_config_read_word(bus, device, function, 0x20);

        header.memory_limit = pci_config_read_word(bus, device, function, 0x22);

        header.prefetchable_memory_base = pci_config_read_word(bus, device, function, 0x24);

        header.prefetchable_memory_limit = pci_config_read_word(bus, device, function, 0x26);

        header.prefetchable_base_upper_32_bits =
            pci_config_read_word(bus, device, function, 0x28) as u32;
        header.prefetchable_base_upper_32_bits |=
            (pci_config_read_word(bus, device, function, 0x2A) as u32) << 16;

        header.prefetchable_limit_upper_32_bits =
            pci_config_read_word(bus, device, function, 0x2C) as u32;
        header.prefetchable_limit_upper_32_bits |=
            (pci_config_read_word(bus, device, function, 0x2E) as u32) << 16;

        header.io_base_upper_16_bits = pci_config_read_word(bus, device, function, 0x30);

        header.io_limit_upper_16_bits = pci_config_read_word(bus, device, function, 0x32);

        header.capabilities_pointer = pci_config_read_word(bus, device, function, 0x34) as u8;

        header.expansion_rom_base_address =
            pci_config_read_word(bus, device, function, 0x38) as u32;
        header.expansion_rom_base_address |=
            (pci_config_read_word(bus, device, function, 0x3A) as u32) << 16;

        (header.interrupt_line, header.interrupt_pin) =
            pci_config_read_word(bus, device, function, 0x3C).split();

        header.bridge_control = pci_config_read_word(bus, device, function, 0x3E);

        header
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
        (low, high)
    }
}
