use mdi::affinity::{CpuAffinity, ThreadBuilder};

#[test]
fn test_num_cpus() {
    let cpus = CpuAffinity::num_cpus();
    assert!(cpus > 0);
}

#[test]
fn test_affinity_config() {
    let config = CpuAffinity::get_thread_affinity_config(4);
    assert_eq!(config.len(), 4);
}

#[test]
fn test_thread_builder() {
    let handle = ThreadBuilder::new()
        .cpu(0)
        .name("test-thread".to_string())
        .spawn(|| {
            42
        });
    
    assert_eq!(handle.join().unwrap(), 42);
}
