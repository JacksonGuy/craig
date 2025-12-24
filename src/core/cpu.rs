use psutil;

pub struct CPUData {
    collector: psutil::cpu::CpuPercentCollector,
    pub cpu_count: u64,
    pub cpu_cores: u64,
}

impl CPUData {
    pub fn new() -> Self {
        Self {
            collector: psutil::cpu::CpuPercentCollector::new().unwrap(),
            cpu_count: psutil::cpu::cpu_count(),
            cpu_cores: psutil::cpu::cpu_count_physical(),
        }
    }

    pub fn get_cpu_usage(&mut self) -> Vec<f32> {
        self.collector.cpu_percent_percpu().unwrap()
    }
}
