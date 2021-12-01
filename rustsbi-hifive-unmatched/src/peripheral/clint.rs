#[derive(Clone, Copy)]
pub struct Clint {
    base: *mut u8,
}

unsafe impl Send for Clint {}
unsafe impl Sync for Clint {}

#[allow(unused)]
impl Clint {
    pub fn new(base: *mut u8) -> Clint {
        Clint { base }
    }

    pub fn get_mtime(&self) -> u64 {
        unsafe {
            core::ptr::read_volatile(self.base.offset(0xbff8) as *mut u64)
        }
    }

    pub fn set_timer(&self, hart_id: usize, instant: u64) {
        unsafe {
            core::ptr::write_volatile((self.base.offset(0x4000) as *mut u64).add(hart_id), instant);
        }
    }

    pub fn send_soft(&self, hart_id: usize) {
        unsafe {
            core::ptr::write_volatile((self.base as *mut u32).add(hart_id), 1);
        }
    }

    pub fn clear_soft(&self, hart_id: usize) {
        unsafe {
            core::ptr::write_volatile((self.base as *mut u32).add(hart_id), 0);
        }
    }
}

impl rustsbi::Ipi for Clint {
    fn max_hart_id(&self) -> usize {
        4
    }

    fn send_ipi_many(&self, hart_mask: rustsbi::HartMask) -> rustsbi::SbiRet {
        for i in 0..=self.max_hart_id() {
            if hart_mask.has_bit(i) {
                self.send_soft(i);
            }
        }
        rustsbi::SbiRet::ok(0)
    }
}

impl rustsbi::Timer for Clint {
    fn set_timer(&self, time_value: u64) {
        let this_mhartid = riscv::register::mhartid::read();
        self.set_timer(this_mhartid, time_value);
    }
}
