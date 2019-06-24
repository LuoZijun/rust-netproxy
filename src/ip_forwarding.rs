
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod sys {
    use sysctl;
    
    use std::io;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    const IPV4_KEY: &str = "net.inet.ip.forwarding";
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    const IPV6_KEY: &str = "net.inet6.ip.forwarding";

    #[cfg(any(target_os = "android", target_os = "linux"))]
    const IPV4_KEY: &str = "net.ipv4.ip_forward";
    #[cfg(any(target_os = "android", target_os = "linux"))]
    const IPV6_KEY: &str = "net.ipv6.ip_forward";

    #[inline]
    fn value_to_bool(value: sysctl::CtlValue) -> Result<bool, io::Error> {
        match value {
            sysctl::CtlValue::Int(n) => Ok(n == 1),
            sysctl::CtlValue::Uint(n) => Ok(n == 1),
            sysctl::CtlValue::Long(n) => Ok(n == 1),
            sysctl::CtlValue::Ulong(n) => Ok(n == 1),
            
            sysctl::CtlValue::U8(n) => Ok(n == 1),
            sysctl::CtlValue::U16(n) => Ok(n == 1),
            sysctl::CtlValue::U32(n) => Ok(n == 1),
            sysctl::CtlValue::U64(n) => Ok(n == 1),

            sysctl::CtlValue::S8(n) => Ok(n == 1),
            sysctl::CtlValue::S16(n) => Ok(n == 1),
            sysctl::CtlValue::S32(n) => Ok(n == 1),
            sysctl::CtlValue::S64(n) => Ok(n == 1),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    #[inline]
    pub fn ipv4_forwarding() -> Result<bool, io::Error> {
        sysctl::value(IPV4_KEY)
            .map(value_to_bool)
            .map_err(|_| io::Error::last_os_error())?
    }


    #[inline]
    pub fn enable_ipv4_forwarding() -> Result<bool, io::Error> {
        sysctl::set_value(IPV4_KEY, sysctl::CtlValue::Int(1))
            .map(value_to_bool)
            .map_err(|_| io::Error::last_os_error())?
    }

    #[inline]
    pub fn disable_ipv4_forwarding() -> Result<bool, io::Error> {
        sysctl::set_value(IPV4_KEY, sysctl::CtlValue::Int(0))
            .map(value_to_bool)
            .map_err(|_| io::Error::last_os_error())?
    }

    #[inline]
    pub fn ipv6_forwarding() -> Result<bool, io::Error> {
        sysctl::value(IPV6_KEY)
            .map(value_to_bool)
            .map_err(|_| io::Error::last_os_error())?
    }

    #[inline]
    pub fn enable_ipv6_forwarding() -> Result<bool, io::Error> {
        sysctl::set_value(IPV6_KEY, sysctl::CtlValue::Int(1))
            .map(value_to_bool)
            .map_err(|_| io::Error::last_os_error())?
    }

    #[inline]
    pub fn disable_ipv6_forwarding() -> Result<bool, io::Error> {
        sysctl::set_value(IPV6_KEY, sysctl::CtlValue::Int(0))
            .map(value_to_bool)
            .map_err(|_| io::Error::last_os_error())?
    }
}


#[cfg(any(target_os = "android", target_os = "linux"))]
mod sys {
    use libc::{ c_int, c_void, };
    
    use std::io;
    use std::ptr;

    pub const CTL_NET: c_int = 3;        // Networking
    
    pub const NET_IPV4: c_int = 5;
    pub const NET_IPV6: c_int = 12;

    pub const NET_IPV4_FORWARD: c_int = 8;
    pub const NET_IPV6_FORWARDING: c_int = 1;


    #[inline]
    pub fn ipv4_forwarding() -> Result<bool, io::Error> {
        let mut mib = [ CTL_NET, NET_IPV4, NET_IPV4_FORWARD, 0 ];
        let mib_ptr: *mut c_int = mib.as_mut_ptr();
        let mut ipforward: c_int = 0;
        let ipforward_ptr = &mut ipforward as *mut i32 as *mut c_void;
        let mut len = std::mem::size_of::<c_int>();
        
        if unsafe { libc::sysctl(mib_ptr, 4, ipforward_ptr, &mut len, ptr::null_mut(), 0) } < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(ipforward == 1)
    }


    #[inline]
    pub fn enable_ipv4_forwarding() -> Result<bool, io::Error> {
        let mut mib = [ CTL_NET, NET_IPV4, NET_IPV4_FORWARD, 0 ];
        let mib_ptr: *mut c_int = mib.as_mut_ptr();
        let mut ipforward: c_int = 1;
        let ipforward_ptr = &mut ipforward as *mut i32 as *mut c_void;
        let len = std::mem::size_of::<c_int>();
        
        if unsafe { libc::sysctl(mib_ptr, 4, ptr::null_mut(), &mut 0, ipforward_ptr, len) } < 0 {
            return Err(io::Error::last_os_error());
        }

        ipv4_forwarding()
    }

    #[inline]
    pub fn disable_ipv4_forwarding() -> Result<bool, io::Error> {
        let mut mib = [ CTL_NET, NET_IPV4, NET_IPV4_FORWARD, 0 ];
        let mib_ptr: *mut c_int = mib.as_mut_ptr();
        let mut ipforward: c_int = 0;
        let ipforward_ptr = &mut ipforward as *mut i32 as *mut c_void;
        let len = std::mem::size_of::<c_int>();
        
        if unsafe { libc::sysctl(mib_ptr, 4, ptr::null_mut(), &mut 0, ipforward_ptr, len) } < 0 {
            return Err(io::Error::last_os_error());
        }

        ipv4_forwarding()
    }

    #[inline]
    pub fn ipv6_forwarding() -> Result<bool, io::Error> {
        let mut mib = [ CTL_NET, NET_IPV6, NET_IPV6_FORWARDING, 0 ];
        let mib_ptr: *mut c_int = mib.as_mut_ptr();
        let mut ipforward: c_int = 0;
        let ipforward_ptr = &mut ipforward as *mut i32 as *mut c_void;
        let mut len = std::mem::size_of::<c_int>();
        
        if unsafe { libc::sysctl(mib_ptr, 4, ipforward_ptr, &mut len, ptr::null_mut(), 0) } < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(ipforward == 1)
    }

    #[inline]
    pub fn enable_ipv6_forwarding() -> Result<bool, io::Error> {
        let mut mib = [ CTL_NET, NET_IPV6, NET_IPV6_FORWARDING, 0 ];
        let mib_ptr: *mut c_int = mib.as_mut_ptr();
        let mut ipforward: c_int = 1;
        let ipforward_ptr = &mut ipforward as *mut i32 as *mut c_void;
        let len = std::mem::size_of::<c_int>();
        
        if unsafe { libc::sysctl(mib_ptr, 4, ptr::null_mut(), &mut 0, ipforward_ptr, len) } < 0 {
            return Err(io::Error::last_os_error());
        }

        ipv6_forwarding()
    }

    #[inline]
    pub fn disable_ipv6_forwarding() -> Result<bool, io::Error> {
        let mut mib = [ CTL_NET, NET_IPV6, NET_IPV6_FORWARDING, 0 ];
        let mib_ptr: *mut c_int = mib.as_mut_ptr();
        let mut ipforward: c_int = 0;
        let ipforward_ptr = &mut ipforward as *mut i32 as *mut c_void;
        let len = std::mem::size_of::<c_int>();
        
        if unsafe { libc::sysctl(mib_ptr, 4, ptr::null_mut(), &mut 0, ipforward_ptr, len) } < 0 {
            return Err(io::Error::last_os_error());
        }

        ipv6_forwarding()
    }
}

pub use self::sys::*;