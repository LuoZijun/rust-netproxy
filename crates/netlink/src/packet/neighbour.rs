use super::align;
use super::RouteType;

use byteorder::{ByteOrder, NativeEndian, NetworkEndian};

use std::io;
use core::ops::Range;
use std::convert::TryFrom;


// Neighbor Setup Service Module
// 
// 0                   1                   2                   3
// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |   Family    |    Reserved1  |           Reserved2           |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                     Interface Index                         |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |           State             |     Flags     |     Type      |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 
// Family: 8 bits
// Address Family: AF_INET for IPv4; and AF_INET6 for IPV6.
// 
// Interface Index: 32 bits
// The unique interface index.
// 
// State: 16 bits
// A bitmask of the following states:
//              NUD_INCOMPLETE   Still attempting to resolve.
//              NUD_REACHABLE    A confirmed working cache entry
//              NUD_STALE        an expired cache entry.
//              NUD_DELAY        Neighbor no longer reachable.
//                               Traffic sent, waiting for
//                               confirmation.
//              NUD_PROBE        A cache entry that is currently
//                               being re-solicited.
//              NUD_FAILED       An invalid cache entry.
//              NUD_NOARP        A device which does not do neighbor
//                               discovery (ARP).
//              NUD_PERMANENT    A static entry.
// Flags: 8 bits
//              NTF_PROXY        A proxy ARP entry.
//              NTF_ROUTER       An IPv6 router.
// 
// Attributes applicable to this service:
//              Attributes      Description
//              ------------------------------------
//              NDA_UNSPEC      Unknown type.
//              NDA_DST         A neighbour cache network.
//                              layer destination address
//              NDA_LLADDR      A neighbor cache link layer
//                              address.
//              NDA_CACHEINFO   Cache statistics.
// 
// Additional Netlink message types applicable to this service:
// RTM_NEWNEIGH, RTM_DELNEIGH, and RTM_GETNEIGH.
// 

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/neighbour.h


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct MacAddr(pub [u8; 6]);


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct AddressFamily(pub u8);

impl AddressFamily {
    pub const V4: Self = Self(2);
    pub const V6: Self = Self(10);
}

impl std::fmt::Debug for AddressFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::V4 => write!(f, "AF_INET"),
            Self::V6 => write!(f, "AF_INET6"),
            _ => write!(f, "AF_UNKNOW({})", self.0),
        }
    }
}

impl std::fmt::Display for AddressFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

bitflags! {
    pub struct NeighbourState: u16 {
        const INCOMPLETE =  0x1; // Still attempting to resolve.
        const REACHABLE  =  0x2; // A confirmed working cache entry.
        const STALE      =  0x4; // an expired cache entry.
        const DELAY      =  0x8; // Neighbor no longer reachable.
                                 // Traffic sent, waiting for confirmation.
        const PROBE      = 0x10; // A cache entry that is currently
                                 // being re-solicited.
        const FAILED     = 0x20; // An invalid cache entry.
        // Dummy states
        const NOARP      = 0x40; // A device that does not do neighbour discovery
        const PERMANENT  = 0x80; // Permanently set entries
        const NONE       = 0x00;
    }
}

// neighbour flags
bitflags! {
    pub struct NeighbourFlags: u8 {
        const USE         =  0x1;
        const SELF        =  0x2;
        const MASTER      =  0x4;
        const PROXY       =  0x8;
        const EXT_LEARNED = 0x10;
        const OFFLOADED   = 0x20;
        const ROUTER      = 0x80;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NeighbourAttrType(pub u16);

impl NeighbourAttrType {
    pub const NDA_UNSPEC: Self       = Self(0);
    pub const NDA_DST: Self          = Self(1);
    pub const NDA_LLADDR: Self       = Self(2);
    pub const NDA_CACHEINFO: Self    = Self(3);
    pub const NDA_PROBES: Self       = Self(4);
    pub const NDA_VLAN: Self         = Self(5);
    pub const NDA_PORT: Self         = Self(6);
    pub const NDA_VNI: Self          = Self(7);
    pub const NDA_IFINDEX: Self      = Self(8);
    pub const NDA_MASTER: Self       = Self(9);
    pub const NDA_LINK_NETNSID: Self = Self(10);
    pub const NDA_SRC_VNI: Self      = Self(11);
    pub const NDA_PROTOCOL: Self     = Self(12); // Originator of entry
}

impl std::fmt::Debug for NeighbourAttrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::NDA_UNSPEC => write!(f, "NDA_UNSPEC"),
            Self::NDA_DST => write!(f, "NDA_DST"),
            Self::NDA_LLADDR => write!(f, "NDA_LLADDR"),
            Self::NDA_CACHEINFO => write!(f, "NDA_CACHEINFO"),
            Self::NDA_PROBES => write!(f, "NDA_PROBES"),
            Self::NDA_VLAN => write!(f, "NDA_VLAN"),
            Self::NDA_PORT => write!(f, "NDA_PORT"),
            Self::NDA_VNI => write!(f, "NDA_VNI"),
            Self::NDA_IFINDEX => write!(f, "NDA_IFINDEX"),
            Self::NDA_MASTER => write!(f, "NDA_MASTER"),
            Self::NDA_LINK_NETNSID => write!(f, "NDA_LINK_NETNSID"),
            Self::NDA_SRC_VNI => write!(f, "NDA_SRC_VNI"),
            Self::NDA_PROTOCOL => write!(f, "NDA_PROTOCOL"),
            _ => write!(f, "NDA_UNKNOW({})", self.0),
        }
    }
}

impl std::fmt::Display for NeighbourAttrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}



// struct nda_cacheinfo {
//     __u32       ndm_confirmed;
//     __u32       ndm_used;
//     __u32       ndm_updated;
//     __u32       ndm_refcnt;
// };

const FAMILY: usize         = 0;
const IFINDEX: Range<usize> = 4..8;
const STATE:   Range<usize> = 8..10;
const FLAGS: usize          = 10;
const KIND: usize           = 11;
const PAYLOAD: usize        = 12;

// attrs
const SRC_ADDR_LEN: Range<usize> = 12..14;


#[derive(Debug, PartialEq, Clone)]
pub struct NeighbourPacket<T: AsRef<[u8]>> {
    buffer: T
}

impl<T: AsRef<[u8]>> NeighbourPacket<T> {
    pub const MIN_SIZE: usize = 12 + 4 + 4 + 4 + 6;

    #[inline]
    pub fn new_unchecked(buffer: T) -> NeighbourPacket<T> {
        NeighbourPacket { buffer }
    }

    #[inline]
    pub fn new_checked(buffer: T) -> Result<NeighbourPacket<T>, io::Error> {
        let v = Self::new_unchecked(buffer);
        v.check_len()?;

        Ok(v)
    }

    #[inline]
    pub fn check_len(&self) -> Result<(), io::Error> {
        let data = self.buffer.as_ref();
        if data.len() < Self::MIN_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "packet is too small."));
        }

        if data.len() < self.total_len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "packet is too small."));
        }

        Ok(())
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.buffer
    }

    #[inline]
    pub fn family(&self) -> AddressFamily {
        let data = self.buffer.as_ref();
        AddressFamily(data[FAMILY])
    }

    #[inline]
    pub fn ifindex(&self) -> i32 {
        let data = self.buffer.as_ref();
        NativeEndian::read_i32(&data[IFINDEX])
    }

    #[inline]
    pub fn state(&self) -> NeighbourState {
        let data = self.buffer.as_ref();
        NeighbourState::from_bits_truncate(NativeEndian::read_u16(&data[STATE]))
    }

    #[inline]
    pub fn flags(&self) -> NeighbourFlags {
        let data = self.buffer.as_ref();
        NeighbourFlags::from_bits_truncate(data[FLAGS])
    }

    #[inline]
    pub fn kind(&self) -> RouteType {
        let data = self.buffer.as_ref();
        RouteType(data[KIND])
    }

    // Attrs
    #[inline]
    fn dst_addr_len(&self) -> u16 {
        let data = self.buffer.as_ref();
        NativeEndian::read_u16(&data[SRC_ADDR_LEN])
    }

    #[inline]
    fn link_addr_start(&self) -> usize {
        let dst_len = align(self.dst_addr_len() as usize);
        PAYLOAD + dst_len
    }

    #[inline]
    fn link_addr_len(&self) -> u16 {
        let data = self.buffer.as_ref();
        let start = self.link_addr_start();
        let end = start + 2;

        NativeEndian::read_u16(&data[start..end])
    }

    #[inline]
    pub fn dst_addr(&self) -> std::net::IpAddr {
        let data = self.buffer.as_ref();
        let len = self.dst_addr_len() as usize;
        match len {
            8 => {
                assert_eq!(self.family(), AddressFamily::V4);
                // 12..14
                // 14..16
                // 16..20
                let octets = NetworkEndian::read_u32(&data[16..20]);
                std::net::Ipv4Addr::from(octets).into()
            },
            10 => {
                // MacAddr
                unreachable!();
            },
            20 => {
                // 12..14
                // 14..16
                // 16..20
                assert_eq!(self.family(), AddressFamily::V6);
                let octets = NetworkEndian::read_u128(&data[16..32]);
                std::net::Ipv6Addr::from(octets).into()
            },
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn link_addr(&self) -> MacAddr {
        let data = self.buffer.as_ref();
        let start = self.link_addr_start() + 4;

        // debug_assert!(self.link_addr_len() >= 10);

        MacAddr([
            data[start+0], data[start+1], data[start+2],
            data[start+3], data[start+4], data[start+5],
        ])
    }

    #[inline]
    pub fn header_len(&self) -> usize {
        12
    }

    // attr list len
    #[inline]
    pub fn payload_len(&self) -> usize {
        let data = self.buffer.as_ref();
        let data_len = data.len();

        let mut offset = PAYLOAD;
        loop {
            if offset >= data_len {
                break;
            }

            if data_len - offset < 8  {
                offset = data_len;
                break;
            }

            let attr_len = NativeEndian::read_u16(&data[offset..offset+2]);
            let len = align(attr_len as usize);
            offset += len;
        }

        offset - PAYLOAD
    }

    #[inline]
    pub fn total_len(&self) -> usize {
        self.header_len() + self.payload_len()
    }
}


impl<'a, T: AsRef<[u8]> + ?Sized> NeighbourPacket<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let data = self.buffer.as_ref();
        &data[PAYLOAD..]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> NeighbourPacket<T> {
    #[inline]
    pub fn set_family(&mut self, value: AddressFamily) {
        let data = self.buffer.as_mut();
        data[FAMILY] = value.0;
    }

    #[inline]
    pub fn set_ifindex(&mut self, value: i32) {
        let data = self.buffer.as_mut();
        NativeEndian::write_i32(&mut data[IFINDEX], value);
    }

    #[inline]
    pub fn set_state(&mut self, value: NeighbourState) {
        let data = self.buffer.as_mut();
        NativeEndian::write_u16(&mut data[STATE], value.bits());
    }

    #[inline]
    pub fn set_flags(&mut self, value: NeighbourFlags) {
        let data = self.buffer.as_mut();
        data[FLAGS] = value.bits();
    }

    #[inline]
    pub fn set_kind(&mut self, value: RouteType) {
        let data = self.buffer.as_mut();
        data[KIND] = value.0;
    }

    #[inline]
    pub fn set_dst_addr(&mut self, value: std::net::IpAddr) {
        match value {
            std::net::IpAddr::V4(v4_addr) => {
                self.set_family(AddressFamily::V4);

                let data = self.buffer.as_mut();
                let octets = u32::from(v4_addr).to_be_bytes();
                // 8, 0, 1, 0,
                data[PAYLOAD+0] = 8;
                data[PAYLOAD+1] = 0;
                data[PAYLOAD+2] = 1;
                data[PAYLOAD+3] = 0;

                data[PAYLOAD+4] = octets[0];
                data[PAYLOAD+5] = octets[1];
                data[PAYLOAD+6] = octets[2];
                data[PAYLOAD+7] = octets[3];
            },
            std::net::IpAddr::V6(v6_addr) => {
                self.set_family(AddressFamily::V6);

                let data = self.buffer.as_mut();
                let octets = u128::from(v6_addr).to_be_bytes();
                // 20, 0, 1, 0,
                data[PAYLOAD+0] = 20;
                data[PAYLOAD+1] = 0;
                data[PAYLOAD+2] = 1;
                data[PAYLOAD+3] = 0;

                let start = PAYLOAD + 4;
                let end = start + 16;
                &mut data[start..end].copy_from_slice(&octets);
            }
        }
    }

    #[inline]
    pub fn set_link_addr(&mut self, value: MacAddr) {
        let start = self.link_addr_start();

        let data = self.buffer.as_mut();

        // 10, 0, 2, 0,
        data[start+0] = 10;
        data[start+1] = 0;
        data[start+2] = 2;
        data[start+3] = 0;

        let start = start + 4;
        let end = start + 6;
        &mut data[start..end].copy_from_slice(&value.0);
    }

    #[inline]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let data = self.buffer.as_mut();
        &mut data[PAYLOAD..]
    }
}


impl std::fmt::Debug for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr = self.0;
        write!(f, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                    addr[0],
                    addr[1],
                    addr[2],
                    addr[3],
                    addr[4],
                    addr[5])
    }
}


impl<'a, T: AsRef<[u8]> + ?Sized> std::fmt::Display for NeighbourPacket<&'a T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NeighbourPacket {{ family: {:?}, ifindex: {}, state: {:?}, flags: {:?}, kind: {:?}, dst_addr: {:?}, link_addr: {:?} }}",
                self.family(),
                self.ifindex(),
                self.state(),
                self.flags(),
                self.kind(),
                self.dst_addr(),
                self.link_addr())
    }
}