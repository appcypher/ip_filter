// lib.rs
use errno::{set_errno, Errno};
use libc::{sockaddr, sockaddr_in, AF_INET, ECONNREFUSED};
use std::collections::HashSet;
use std::env;
use std::ffi::{c_char, CStr};
use std::os::raw::c_int;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Once,
};
use std::{net::Ipv4Addr, str::FromStr};

static INIT: Once = Once::new();
static mut BLOCKED_IPS: Option<HashSet<u32>> = None;
static LIBRARY_LOADED: AtomicBool = AtomicBool::new(false);

// Define the RTLD constants for macOS
#[allow(non_upper_case_globals)]
const RTLD_NEXT: *mut std::ffi::c_void = -1isize as *mut std::ffi::c_void;

#[link(name = "c")]
extern "C" {
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const c_char) -> *mut std::ffi::c_void;
}

fn initialize_blocked_ips() {
    INIT.call_once(|| unsafe {
        println!("Initializing blocked IPs");
        let mut blocked = HashSet::new();
        if let Ok(blocked_list) = env::var("BLOCKED_IPS") {
            println!("BLOCKED_IPS environment variable: {}", blocked_list);
            for ip_str in blocked_list.split(',') {
                if let Ok(ip) = Ipv4Addr::from_str(ip_str.trim()) {
                    blocked.insert(u32::from_be_bytes(ip.octets()));
                    println!("Added blocked IP: {}", ip);
                } else {
                    println!("Failed to parse IP: {}", ip_str);
                }
            }
        } else {
            println!("BLOCKED_IPS environment variable not set");
        }
        BLOCKED_IPS = Some(blocked);
        LIBRARY_LOADED.store(true, Ordering::SeqCst);
        println!("Network filter library loaded successfully");
        println!("Blocked IPs: {:?}", BLOCKED_IPS.as_ref().unwrap());
    });
}

#[no_mangle]
pub unsafe extern "C" fn connect(socket: c_int, address: *const sockaddr, len: c_int) -> c_int {
    initialize_blocked_ips();

    println!("Intercepted connect call");

    if !LIBRARY_LOADED.load(Ordering::SeqCst) {
        println!("Warning: Library not properly loaded!");
    }

    // Check if this is an IPv4 connection
    if !address.is_null() && (*address).sa_family as c_int == AF_INET {
        let addr_in = address as *const sockaddr_in;
        let ip_addr = (*addr_in).sin_addr.s_addr;

        // Print the IP being accessed
        let bytes = ip_addr.to_ne_bytes();
        println!(
            "Attempting to connect to IP: {}.{}.{}.{}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );

        // Check if this IP is blocked
        if let Some(ref blocked_ips) = BLOCKED_IPS {
            println!("Checking if IP is blocked. Blocked IPs: {:?}", blocked_ips);
            if blocked_ips.contains(&ip_addr) {
                println!(
                    "Blocking connection to IP: {}.{}.{}.{}",
                    bytes[0], bytes[1], bytes[2], bytes[3]
                );
                set_errno(Errno(ECONNREFUSED));
                return -1;
            } else {
                println!("IP is not blocked, allowing connection");
            }
        } else {
            println!("Warning: BLOCKED_IPS is None");
        }
    } else {
        println!("Not an IPv4 connection or null address");
    }

    let connect_sym = CStr::from_bytes_with_nul(b"connect\0").unwrap();
    let original_connect: Option<unsafe extern "C" fn(c_int, *const sockaddr, c_int) -> c_int> = {
        let sym = dlsym(RTLD_NEXT, connect_sym.as_ptr());
        if sym.is_null() {
            println!("Failed to find original connect symbol");
            None
        } else {
            Some(std::mem::transmute(sym))
        }
    };

    match original_connect {
        Some(func) => func(socket, address, len),
        None => {
            set_errno(Errno(libc::ENOSYS));
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, sock_type: c_int, protocol: c_int) -> c_int {
    println!("Intercepted socket call");

    if !LIBRARY_LOADED.load(Ordering::SeqCst) {
        println!("Warning: Library not properly loaded!");
    }

    let socket_sym = CStr::from_bytes_with_nul(b"socket\0").unwrap();
    let original_socket: Option<unsafe extern "C" fn(c_int, c_int, c_int) -> c_int> = {
        let sym = dlsym(RTLD_NEXT, socket_sym.as_ptr());
        if sym.is_null() {
            println!("Failed to find original socket symbol");
            None
        } else {
            Some(std::mem::transmute(sym))
        }
    };

    match original_socket {
        Some(func) => func(domain, sock_type, protocol),
        None => {
            set_errno(Errno(libc::ENOSYS));
            -1
        }
    }
}
