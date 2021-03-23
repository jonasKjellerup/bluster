//! This module implements parts of the bluez Bluetooth Management API.
//! The documentation for the Bluetooth Management API can be found at:
//! https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc/mgmt-api.txt

use std::os::unix;
use std::os::unix::io::FromRawFd;
use std::os::raw::{c_ushort, c_int};
use std::io;

use tokio::net::UnixStream;

use libc;


const BTPROTO_HCI: c_int = 1;
const HCI_CHANNEL_CONTROL: c_ushort = 3;
const HCI_DEV_NONE: c_ushort = 65535;


/// Equivalent to the `sockaddr_hci` struct in C.
#[repr(C)]
struct HciSocketAddress {
    hci_family: c_ushort,
    hci_dev: c_ushort,
    hci_channel: c_ushort,
}

impl HciSocketAddress {
    const fn get_mgmt_address() -> Self {
        Self {
            hci_family: libc::AF_BLUETOOTH as c_ushort,
            hci_dev: HCI_DEV_NONE,
            hci_channel: HCI_CHANNEL_CONTROL,
        }
    }
}

struct ManagementSocket(UnixStream);

impl ManagementSocket {

    fn new() -> Result<Self, io::Error> {
        let fd = unsafe {
            libc::socket(
                libc::PF_BLUETOOTH,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                BTPROTO_HCI,
            )
        };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let addr = HciSocketAddress::get_mgmt_address();
        let r = unsafe {
            libc::bind(
                fd,
                &addr as *const HciSocketAddress as *const libc::sockaddr,
                std::mem::size_of::<HciSocketAddress>() as libc::socklen_t,
            )
        };

        if r < 0 {
            let err = io::Error::last_os_error();

            unsafe {libc::close(fd);}

            return Err(err);
        }

        let stream = unsafe {unix::net::UnixStream::from_raw_fd(fd)};
        let stream = UnixStream::from_std(stream)?;

        Ok(ManagementSocket(stream))
    }
}
