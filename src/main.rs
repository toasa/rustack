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
            println!("受信: {} bytes", nread);
        }

        for chunk in buf[..nread].chunks(16) {
            for byte in chunk {
                print!("{:02x} ", byte);
            }
            println!();
        }
        println!("------------------------------------------------")
    }
}
