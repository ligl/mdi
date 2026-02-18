use crate::Result;

/// CPU Affinity 管理
/// 将线程绑定到特定的 CPU 核心，提高缓存局部性和减少上下文切换
pub struct CpuAffinity;

impl CpuAffinity {
    /// 获取系统 CPU 核心数
    pub fn num_cpus() -> usize {
        num_cpus::get()
    }

    /// 获取物理 CPU 数
    pub fn num_physical_cpus() -> usize {
        num_cpus::get_physical()
    }

    /// 将当前线程绑定到指定 CPU 核心
    /// 
    /// # Arguments
    /// * `cpu_id` - CPU 核心编号（0-based）
    pub fn bind_current_thread(cpu_id: usize) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::mem;
            use libc::{cpu_set_t, CPU_ZERO, CPU_SET, sched_setaffinity};

            unsafe {
                let mut cpu_set: cpu_set_t = mem::zeroed();
                CPU_ZERO(&mut cpu_set);
                CPU_SET(cpu_id, &mut cpu_set);

                let result = sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set);
                if result != 0 {
                    return Err(crate::MdiError::Other(
                        format!("Failed to set CPU affinity: {}", std::io::Error::last_os_error())
                    ));
                }
            }
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("CPU affinity not fully supported on this platform");
            Ok(())
        }
    }

    /// 创建线程并绑定到指定 CPU 核心
    pub fn spawn_with_affinity<F, T>(cpu_id: usize, f: F) -> std::thread::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(move || {
            let _ = Self::bind_current_thread(cpu_id);
            f()
        })
    }

    /// 获取推荐的线程绑定配置
    /// 返回 Vec 包含每个线程应该绑定的 CPU 核心号
    pub fn get_thread_affinity_config(num_threads: usize) -> Vec<usize> {
        let num_cpus = Self::num_cpus();
        
        if num_threads <= num_cpus {
            // 如果线程数小于 CPU 核心数，采用分散绑定
            (0..num_threads).collect()
        } else {
            // 如果线程数大于 CPU 核心数，采用循环绑定
            (0..num_threads).map(|i| i % num_cpus).collect()
        }
    }
}

/// Thread Builder with CPU Affinity
pub struct ThreadBuilder {
    cpu_id: Option<usize>,
    name: Option<String>,
}

impl ThreadBuilder {
    pub fn new() -> Self {
        ThreadBuilder {
            cpu_id: None,
            name: None,
        }
    }

    pub fn cpu(mut self, cpu_id: usize) -> Self {
        self.cpu_id = Some(cpu_id);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn spawn<F, T>(self, f: F) -> std::thread::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let mut builder = std::thread::Builder::new();
        if let Some(name) = self.name {
            builder = builder.name(name);
        }

        let cpu_id = self.cpu_id;
        
        builder
            .spawn(move || {
                if let Some(cpu_id) = cpu_id {
                    let _ = CpuAffinity::bind_current_thread(cpu_id);
                }
                f()
            })
            .expect("Failed to spawn thread")
    }
}
