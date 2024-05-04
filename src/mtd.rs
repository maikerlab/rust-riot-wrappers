use embedded_storage::{ReadStorage, Storage};
use crate::error::{ENOMEM, NumericError};
use riot_sys::libc::c_void;
use riot_sys::inline::{flashpage_addr};
use riot_sys::{flashpage_erase, flashpage_write, flashpage_read};

pub struct FlashpageStorage {
    capacity: usize
}

impl FlashpageStorage {
    // TODO: For testing allow storing one page of 512 Bytes size
    pub fn new(&self) -> Self {
        Self { capacity: 512 }
    }
}

impl ReadStorage for FlashpageStorage {
    type Error = NumericError;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if bytes.len() > (self.capacity - offset as usize) {
            Err(ENOMEM)
        } else {
            unsafe {
                flashpage_read(offset, bytes.as_ptr() as *mut c_void);
            }
            Ok(())
        }
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Storage for FlashpageStorage {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        // TODO: as long as data fits within the page we just write it!
        if !(offset >= 0 && offset < self.capacity() as u32) {
            Err(ENOMEM)
        } else {
            unsafe {
                let addr = flashpage_addr(offset);
                flashpage_erase(offset);
                flashpage_write(addr, bytes.as_ptr() as *const c_void, bytes.len() as u32);
            }
            Ok(())
        }
    }
}
