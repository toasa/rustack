mod eth;

use eth::EthernetFrame;
use std::{fs::OpenOptions, io::Read, os::fd::AsRawFd};

#[repr(C)]
struct Ifreq {
    ifr_name: [libc::c_char; 16],
    ifr_flags: libc::c_short,
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
