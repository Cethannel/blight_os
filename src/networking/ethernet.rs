pub struct Packet {
    pub sfd: u8,
    pub dest: [u8; 6],
    pub src: [u8; 6],
    pub data: [u8; 1500 - 46],
    pub len: usize,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            sfd: 0b10101011,
            dest: [0; 6],
            src: [0; 6],
            data: [0; 1500 - 46],
            len: 0,
        }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Packet, PacketError> {
        if slice.len() < 14 {
            return Err(PacketError::InvalidPacket);
        }

        for i in 0..7 {
            if slice[i] != 0b10101010 {
                return Err(PacketError::NoStart);
            }
        }

        let slice = &slice[8..];

        let mut out = Packet::new();
        out.sfd = 0b10101011;

        for i in 0..6 {
            out.dest[i] = slice[i];
        }

        for i in 0..6 {
            out.src[i] = slice[i + 6];
        }

        let len = slice[12] as usize * 256 + slice[13] as usize;

        let slice = &slice[14..];

        for i in 0..len {
            out.data[i] = slice[i];
        }

        Ok(out)
    }

    pub fn to_slice(&self) -> [u8; 1504] {
        let mut out = [0; 1504];

        for i in 0..7 {
            out[i] = 0b10101010;
        }

        out[7] = self.sfd;

        for i in 0..6 {
            out[i + 8] = self.dest[i];
        }

        for i in 0..6 {
            out[i + 14] = self.src[i];
        }

        out[14] = (self.len / 256) as u8;
        out[15] = (self.len % 256) as u8;

        for i in 0..self.len {
            out[i + 16] = self.data[i];
        }

        // TODO: CRC
        out[1500] = 0;
        out[1501] = 0;
        out[1502] = 0;
        out[1503] = 0;

        out
    }
}

#[derive(Debug)]
pub enum PacketError {
    NoStart,
    InvalidPacket,
}
