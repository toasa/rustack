use std::fmt;
use std::{fs::OpenOptions, io::Read, os::fd::AsRawFd};

#[repr(C)]
struct Ifreq {
    ifr_name: [libc::c_char; 16],
    ifr_flags: libc::c_short,
}

pub struct MacAddress(pub [u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

#[derive(Debug)]
pub enum EtherType {
    ARP,
    Unkown(u16),
}

impl EtherType {
    fn from_u16(val: u16) -> Self {
        match val {
            0x0806 => EtherType::ARP,
            other => EtherType::Unkown(other),
        }
    }
}

pub struct EthernetFrame<'a> {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ethertype: EtherType,
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self, &'static str> {
        if buf.len() < 14 {
            return Err("Frame too short");
        }

        let mut dst_bytes = [0u8; 6];
        dst_bytes.copy_from_slice(&buf[0..6]);

        let mut src_bytes = [0u8; 6];
        src_bytes.copy_from_slice(&buf[6..12]);

        let ethertype_raw = u16::from_be_bytes([buf[12], buf[13]]);

        Ok(EthernetFrame {
            dst_mac: MacAddress(dst_bytes),
            src_mac: MacAddress(src_bytes),
            ethertype: EtherType::from_u16(ethertype_raw),
            payload: &buf[14..],
        })
    }
}

fn main() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/net/tun")?;

    let mut ifr = Ifreq {
        ifr_name: [0; 16],
        ifr_flags: (libc::IFF_TAP | libc::IFF_NO_PI) as i16,
    };

    let name = "tap0";
    for (i, b) in name.as_bytes().iter().enumerate() {
        ifr.ifr_name[i] = *b as libc::c_char;
    }

    unsafe {
        let res = libc::ioctl(
            file.as_raw_fd(),
            libc::TUNSETIFF,
            &ifr as *const Ifreq as *mut libc::c_void,
        );
        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    println!("tap0 にアタッチしました！パケット待機中です...");

    let mut buf = [0u8; 1500];

    loop {
        let nread = file.read(&mut buf)?;
        if nread > 0 {
            match EthernetFrame::parse(&buf[..nread]) {
                Ok(frame) => {
                    println!("---------------------------------------");
                    println!("  Ethernet フレーム受信 ({} bytes)", nread);
                    println!("  Src MAC: {}", frame.src_mac);
                    println!("  Dst MAC: {}", frame.dst_mac);
                    println!("  EtherType : {:?}", frame.ethertype);
                    println!("  Payload : {} bytes", frame.payload.len());
                }
                Err(e) => {
                    eprintln!("parse error: {}", e);
                }
            }
        }
    }
}
