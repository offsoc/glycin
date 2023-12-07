use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::os::fd::FromRawFd;
use std::os::fd::RawFd;

use zbus::zvariant;

use super::Texture;

#[derive(Debug)]
pub struct SharedMemory {
    memfd: RawFd,
    pub mmap: memmap::MmapMut,
}

impl SharedMemory {
    pub fn new(size: u64) -> Self {
        let memfd = nix::sys::memfd::memfd_create(
            &CString::new("glycin-frame").unwrap(),
            nix::sys::memfd::MemFdCreateFlag::MFD_CLOEXEC
                | nix::sys::memfd::MemFdCreateFlag::MFD_ALLOW_SEALING,
        )
        .expect("Failed to create memfd");
        nix::unistd::ftruncate(memfd, size.try_into().expect("Required memory too large"))
            .expect("Failed to set memfd size");
        let mmap = unsafe { memmap::MmapMut::map_mut(memfd) }.expect("Mailed to mmap memfd");

        Self { mmap, memfd }
    }

    pub fn into_texture(self) -> Texture {
        let owned_fd = unsafe { zvariant::OwnedFd::from_raw_fd(self.memfd) };
        Texture::MemFd(owned_fd)
    }
}

impl Deref for SharedMemory {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.mmap.deref()
    }
}

impl DerefMut for SharedMemory {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.mmap.deref_mut()
    }
}
