use std::ffi::c_void;
use std::net::Ipv4Addr;
use std::sync::OnceLock;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::IpHelper::{IcmpCreateFile, IcmpSendEcho, ICMP_ECHO_REPLY};

static ICMP_HANDLE: OnceLock<isize> = OnceLock::new();

fn get_icmp_handle() -> HANDLE {
    let val = *ICMP_HANDLE.get_or_init(|| {
        let handle = unsafe { IcmpCreateFile() };
        if let Ok(h) = handle {
            h.0 as isize // Win32 Foundation HANDLE in 0.52.0 is a struct with pub isize or *mut c_void. Let's just use transmute to be safe.
        } else {
            0
        }
    });
    // In windows 0.52.0, HANDLE is `pub struct HANDLE(pub isize)` or `*mut c_void`. Wait, it changed from isize to *mut c_void in some versions, and back. Let's just use transmute.
    if val == 0 {
        return HANDLE::default();
    }
    unsafe { std::mem::transmute(val) }
}

pub async fn get_ping(host: &str) -> f64 {
    let host_string = host.to_string();
    
    let rtt = tokio::task::spawn_blocking(move || {
        let handle = get_icmp_handle();
        // SinceHANDLE default is 0 or null
        // We will just proceed and if it fails, it returns 0.0

        let ip: Ipv4Addr = match host_string.parse() {
            Ok(ip) => ip,
            Err(_) => return 0.0,
        };

        // For Windows IcmpSendEcho, DestinationAddress is an IPAddr (u32 in network byte order usually, or it's system-dependent, but u32::from_ne_bytes(ip.octets()) works)
        let ip_u32 = u32::from_ne_bytes(ip.octets());
        
        let request_data = b"md_monitor_ping_data";
        let mut reply_buffer = vec![0u8; std::mem::size_of::<ICMP_ECHO_REPLY>() + request_data.len() + 8];
        let timeout_ms = 1000;

        let ret = unsafe {
            IcmpSendEcho(
                handle,
                ip_u32,
                request_data.as_ptr() as *const c_void,
                request_data.len() as u16,
                None,
                reply_buffer.as_mut_ptr() as *mut c_void,
                reply_buffer.len() as u32,
                timeout_ms,
            )
        };

        if ret == 0 {
            return 0.0;
        }

        let reply = unsafe { &*(reply_buffer.as_ptr() as *const ICMP_ECHO_REPLY) };
        if reply.Status == 0 {
            reply.RoundTripTime as f64
        } else {
            0.0
        }
    }).await.unwrap_or(0.0);
    
    rtt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_ping_invalid_ip() {
        let rtt = get_ping("invalid_host").await;
        assert_eq!(rtt, 0.0);
    }
}

