use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::gpu::common::{Cache, GpuInfo, GpuVendor, Memory, MemoryType, Topology};

/// Detect NVIDIA GPUs using nvidia-smi
pub fn detect_nvidia_gpus() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();
    
    // Check if nvidia-smi is available
    if !is_nvidia_smi_available() {
        return Ok(vec![]);
    }
    
    // Run nvidia-smi to get GPU info
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,driver_version,memory.total,pci.bus_id,pstate,clocks.max.gr,clocks.current.gr", "--format=csv,noheader"])
        .output()
        .context("Failed to execute nvidia-smi")?;
    
    if !output.status.success() {
        return Err(anyhow!("nvidia-smi command failed"));
    }
    
    let output_str = String::from_utf8(output.stdout)
        .context("nvidia-smi output is not valid UTF-8")?;
    
    // Parse each GPU line
    for line in output_str.lines() {
        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if fields.len() < 7 {
            continue;
        }
        
        let name = fields[0];
        let driver_version = fields[1];
        let memory_total = fields[2];
        let pci_bus_id = fields[3];
        let performance_state = fields[4];
        let max_clock = fields[5];
        let current_clock = fields[6];
        
        // Create GPU info
        let mut gpu_info = GpuInfo::new(name, GpuVendor::Nvidia);
        gpu_info.driver_version = Some(driver_version.to_string());
        
        // Parse memory
        if let Some(memory_mb) = parse_nvidia_memory(memory_total) {
            let memory = Memory {
                size_bytes: memory_mb * 1024 * 1024,
                memory_type: get_nvidia_memory_type(name),
                bus_width: get_nvidia_bus_width(name),
                clock_mhz: 0, // To be populated later
            };
            gpu_info.memory = Some(memory);
        }
        
        // Parse clocks
        if let Ok(current_mhz) = parse_nvidia_clock(current_clock) {
            gpu_info.freq_mhz = current_mhz;
        }
        
        if let Ok(max_mhz) = parse_nvidia_clock(max_clock) {
            gpu_info.max_freq_mhz = max_mhz;
        }
        
        // Try to get architecture and compute capability
        if let Some((arch, compute_cap)) = get_nvidia_architecture(name) {
            gpu_info.architecture = arch;
            gpu_info.compute_capability = Some(compute_cap);
        }
        
        // Try to get chip info
        if let Some(chip) = get_nvidia_chip(name) {
            gpu_info.chip = chip;
        }
        
        // Try to get manufacturing process
        if let Some(process) = get_nvidia_process_nm(name) {
            gpu_info.process_nm = Some(process);
        }
        
        // Try to get topology information
        if let Some(topology) = get_nvidia_topology(name) {
            gpu_info.topology = Some(topology);
        }
        
        // Try to get cache information
        if let Some(cache) = get_nvidia_cache(name) {
            gpu_info.cache = Some(cache);
        }
        
        // Calculate peak performance
        if let Some(ref topology) = gpu_info.topology {
            if let Some(cuda_cores) = topology.cuda_cores {
                // Peak FLOPS = 2 * cores * clock
                let peak_gflops = 2.0 * cuda_cores as f64 * gpu_info.max_freq_mhz as f64 / 1000.0;
                gpu_info.peak_performance_gflops = Some(peak_gflops);
            }
        }
        
        gpus.push(gpu_info);
    }
    
    Ok(gpus)
}

/// Check if nvidia-smi is available
fn is_nvidia_smi_available() -> bool {
    Command::new("which")
        .arg("nvidia-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Parse memory total from nvidia-smi output
fn parse_nvidia_memory(memory_str: &str) -> Option<u64> {
    let re = Regex::new(r"(\d+) MiB").ok()?;
    re.captures(memory_str)
        .and_then(|cap| cap[1].parse::<u64>().ok())
}

/// Parse clock value from nvidia-smi output
fn parse_nvidia_clock(clock_str: &str) -> Result<u32> {
    let re = Regex::new(r"(\d+) MHz").context("Invalid regex")?;
    let cap = re.captures(clock_str)
        .ok_or_else(|| anyhow!("Invalid clock format"))?;
    
    cap[1].parse::<u32>()
        .context("Invalid clock value")
}

/// Determine memory type based on GPU name
fn get_nvidia_memory_type(name: &str) -> MemoryType {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 30") || name_lower.contains("rtx 40") {
        MemoryType::Gddr6X
    } else if name_lower.contains("rtx 20") || name_lower.contains("gtx 16") || name_lower.contains("rtx a") {
        MemoryType::Gddr6
    } else if name_lower.contains("titan v") || name_lower.contains("tesla v") {
        MemoryType::Hbm2
    } else if name_lower.contains("titan x") || name_lower.contains("gtx 1080") {
        MemoryType::Gddr5X
    } else if name_lower.contains("gtx 9") || name_lower.contains("gtx 10") {
        MemoryType::Gddr5
    } else {
        MemoryType::Unknown
    }
}

/// Estimate memory bus width based on GPU name
fn get_nvidia_bus_width(name: &str) -> u32 {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 3090") || name_lower.contains("rtx 3080") || 
       name_lower.contains("rtx 4090") || name_lower.contains("rtx 4080") {
        384
    } else if name_lower.contains("rtx 3070") || name_lower.contains("rtx 3060 ti") ||
              name_lower.contains("rtx 4070") {
        256
    } else if name_lower.contains("rtx 3060") || name_lower.contains("rtx 4060") {
        192
    } else if name_lower.contains("rtx 3050") || name_lower.contains("rtx 4050") {
        128
    } else if name_lower.contains("rtx 2080") {
        256
    } else if name_lower.contains("rtx 2070") || name_lower.contains("rtx 2060") {
        192
    } else if name_lower.contains("gtx 1080") || name_lower.contains("gtx 1070") {
        256
    } else if name_lower.contains("gtx 1060") {
        192
    } else if name_lower.contains("gtx 1050") {
        128
    } else {
        256  // Default value
    }
}

/// Determine NVIDIA architecture and compute capability
fn get_nvidia_architecture(name: &str) -> Option<(String, String)> {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 40") {
        Some(("Ada Lovelace".to_string(), "8.9".to_string()))
    } else if name_lower.contains("rtx 30") {
        Some(("Ampere".to_string(), "8.6".to_string()))
    } else if name_lower.contains("a100") || name_lower.contains("a30") {
        Some(("Ampere".to_string(), "8.0".to_string()))
    } else if name_lower.contains("rtx 20") || name_lower.contains("gtx 16") {
        Some(("Turing".to_string(), "7.5".to_string()))
    } else if name_lower.contains("tesla v") || name_lower.contains("titan v") {
        Some(("Volta".to_string(), "7.0".to_string()))
    } else if name_lower.contains("gtx 10") || name_lower.contains("tesla p") {
        Some(("Pascal".to_string(), "6.1".to_string()))
    } else if name_lower.contains("tesla p100") {
        Some(("Pascal".to_string(), "6.0".to_string()))
    } else if name_lower.contains("gtx 9") || name_lower.contains("tesla m40") {
        Some(("Maxwell".to_string(), "5.2".to_string()))
    } else if name_lower.contains("gtx 750") || name_lower.contains("gtx 860m") {
        Some(("Maxwell".to_string(), "5.0".to_string()))
    } else if name_lower.contains("gtx 780") || name_lower.contains("tesla k") {
        Some(("Kepler".to_string(), "3.5".to_string()))
    } else {
        None
    }
}

/// Determine NVIDIA chip name
fn get_nvidia_chip(name: &str) -> Option<String> {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 4090") || name_lower.contains("rtx 4080") {
        Some("AD102".to_string())
    } else if name_lower.contains("rtx 4070") {
        Some("AD104".to_string())
    } else if name_lower.contains("rtx 4060") {
        Some("AD106".to_string())
    } else if name_lower.contains("rtx 4050") {
        Some("AD107".to_string())
    } else if name_lower.contains("rtx 3090") || name_lower.contains("rtx 3080") {
        Some("GA102".to_string())
    } else if name_lower.contains("rtx 3070") || name_lower.contains("rtx 3060 ti") {
        Some("GA104".to_string())
    } else if name_lower.contains("rtx 3060") {
        Some("GA106".to_string())
    } else if name_lower.contains("rtx 3050") {
        Some("GA107".to_string())
    } else if name_lower.contains("a100") {
        Some("GA100".to_string())
    } else if name_lower.contains("rtx 2080") {
        Some("TU102".to_string())
    } else if name_lower.contains("rtx 2070") || name_lower.contains("rtx 2060") {
        Some("TU106".to_string())
    } else if name_lower.contains("gtx 1660") || name_lower.contains("gtx 1650") {
        Some("TU116".to_string())
    } else {
        None
    }
}

/// Determine manufacturing process based on architecture
fn get_nvidia_process_nm(name: &str) -> Option<u32> {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 40") {
        Some(4)  // 4nm for Ada Lovelace
    } else if name_lower.contains("rtx 30") || name_lower.contains("a100") {
        Some(8)  // 8nm for Ampere consumer GPUs
    } else if name_lower.contains("rtx 20") || name_lower.contains("gtx 16") {
        Some(12) // 12nm for Turing
    } else if name_lower.contains("titan v") || name_lower.contains("tesla v") {
        Some(12) // 12nm for Volta
    } else if name_lower.contains("gtx 10") || name_lower.contains("tesla p") {
        Some(16) // 16nm for Pascal
    } else if name_lower.contains("gtx 9") {
        Some(28) // 28nm for Maxwell Gen 2
    } else if name_lower.contains("gtx 750") || name_lower.contains("gtx 860m") {
        Some(28) // 28nm for Maxwell Gen 1
    } else {
        None
    }
}

/// Get topology information for NVIDIA GPUs
fn get_nvidia_topology(name: &str) -> Option<Topology> {
    let name_lower = name.to_lowercase();
    
    let (sm_count, cores_per_sm) = if name_lower.contains("rtx 4090") {
        (128, 128)  // Ada Lovelace, 16,384 CUDA cores
    } else if name_lower.contains("rtx 4080") {
        (76, 128)   // Ada Lovelace, 9,728 CUDA cores
    } else if name_lower.contains("rtx 4070 ti") {
        (60, 128)   // Ada Lovelace, 7,680 CUDA cores
    } else if name_lower.contains("rtx 4070") {
        (46, 128)   // Ada Lovelace, 5,888 CUDA cores
    } else if name_lower.contains("rtx 4060 ti") {
        (34, 128)   // Ada Lovelace, 4,352 CUDA cores
    } else if name_lower.contains("rtx 3090") {
        (82, 128)   // Ampere, 10,496 CUDA cores
    } else if name_lower.contains("rtx 3080") {
        (68, 128)   // Ampere, 8,704 CUDA cores
    } else if name_lower.contains("rtx 3070") {
        (46, 128)   // Ampere, 5,888 CUDA cores
    } else if name_lower.contains("rtx 3060 ti") {
        (38, 128)   // Ampere, 4,864 CUDA cores
    } else if name_lower.contains("rtx 2080 ti") {
        (68, 64)    // Turing, 4,352 CUDA cores
    } else if name_lower.contains("rtx 2080") {
        (46, 64)    // Turing, 2,944 CUDA cores
    } else if name_lower.contains("rtx 2070") {
        (36, 64)    // Turing, 2,304 CUDA cores
    } else if name_lower.contains("rtx 2060") {
        (30, 64)    // Turing, 1,920 CUDA cores
    } else if name_lower.contains("gtx 1080 ti") {
        (28, 128)   // Pascal, 3,584 CUDA cores
    } else if name_lower.contains("gtx 1080") {
        (20, 128)   // Pascal, 2,560 CUDA cores
    } else if name_lower.contains("gtx 1070") {
        (15, 128)   // Pascal, 1,920 CUDA cores
    } else if name_lower.contains("gtx 1060") {
        (10, 128)   // Pascal, 1,280 CUDA cores
    } else {
        return None;
    };
    
    let cuda_cores = sm_count * cores_per_sm;
    
    // Tensor cores: 0 for Pascal and earlier, 8 per SM for Turing, 4 per SM for Ampere
    let tensor_cores = if name_lower.contains("rtx 40") || name_lower.contains("rtx 30") {
        Some(sm_count * 4)
    } else if name_lower.contains("rtx 20") {
        Some(sm_count * 8)
    } else {
        None
    };
    
    // RT cores: 0 for Pascal and earlier, 1 per SM for Turing and Ampere
    let rt_cores = if name_lower.contains("rtx") {
        Some(sm_count)
    } else {
        None
    };
    
    Some(Topology {
        compute_units: sm_count,
        cuda_cores: Some(cuda_cores),
        tensor_cores,
        rt_cores,
        sm_count: Some(sm_count),
        stream_processors: None,
        rops: None,
        tmus: None,
        execution_units: None,
        slices: None,
        subslices: None,
    })
}

/// Get cache information for NVIDIA GPUs
fn get_nvidia_cache(name: &str) -> Option<Cache> {
    let name_lower = name.to_lowercase();
    
    // L2 cache sizes vary widely by GPU model
    let l2_size = if name_lower.contains("rtx 4090") {
        Some(72 * 1024 * 1024)  // 72 MB for RTX 4090
    } else if name_lower.contains("rtx 4080") {
        Some(64 * 1024 * 1024)  // 64 MB for RTX 4080
    } else if name_lower.contains("rtx 4070") {
        Some(48 * 1024 * 1024)  // 48 MB for RTX 4070
    } else if name_lower.contains("rtx 4060") {
        Some(32 * 1024 * 1024)  // 32 MB for RTX 4060
    } else if name_lower.contains("rtx 3090") {
        Some(6 * 1024 * 1024)   // 6 MB for RTX 3090
    } else if name_lower.contains("rtx 3080") {
        Some(5 * 1024 * 1024)   // 5 MB for RTX 3080
    } else if name_lower.contains("rtx 3070") {
        Some(4 * 1024 * 1024)   // 4 MB for RTX 3070
    } else if name_lower.contains("rtx 3060") {
        Some(3 * 1024 * 1024)   // 3 MB for RTX 3060
    } else if name_lower.contains("rtx 2080 ti") {
        Some(6 * 1024 * 1024)   // 6 MB for RTX 2080 Ti
    } else if name_lower.contains("rtx 2080") {
        Some(4 * 1024 * 1024)   // 4 MB for RTX 2080
    } else if name_lower.contains("rtx 2070") {
        Some(4 * 1024 * 1024)   // 4 MB for RTX 2070
    } else if name_lower.contains("rtx 2060") {
        Some(3 * 1024 * 1024)   // 3 MB for RTX 2060
    } else if name_lower.contains("gtx 1080 ti") {
        Some(3 * 1024 * 1024)   // 3 MB for GTX 1080 Ti
    } else if name_lower.contains("gtx 1080") {
        Some(2 * 1024 * 1024)   // 2 MB for GTX 1080
    } else if name_lower.contains("gtx 1070") {
        Some(2 * 1024 * 1024)   // 2 MB for GTX 1070
    } else if name_lower.contains("gtx 1060") {
        Some(1536 * 1024)       // 1.5 MB for GTX 1060
    } else {
        None
    };
    
    Some(Cache {
        l1_size: None, // NVIDIA doesn't typically publish L1 cache sizes
        l2_size,
        l3_size: None, // No L3 cache on most NVIDIA GPUs
    })
}
