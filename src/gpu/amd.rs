use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::gpu::common::{Cache, GpuInfo, GpuVendor, Memory, MemoryType, Topology};

/// Detect AMD GPUs
pub fn detect_amd_gpus() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();
    
    // Check for AMD GPUs in the system
    if let Ok(amd_gpu_paths) = find_amd_gpus_in_sysfs() {
        for path in amd_gpu_paths {
            if let Ok(mut gpu_info) = get_amd_gpu_info_from_sysfs(&path) {
                // Try to enhance info using rocm-smi if available
                if is_rocm_smi_available() {
                    if let Ok(()) = enhance_with_rocm_smi(&mut gpu_info) {
                        // Additional info added from rocm-smi
                    }
                }
                
                gpus.push(gpu_info);
            }
        }
    }
    
    Ok(gpus)
}

/// Find AMD GPU directories in sysfs
fn find_amd_gpus_in_sysfs() -> Result<Vec<PathBuf>> {
    let mut gpu_paths = Vec::new();
    
    // Try AMD card directory in /sys/class/drm
    let drm_path = Path::new("/sys/class/drm");
    if drm_path.exists() {
        for entry in fs::read_dir(drm_path).context("Failed to read DRM directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // Check for AMD GPUs (card directories with amdgpu driver)
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("card") && !name.contains("renderD") {
                    let device_path = path.join("device");
                    if device_path.exists() {
                        // Check if this is an AMD GPU
                        let vendor_path = device_path.join("vendor");
                        if let Ok(vendor) = fs::read_to_string(vendor_path) {
                            if vendor.trim() == "0x1002" {
                                gpu_paths.push(device_path);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(gpu_paths)
}

/// Extract AMD GPU information from sysfs
fn get_amd_gpu_info_from_sysfs(device_path: &Path) -> Result<GpuInfo> {
    // Read device ID
    let device_id_path = device_path.join("device");
    let device_id = fs::read_to_string(device_id_path)
        .map(|id| id.trim().trim_start_matches("0x").to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    // Read subsystem name (typically contains the full GPU model name)
    let product_name_path = device_path.join("product_name");
    let name = if product_name_path.exists() {
        fs::read_to_string(product_name_path)
            .map(|name| name.trim().to_string())
            .unwrap_or_else(|_| format!("AMD GPU (Device ID: {})", device_id))
    } else {
        // Try to get name from modalias
        let modalias_path = device_path.join("modalias");
        if modalias_path.exists() {
            if let Ok(modalias) = fs::read_to_string(modalias_path) {
                let re = Regex::new(r"pci:v00001002d0000([0-9A-Fa-f]{4})").ok();
                if let Some(re) = re {
                    if let Some(caps) = re.captures(&modalias) {
                        format!("AMD GPU (Device ID: {})", &caps[1])
                    } else {
                        format!("AMD GPU (Device ID: {})", device_id)
                    }
                } else {
                    format!("AMD GPU (Device ID: {})", device_id)
                }
            } else {
                format!("AMD GPU (Device ID: {})", device_id)
            }
        } else {
            format!("AMD GPU (Device ID: {})", device_id)
        }
    };
    
    // Create basic GPU info
    let mut gpu_info = GpuInfo::new(&name, GpuVendor::Amd);
    
    // Read frequencies
    if let Some(pp_dpm_sclk_path) = find_file_in_dir(device_path, "pp_dpm_sclk") {
        if let Ok(content) = fs::read_to_string(pp_dpm_sclk_path) {
            // Try to extract max clock from pp_dpm_sclk
            // The format is typically "3: 1860Mhz *"
            let re = Regex::new(r"(\d+): (\d+)Mhz").ok();
            if let Some(re) = re {
                let mut max_freq = 0;
                for cap in re.captures_iter(&content) {
                    if let Ok(freq) = cap[2].parse::<u32>() {
                        max_freq = std::cmp::max(max_freq, freq);
                    }
                }
                if max_freq > 0 {
                    gpu_info.max_freq_mhz = max_freq;
                    
                    // Also look for the current frequency (marked with *)
                    let re_current = Regex::new(r"(\d+): (\d+)Mhz \*").ok();
                    if let Some(re_current) = re_current {
                        if let Some(cap) = re_current.captures(&content) {
                            if let Ok(freq) = cap[2].parse::<u32>() {
                                gpu_info.freq_mhz = freq;
                            }
                        }
                    } else {
                        gpu_info.freq_mhz = max_freq; // Fall back to max freq
                    }
                }
            }
        }
    }
    
    // Determine architecture
    let (architecture, chip, process_nm) = get_amd_architecture(&name, &device_id);
    gpu_info.architecture = architecture;
    gpu_info.chip = chip;
    gpu_info.process_nm = process_nm;
    
    // Try to get memory info
    if let Some(memory_info_path) = find_file_in_dir(device_path, "mem_info_vram_total") {
        if let Ok(content) = fs::read_to_string(memory_info_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                let memory = Memory {
                    size_bytes: bytes,
                    memory_type: get_amd_memory_type(&name),
                    bus_width: get_amd_bus_width(&name),
                    clock_mhz: 0, // To be populated later
                };
                gpu_info.memory = Some(memory);
            }
        }
    }
    
    // Try to get topology information
    gpu_info.topology = get_amd_topology(&name);
    
    // Try to get cache information
    gpu_info.cache = get_amd_cache(&name);
    
    // Calculate peak performance
    if let Some(ref topology) = gpu_info.topology {
        if let Some(stream_processors) = topology.stream_processors {
            // Peak FLOPS = 2 * stream_processors * clock
            let peak_gflops = 2.0 * stream_processors as f64 * gpu_info.max_freq_mhz as f64 / 1000.0;
            gpu_info.peak_performance_gflops = Some(peak_gflops);
        }
    }
    
    Ok(gpu_info)
}

/// Find a file with the given name in a directory, including subdirectories
fn find_file_in_dir(dir: &Path, filename: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(filename) {
                    return Some(path);
                } else if path.is_dir() {
                    if let Some(found) = find_file_in_dir(&path, filename) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

/// Check if rocm-smi is available
fn is_rocm_smi_available() -> bool {
    Command::new("which")
        .arg("rocm-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Enhance GPU info using rocm-smi
fn enhance_with_rocm_smi(gpu_info: &mut GpuInfo) -> Result<()> {
    let output = Command::new("rocm-smi")
        .args(["--showdriverversion", "--showmemvendor", "--showvbios", "-a"])
        .output()
        .context("Failed to execute rocm-smi")?;
    
    if !output.status.success() {
        return Err(anyhow!("rocm-smi command failed"));
    }
    
    let output_str = String::from_utf8(output.stdout)
        .context("rocm-smi output is not valid UTF-8")?;
    
    // Extract driver version
    let re_driver = Regex::new(r"Driver Version: (.+)").ok();
    if let Some(re) = re_driver {
        if let Some(cap) = re.captures(&output_str) {
            gpu_info.driver_version = Some(cap[1].trim().to_string());
        }
    }
    
    // Could extract more info if needed...
    
    Ok(())
}

/// Determine AMD architecture, chip, and manufacturing process based on device ID and name
fn get_amd_architecture(name: &str, device_id: &str) -> (String, String, Option<u32>) {
    let name_lower = name.to_lowercase();
    
    // RDNA 3 (RX 7000)
    if name_lower.contains("rx 7900") || device_id.starts_with("744c") || device_id.starts_with("7440") {
        ("RDNA 3".to_string(), "Navi 31".to_string(), Some(5))
    }
    // RDNA 2 (RX 6000)
    else if name_lower.contains("rx 6950") || name_lower.contains("rx 6900") || device_id.starts_with("73a") {
        ("RDNA 2".to_string(), "Navi 21".to_string(), Some(7))
    }
    else if name_lower.contains("rx 6800") || device_id.starts_with("73b") {
        ("RDNA 2".to_string(), "Navi 21".to_string(), Some(7))
    }
    else if name_lower.contains("rx 6700") || device_id.starts_with("73d") {
        ("RDNA 2".to_string(), "Navi 22".to_string(), Some(7))
    }
    else if name_lower.contains("rx 6600") || device_id.starts_with("73e") || device_id.starts_with("73f") {
        ("RDNA 2".to_string(), "Navi 23".to_string(), Some(7))
    }
    else if name_lower.contains("rx 6500") || name_lower.contains("rx 6400") || device_id.starts_with("743") {
        ("RDNA 2".to_string(), "Navi 24".to_string(), Some(6))
    }
    // RDNA (RX 5000)
    else if name_lower.contains("rx 5700") || device_id.starts_with("731") {
        ("RDNA".to_string(), "Navi 10".to_string(), Some(7))
    }
    else if name_lower.contains("rx 5600") || device_id.starts_with("731") {
        ("RDNA".to_string(), "Navi 10".to_string(), Some(7))
    }
    else if name_lower.contains("rx 5500") || device_id.starts_with("7340") {
        ("RDNA".to_string(), "Navi 14".to_string(), Some(7))
    }
    // Vega
    else if name_lower.contains("vega") || name_lower.contains("radeon vii") || device_id.starts_with("66") {
        ("Vega".to_string(), "Vega 10/20".to_string(), Some(7))
    }
    // Polaris (RX 400/500)
    else if name_lower.contains("rx 5") || name_lower.contains("rx 4") || device_id.starts_with("67") {
        ("Polaris".to_string(), "Polaris".to_string(), Some(14))
    }
    // Older GCN
    else if name_lower.contains("rx 3") || name_lower.contains("r9") || device_id.starts_with("6") {
        ("GCN".to_string(), "GCN".to_string(), Some(28))
    }
    else {
        ("Unknown".to_string(), "Unknown".to_string(), None)
    }
}

/// Determine AMD memory type
fn get_amd_memory_type(name: &str) -> MemoryType {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rx 7900") {
        MemoryType::Gddr6
    } else if name_lower.contains("rx 6") {
        MemoryType::Gddr6
    } else if name_lower.contains("rx 5") {
        MemoryType::Gddr6
    } else if name_lower.contains("radeon vii") {
        MemoryType::Hbm2
    } else if name_lower.contains("vega") {
        MemoryType::Hbm2
    } else if name_lower.contains("rx 580") || name_lower.contains("rx 570") || 
              name_lower.contains("rx 480") || name_lower.contains("rx 470") {
        MemoryType::Gddr5
    } else {
        MemoryType::Unknown
    }
}

/// Determine AMD memory bus width
fn get_amd_bus_width(name: &str) -> u32 {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rx 7900") {
        384
    } else if name_lower.contains("rx 6900") || name_lower.contains("rx 6800") {
        256
    } else if name_lower.contains("rx 6700") {
        192
    } else if name_lower.contains("rx 6600") {
        128
    } else if name_lower.contains("rx 6500") || name_lower.contains("rx 6400") {
        64
    } else if name_lower.contains("rx 5700") {
        256
    } else if name_lower.contains("rx 5600") {
        192
    } else if name_lower.contains("rx 5500") {
        128
    } else if name_lower.contains("radeon vii") {
        4096  // 4096-bit for HBM2
    } else if name_lower.contains("vega") {
        2048  // 2048-bit for HBM2
    } else if name_lower.contains("rx 580") || name_lower.contains("rx 480") {
        256
    } else if name_lower.contains("rx 570") || name_lower.contains("rx 470") {
        256
    } else if name_lower.contains("rx 560") || name_lower.contains("rx 460") {
        128
    } else {
        256  // Default value
    }
}

/// Get topology information for AMD GPUs
fn get_amd_topology(name: &str) -> Option<Topology> {
    let name_lower = name.to_lowercase();
    
    let stream_processors = if name_lower.contains("rx 7900 xtx") {
        Some(12288)  // RDNA 3, 96 CUs
    } else if name_lower.contains("rx 7900 xt") {
        Some(10752)  // RDNA 3, 84 CUs
    } else if name_lower.contains("rx 6950 xt") || name_lower.contains("rx 6900 xt") {
        Some(5120)   // RDNA 2, 80 CUs
    } else if name_lower.contains("rx 6800 xt") {
        Some(4608)   // RDNA 2, 72 CUs
    } else if name_lower.contains("rx 6800") {
        Some(3840)   // RDNA 2, 60 CUs
    } else if name_lower.contains("rx 6700 xt") {
        Some(2560)   // RDNA 2, 40 CUs
    } else if name_lower.contains("rx 6600 xt") {
        Some(2048)   // RDNA 2, 32 CUs
    } else if name_lower.contains("rx 6600") {
        Some(1792)   // RDNA 2, 28 CUs
    } else if name_lower.contains("rx 6500 xt") {
        Some(1024)   // RDNA 2, 16 CUs
    } else if name_lower.contains("rx 5700 xt") {
        Some(2560)   // RDNA, 40 CUs
    } else if name_lower.contains("rx 5700") {
        Some(2304)   // RDNA, 36 CUs
    } else if name_lower.contains("rx 5600 xt") {
        Some(2048)   // RDNA, 32 CUs
    } else if name_lower.contains("rx 5500 xt") {
        Some(1408)   // RDNA, 22 CUs
    } else if name_lower.contains("radeon vii") {
        Some(3840)   // Vega 20, 60 CUs
    } else if name_lower.contains("vega 64") {
        Some(4096)   // Vega 10, 64 CUs
    } else if name_lower.contains("vega 56") {
        Some(3584)   // Vega 10, 56 CUs
    } else if name_lower.contains("rx 580") {
        Some(2304)   // Polaris 20, 36 CUs
    } else if name_lower.contains("rx 570") {
        Some(2048)   // Polaris 20, 32 CUs
    } else {
        None
    };
    
    // Compute units = stream_processors / 64 for most AMD GPUs
    let compute_units = if let Some(sp) = stream_processors {
        if name_lower.contains("rx 7") {
            sp / 128  // RDNA 3 has 128 stream processors per CU
        } else {
            sp / 64   // RDNA 1/2 and older have 64 stream processors per CU
        }
    } else {
        0
    };
    
    // ROPs (Render Output Units) and TMUs (Texture Mapping Units)
    let (rops, tmus) = if name_lower.contains("rx 7900 xtx") {
        (Some(192), Some(384))
    } else if name_lower.contains("rx 7900 xt") {
        (Some(176), Some(336))
    } else if name_lower.contains("rx 6950 xt") || name_lower.contains("rx 6900 xt") {
        (Some(128), Some(160))
    } else if name_lower.contains("rx 6800 xt") {
        (Some(128), Some(144))
    } else if name_lower.contains("rx 6800") {
        (Some(96), Some(120))
    } else if name_lower.contains("rx 6700 xt") {
        (Some(64), Some(160))
    } else if name_lower.contains("rx 6600 xt") {
        (Some(64), Some(128))
    } else if name_lower.contains("rx 5700 xt") {
        (Some(64), Some(160))
    } else if name_lower.contains("rx 580") {
        (Some(32), Some(144))
    } else {
        (None, None)
    };
    
    stream_processors.map(|sp| Topology {
        compute_units: compute_units as u32,
        cuda_cores: None,
        tensor_cores: None,
        rt_cores: None,
        sm_count: None,
        stream_processors: Some(sp),
        rops,
        tmus,
        execution_units: None,
        slices: None,
        subslices: None,
    })
}

/// Get cache information for AMD GPUs
fn get_amd_cache(name: &str) -> Option<Cache> {
    let name_lower = name.to_lowercase();
    
    // L2 and L3 cache sizes
    let (l2_size, l3_size) = if name_lower.contains("rx 7900") {
        (Some(6 * 1024 * 1024), Some(96 * 1024 * 1024))  // 6 MB L2, 96 MB Infinity Cache
    } else if name_lower.contains("rx 6950") || name_lower.contains("rx 6900") || name_lower.contains("rx 6800") {
        (Some(512 * 1024), Some(128 * 1024 * 1024))  // 512 KB L2, 128 MB Infinity Cache
    } else if name_lower.contains("rx 6700") {
        (Some(384 * 1024), Some(96 * 1024 * 1024))   // 384 KB L2, 96 MB Infinity Cache
    } else if name_lower.contains("rx 6600") {
        (Some(256 * 1024), Some(32 * 1024 * 1024))   // 256 KB L2, 32 MB Infinity Cache
    } else if name_lower.contains("rx 6500") || name_lower.contains("rx 6400") {
        (Some(128 * 1024), Some(16 * 1024 * 1024))   // 128 KB L2, 16 MB Infinity Cache
    } else if name_lower.contains("rx 5700") {
        (Some(4 * 1024 * 1024), None)                // 4 MB L2, no L3
    } else if name_lower.contains("rx 5600") {
        (Some(4 * 1024 * 1024), None)                // 4 MB L2, no L3
    } else if name_lower.contains("rx 5500") {
        (Some(4 * 1024 * 1024), None)                // 4 MB L2, no L3
    } else if name_lower.contains("radeon vii") {
        (Some(4 * 1024 * 1024), None)                // 4 MB L2, no L3
    } else if name_lower.contains("vega") {
        (Some(4 * 1024 * 1024), None)                // 4 MB L2, no L3
    } else if name_lower.contains("rx 580") || name_lower.contains("rx 570") {
        (Some(2 * 1024 * 1024), None)                // 2 MB L2, no L3
    } else {
        (None, None)
    };
    
    Some(Cache {
        l1_size: None, // AMD doesn't typically publish L1 cache sizes
        l2_size,
        l3_size,
    })
}
