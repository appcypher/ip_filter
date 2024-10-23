// main.rs - Test program
use std::net::Ipv4Addr;
use std::os::unix::io::RawFd;
use libc::{socket, connect, sockaddr_in, AF_INET, SOCK_STREAM};
use std::mem;
use std::str::FromStr;

fn main() {
    println!("Starting network filter test program");

    let addresses = [
        ("192.168.1.1", 80),
        ("8.8.8.8", 53),
        ("1.1.1.1", 53),
    ];

    for (ip, port) in addresses {
        println!("Attempting to connect to {}:{}", ip, port);

        unsafe {
            let sock: RawFd = socket(AF_INET, SOCK_STREAM, 0);
            if sock == -1 {
                println!("Failed to create socket for {}:{}", ip, port);
                continue;
            }

            let mut addr: sockaddr_in = std::mem::zeroed();
            addr.sin_len = std::mem::size_of::<sockaddr_in>() as u8;
            addr.sin_family = AF_INET as u8;
            addr.sin_port = (port as u16).to_be();
            addr.sin_addr = libc::in_addr {
                s_addr: u32::from(Ipv4Addr::from_str(ip).unwrap()).to_be()
            };

            let res = connect(sock, &addr as *const _ as *const libc::sockaddr, mem::size_of_val(&addr) as u32);
            if res == -1 {
                println!("Failed to connect to {}:{}: {}", ip, port, std::io::Error::last_os_error());
            } else {
                println!("Connected to {}:{}", ip, port);
            }

            libc::close(sock);
        }
    }

    println!("Test program completed");
}
