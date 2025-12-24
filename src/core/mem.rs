use psutil;
use size::{Size, Base};

pub struct MemData {}

impl MemData {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_percent(&self) -> f32 {
        let main = psutil::memory::virtual_memory().unwrap();
        main.percent()
    }

    pub fn get_total(&self) -> u64 {
        let main = psutil::memory::virtual_memory().unwrap();
        main.total()
    }

    pub fn get_used(&self) -> u64 {
        let main = psutil::memory::virtual_memory().unwrap();
        main.used()
    }

    pub fn bytes_to_string(&self, bytes: u64) -> String {
        let size = Size::from_bytes(bytes);
        size.format().with_base(Base::Base10).to_string()
    }
}
