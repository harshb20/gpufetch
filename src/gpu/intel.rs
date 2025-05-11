use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::gpu::common::{Cache, GpuInfo, GpuVendor, Memory, MemoryType, Topology};

/// Detect Intel GPUs
pub fn detect_intel_gpus() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();
    
    // Check for Intel GPUs in the system
    if let Ok(intel_gpu_paths) = find_intel_gpus_in_sysfs() {
        for path in intel_gpu_paths {
            if let Ok(gpu_info) = get_intel_gpu_info_from_sysfs(&path) {
                gpus.push(gpu_info);
            }
        }
    }
    
    Ok(gpus)
}

/// Find Intel GPU directories in sysfs
fn find_intel_gpus_in_sysfs() -> Result<Vec<PathBuf>> {
    let mut gpu_paths = Vec::new();
    
    // Try Intel card directory in /sys/class/drm
    let drm_path = Path::new("/sys/class/drm");
    if drm_path.exists() {
        for entry in fs::read_dir(drm_path).context("Failed to read DRM directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // Check for Intel GPUs (card directories with i915 driver)
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("card") && !name.contains("renderD") {
                    let device_path = path.join("device");
                    if device_path.exists() {
                        // Check if this is an Intel GPU
                        let vendor_path = device_path.join("vendor");
                        if let Ok(vendor) = fs::read_to_string(vendor_path) {
                            if vendor.trim() == "0x8086" {
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

/// Extract Intel GPU information from sysfs
fn get_intel_gpu_info_from_sysfs(device_path: &Path) -> Result<GpuInfo> {
    // Read device ID
    let device_id_path = device_path.join("device");
    let device_id = fs::read_to_string(device_id_path)
        .map(|id| id.trim().trim_start_matches("0x").to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    // Read product name
    let name = get_intel_gpu_name(&device_id, device_path);
    
    // Create basic GPU info
    let mut gpu_info = GpuInfo::new(&name, GpuVendor::Intel);
    gpu_info.is_integrated = true;  // Most Intel GPUs are integrated
    
    // Read frequencies
    read_intel_frequencies(device_path, &mut gpu_info);
    
    // Determine architecture and other info
    let (architecture, chip, generation, process_nm) = get_intel_architecture(&name, &device_id);
    gpu_info.architecture = architecture;
    gpu_info.chip = chip;
    gpu_info.process_nm = process_nm;
    
    // Try to get memory info (integrated GPUs usually use system memory)
    gpu_info.memory = get_intel_memory(&name);
    
    // Try to get topology information
    gpu_info.topology = get_intel_topology(&name, generation);
    
    // Try to get cache information
    gpu_info.cache = get_intel_cache(&name, generation);
    
    // Calculate peak performance
    if let Some(ref topology) = gpu_info.topology {
        if let Some(execution_units) = topology.execution_units {
            // Peak FLOPS = 2 * 8 * execution_units * clock (Intel GPUs have 8 ALUs per EU)
            let peak_gflops = 2.0 * 8.0 * execution_units as f64 * gpu_info.max_freq_mhz as f64 / 1000.0;
            gpu_info.peak_performance_gflops = Some(peak_gflops);
        }
    }
    
    // Get driver info
    if is_intel_gpu_tool_available() {
        let driver_version = get_intel_driver_version();
        gpu_info.driver_version = driver_version;
    }
    
    Ok(gpu_info)
}

/// Get Intel GPU name based on device ID
fn get_intel_gpu_name(device_id: &str, device_path: &Path) -> String {
    // First try to read the product_name
    let product_name_path = device_path.join("product_name");
    if product_name_path.exists() {
        if let Ok(name) = fs::read_to_string(product_name_path) {
            let name = name.trim();
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    
    // Map known device IDs to names
    match device_id {
        // Tiger Lake (Gen12)
        "9a49" => "Intel Iris Xe Graphics (96 EUs)".to_string(),
        "9a40" => "Intel Iris Xe Graphics (80 EUs)".to_string(),
        "9a78" => "Intel UHD Graphics (32 EUs)".to_string(),
        // Rocket Lake (Gen12)
        "4c8a" => "Intel UHD Graphics 750".to_string(),
        "4c8b" => "Intel UHD Graphics 730".to_string(),
        // Alder Lake (Gen12)
        "4680" => "Intel UHD Graphics 770".to_string(),
        "4690" => "Intel UHD Graphics 770".to_string(),
        "4692" => "Intel UHD Graphics 730".to_string(),
        "4693" => "Intel UHD Graphics 710".to_string(),
        // Ice Lake (Gen11)
        "8a52" => "Intel Iris Plus Graphics G7".to_string(),
        "8a53" => "Intel Iris Plus Graphics G7".to_string(),
        "8a5c" => "Intel Iris Plus Graphics G4".to_string(),
        "8a5a" => "Intel Iris Plus Graphics G4".to_string(),
        "8a51" => "Intel Iris Plus Graphics G1".to_string(),
        "8a56" => "Intel UHD Graphics G1".to_string(),
        "8a58" => "Intel UHD Graphics G1".to_string(),
        // Gen9.5 (Kaby Lake, Coffee Lake, etc.)
        "5917" => "Intel UHD Graphics 620".to_string(),
        "3ea0" => "Intel UHD Graphics 620".to_string(),
        "3e91" => "Intel UHD Graphics 630".to_string(),
        "3e92" => "Intel UHD Graphics 630".to_string(),
        "3e98" => "Intel UHD Graphics 630".to_string(),
        "3e9b" => "Intel UHD Graphics 630".to_string(),
        "9bc5" => "Intel UHD Graphics 630".to_string(),
        "9bc8" => "Intel UHD Graphics 630".to_string(),
        "5902" => "Intel HD Graphics 610".to_string(),
        "5906" => "Intel HD Graphics 610".to_string(),
        "590b" => "Intel HD Graphics 610".to_string(),
        "591e" => "Intel HD Graphics 615".to_string(),
        "5912" => "Intel HD Graphics 630".to_string(),
        "591b" => "Intel HD Graphics 630".to_string(),
        "591a" => "Intel HD Graphics P630".to_string(),
        "591d" => "Intel HD Graphics P630".to_string(),
        "5926" => "Intel Iris Plus Graphics 640".to_string(),
        "5927" => "Intel Iris Plus Graphics 650".to_string(),
        "3185" => "Intel UHD Graphics 600".to_string(),
        "3184" => "Intel UHD Graphics 605".to_string(),
        // Gen9 (Skylake)
        "1902" => "Intel HD Graphics 510".to_string(),
        "1906" => "Intel HD Graphics 510".to_string(),
        "190b" => "Intel HD Graphics 510".to_string(),
        "191e" => "Intel HD Graphics 515".to_string(),
        "1916" => "Intel HD Graphics 520".to_string(),
        "1921" => "Intel HD Graphics 520".to_string(),
        "1912" => "Intel HD Graphics 530".to_string(),
        "191b" => "Intel HD Graphics 530".to_string(),
        "191d" => "Intel HD Graphics P530".to_string(),
        // Gen8 (Broadwell)
        "1606" => "Intel HD Graphics (Broadwell)".to_string(),
        "161e" => "Intel HD Graphics 5300".to_string(),
        "1616" => "Intel HD Graphics 5500".to_string(),
        "1612" => "Intel HD Graphics 5600".to_string(),
        "161a" => "Intel HD Graphics P5700".to_string(),
        "1626" => "Intel HD Graphics 6000".to_string(),
        "162b" => "Intel Iris Graphics 6100".to_string(),
        "1622" => "Intel Iris Pro Graphics 6200".to_string(),
        "162a" => "Intel Iris Pro Graphics P6300".to_string(),
        // Gen7.5 (Haswell)
        "0402" => "Intel HD Graphics (Haswell)".to_string(),
        "0406" => "Intel HD Graphics (Haswell)".to_string(),
        "040a" => "Intel HD Graphics (Haswell)".to_string(),
        "0412" => "Intel HD Graphics 4600".to_string(),
        "0416" => "Intel HD Graphics 4600".to_string(),
        "041a" => "Intel HD Graphics P4600".to_string(),
        "0a16" => "Intel HD Graphics 4400".to_string(),
        "0a1e" => "Intel HD Graphics 4200".to_string(),
        "0a2e" => "Intel Iris Graphics 5100".to_string(),
        "0d22" => "Intel Iris Pro Graphics 5200".to_string(),
        "0d26" => "Intel Iris Pro Graphics P5200".to_string(),
        // Gen7 (Ivy Bridge)
        "0152" => "Intel HD Graphics 2500".to_string(),
        "0156" => "Intel HD Graphics 2500".to_string(),
        "0162" => "Intel HD Graphics 4000".to_string(),
        "0166" => "Intel HD Graphics 4000".to_string(),
        "016a" => "Intel HD Graphics P4000".to_string(),
        "015a" => "Intel HD Graphics (Ivy Bridge)".to_string(),
        "0f30" => "Intel HD Graphics (Bay Trail)".to_string(),
        "0f31" => "Intel HD Graphics (Bay Trail)".to_string(),
        "0f32" => "Intel HD Graphics (Bay Trail)".to_string(),
        "0f33" => "Intel HD Graphics (Bay Trail)".to_string(),
        "0155" => "Intel HD Graphics (Cherry Trail)".to_string(),
        "0157" => "Intel HD Graphics (Cherry Trail)".to_string(),
        // Gen6 (Sandy Bridge)
        "0102" => "Intel HD Graphics 2000".to_string(),
        "0106" => "Intel HD Graphics 2000".to_string(),
        "0112" => "Intel HD Graphics 3000".to_string(),
        "0116" => "Intel HD Graphics 3000".to_string(),
        "0122" => "Intel HD Graphics 3000".to_string(),
        "0126" => "Intel HD Graphics 3000".to_string(),
        "010a" => "Intel HD Graphics (Sandy Bridge)".to_string(),
        _ => format!("Intel GPU (Device ID: {}, Generation Unknown)", device_id),
    }
}

/// Read Intel GPU frequencies from sysfs
fn read_intel_frequencies(device_path: &Path, gpu_info: &mut GpuInfo) {
    // Try to read max frequency
    let max_freq_path = device_path.join("gt_max_freq_mhz");
    if max_freq_path.exists() {
        if let Ok(content) = fs::read_to_string(max_freq_path) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                gpu_info.max_freq_mhz = freq;
            }
        }
    }
    
    // Try to read min frequency
    let min_freq_path = device_path.join("gt_min_freq_mhz");
    if min_freq_path.exists() {
        if let Ok(content) = fs::read_to_string(min_freq_path) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                gpu_info.freq_mhz = freq;
            }
        }
    } else {
        gpu_info.freq_mhz = gpu_info.max_freq_mhz;
    }
    
    // If neither is found, use reasonable defaults
    if gpu_info.max_freq_mhz == 0 {
        let name_lower = gpu_info.name.to_lowercase();
        if name_lower.contains("gen12") || name_lower.contains("iris xe") {
            gpu_info.max_freq_mhz = 1450;  // Typical for Tiger Lake
        } else if name_lower.contains("gen11") || name_lower.contains("iris plus") {
            gpu_info.max_freq_mhz = 1100;  // Typical for Ice Lake
        } else if name_lower.contains("uhd graphics") {
            gpu_info.max_freq_mhz = 1150;  // Typical for UHD Graphics
        } else if name_lower.contains("hd graphics 6") {
            gpu_info.max_freq_mhz = 1100;  // Typical for HD Graphics 6xxx
        } else if name_lower.contains("hd graphics 5") {
            gpu_info.max_freq_mhz = 1050;  // Typical for HD Graphics 5xxx
        } else if name_lower.contains("hd graphics 4") {
            gpu_info.max_freq_mhz = 1150;  // Typical for HD Graphics 4xxx
        } else if name_lower.contains("hd graphics 3") {
            gpu_info.max_freq_mhz = 1150;  // Typical for HD Graphics 3xxx
        } else if name_lower.contains("hd graphics 2") {
            gpu_info.max_freq_mhz = 1100;  // Typical for HD Graphics 2xxx
        } else {
            gpu_info.max_freq_mhz = 1000;  // Default
        }
        
        if gpu_info.freq_mhz == 0 {
            gpu_info.freq_mhz = gpu_info.max_freq_mhz;
        }
    }
}

/// Check if intel-gpu-tools is available
fn is_intel_gpu_tool_available() -> bool {
    Command::new("which")
        .arg("intel_gpu_top")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get Intel driver version
fn get_intel_driver_version() -> Option<String> {
    // Try reading from X server output
    if let Ok(output) = Command::new("sh")
        .args(["-c", "DISPLAY=:0 glxinfo | grep 'OpenGL version string'"])
        .output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let re = Regex::new(r"OpenGL version string: .* Mesa (.+)").ok()?;
            if let Some(cap) = re.captures(&output_str) {
                return Some(cap[1].trim().to_string());
            }
        }
    }
    
    // Try reading from direct rendering info
    if let Ok(output) = Command::new("sh")
        .args(["-c", "DISPLAY=:0 glxinfo | grep 'direct rendering'"])
        .output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("direct rendering: Yes") {
                return Some("Mesa DRI Intel".to_string());
            }
        }
    }
    
    // Try finding kernel driver version
    if let Ok(output) = Command::new("sh")
        .args(["-c", "modinfo -F version i915"])
        .output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return Some(format!("i915 {}", version));
            }
        }
    }
    
    None
}

/// Determine Intel architecture and generation
fn get_intel_architecture(name: &str, device_id: &str) -> (String, String, u32, Option<u32>) {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("iris xe") || device_id.starts_with("9a") || 
       device_id.starts_with("4c8") || device_id.starts_with("468") || 
       device_id.starts_with("469") {
        ("Gen12 (Xe)".to_string(), "Gen12".to_string(), 12, Some(10))
    }
    else if name_lower.contains("iris plus") || device_id.starts_with("8a") {
        ("Gen11".to_string(), "Gen11".to_string(), 11, Some(10))
    }
    else if name_lower.contains("uhd graphics") || name_lower.contains("hd graphics 6") || 
             device_id.starts_with("3e") || device_id.starts_with("3184") || 
             device_id.starts_with("3185") || device_id.starts_with("9bc") {
        ("Gen9.5".to_string(), "Gen9.5".to_string(), 10, Some(14))
    }
    else if name_lower.contains("hd graphics 5") || device_id.starts_with("19") {
        ("Gen9".to_string(), "Gen9".to_string(), 9, Some(14))
    }
    else if name_lower.contains("hd graphics") && (name_lower.contains("6000") || 
             name_lower.contains("5500") || name_lower.contains("5300") || 
             device_id.starts_with("16")) {
        ("Gen8 (Broadwell)".to_string(), "Gen8".to_string(), 8, Some(14))
    }
    else if name_lower.contains("hd graphics 4") || device_id.starts_with("04") || 
             device_id.starts_with("0a") || device_id.starts_with("0d2") {
        ("Gen7.5 (Haswell)".to_string(), "Gen7.5".to_string(), 8, Some(22))
    }
    else if name_lower.contains("hd graphics 2500") || name_lower.contains("hd graphics 4000") || 
             device_id.starts_with("015") || device_id.starts_with("016") || 
             device_id.starts_with("0f3") {
        ("Gen7 (Ivy Bridge)".to_string(), "Gen7".to_string(), 7, Some(22))
    }
    else if name_lower.contains("hd graphics 2000") || name_lower.contains("hd graphics 3000") || 
             device_id.starts_with("010") || device_id.starts_with("011") || 
             device_id.starts_with("012") {
        ("Gen6 (Sandy Bridge)".to_string(), "Gen6".to_string(), 6, Some(32))
    }
    else {
        ("Unknown".to_string(), "Unknown".to_string(), 0, None)
    }
}

/// Create a memory object for Intel GPUs
fn get_intel_memory(name: &str) -> Option<Memory> {
    let name_lower = name.to_lowercase();
    
    // Integrated GPUs use system memory, so size is not fixed
    // Common sizes in laptops/desktops range from 128MB to 1.5GB dynamically allocated
    
    // Memory type depends on the CPU generation
    let memory_type = if name_lower.contains("iris xe") || name_lower.contains("gen12") {
        MemoryType::Ddr4  // Tiger Lake, Rocket Lake, Alder Lake typically use DDR4
    } else if name_lower.contains("iris plus") || name_lower.contains("gen11") {
        MemoryType::Ddr4  // Ice Lake typically uses DDR4
    } else if name_lower.contains("uhd graphics") || name_lower.contains("hd graphics 6") {
        MemoryType::Ddr4  // Coffee Lake, Kaby Lake typically use DDR4
    } else if name_lower.contains("hd graphics 5") {
        MemoryType::Ddr3  // Skylake could use DDR3 or DDR4
    } else if name_lower.contains("hd graphics 4") {
        MemoryType::Ddr3  // Haswell, Broadwell typically use DDR3
    } else {
        MemoryType::Ddr3  // Older generations use DDR3
    };
    
    // Bus width is usually the same as system memory bus width
    // but we'll make some approximations based on generation
    let bus_width = if name_lower.contains("iris xe") || name_lower.contains("gen12") {
        128  // Newer integrated GPUs typically use 128-bit memory bus
    } else if name_lower.contains("iris plus") || name_lower.contains("gen11") {
        128
    } else if name_lower.contains("uhd graphics") || name_lower.contains("hd graphics 6") {
        128
    } else {
        64   // Older integrated GPUs typically use 64-bit memory bus
    };
    
    // Provide a default size (actual size is dynamic based on system memory)
    let size_bytes = 1024 * 1024 * 1024;  // 1 GB is a reasonable default
    
    // Memory clock is typically the same as system memory
    let clock_mhz = if name_lower.contains("iris xe") || name_lower.contains("gen12") {
        3200  // DDR4-3200 is common for newer CPUs
    } else if name_lower.contains("iris plus") || name_lower.contains("gen11") {
        2933  // DDR4-2933 is common for Ice Lake
    } else if name_lower.contains("uhd graphics") || name_lower.contains("hd graphics 6") {
        2666  // DDR4-2666 is common for Coffee Lake
    } else if name_lower.contains("hd graphics 5") {
        2133  // DDR3-2133 or DDR4-2133 is common for Skylake
    } else if name_lower.contains("hd graphics 4") {
        1600  // DDR3-1600 is common for Haswell
    } else {
        1333  // DDR3-1333 is common for older generations
    };
    
    Some(Memory {
        size_bytes,
        memory_type,
        bus_width,
        clock_mhz,
    })
}

/// Get topology information for Intel GPUs
fn get_intel_topology(name: &str, generation: u32) -> Option<Topology> {
    let name_lower = name.to_lowercase();
    
    let execution_units = if name_lower.contains("iris xe") && name_lower.contains("96") {
        Some(96)  // Xe Graphics with 96 EUs
    } else if name_lower.contains("iris xe") && name_lower.contains("80") {
        Some(80)  // Xe Graphics with 80 EUs
    } else if name_lower.contains("iris xe") || name_lower.contains("770") {
        Some(32)  // Default Xe Graphics or UHD 770
    } else if name_lower.contains("750") {
        Some(32)  // UHD 750
    } else if name_lower.contains("730") {
        Some(24)  // UHD 730
    } else if name_lower.contains("710") {
        Some(16)  // UHD 710
    } else if name_lower.contains("iris plus") && name_lower.contains("g7") {
        Some(64)  // Gen11 Iris Plus G7
    } else if name_lower.contains("iris plus") && name_lower.contains("g4") {
        Some(48)  // Gen11 Iris Plus G4
    } else if name_lower.contains("iris plus") || name_lower.contains("g1") {
        Some(32)  // Default Iris Plus or G1
    } else if name_lower.contains("uhd graphics 630") {
        Some(24)  // UHD 630
    } else if name_lower.contains("uhd graphics 620") {
        Some(24)  // UHD 620
    } else if name_lower.contains("uhd graphics") {
        Some(24)  // Default UHD Graphics
    } else if name_lower.contains("iris graphics") {
        Some(48)  // Iris Graphics
    } else if name_lower.contains("iris pro") {
        Some(48)  // Iris Pro
    } else if name_lower.contains("hd graphics 6") {
        Some(48)  // HD 6xxx
    } else if name_lower.contains("hd graphics 5") {
        Some(24)  // HD 5xxx
    } else if name_lower.contains("hd graphics 4") {
        Some(20)  // HD 4xxx
    } else if name_lower.contains("hd graphics 3") {
        Some(12)  // HD 3xxx
    } else if name_lower.contains("hd graphics 2") {
        Some(6)   // HD 2xxx
    } else {
        None
    };
    
    // Structure depends on generation
    let (slices, subslices) = match generation {
        12 => {  // Gen12 (Xe)
            if name_lower.contains("96") {
                (Some(1), Some(6))  // 1 slice, 6 subslices, 16 EUs per subslice
            } else if name_lower.contains("80") {
                (Some(1), Some(5))  // 1 slice, 5 subslices, 16 EUs per subslice
            } else if execution_units.unwrap_or(0) >= 32 {
                (Some(1), Some(2))  // 1 slice, 2 subslices, 16 EUs per subslice
            } else {
                (Some(1), Some(1))  // 1 slice, 1 subslice, variable EUs
            }
        },
        11 => {  // Gen11
            if execution_units.unwrap_or(0) >= 64 {
                (Some(1), Some(8))  // 1 slice, 8 subslices, 8 EUs per subslice
            } else if execution_units.unwrap_or(0) >= 48 {
                (Some(1), Some(6))  // 1 slice, 6 subslices, 8 EUs per subslice
            } else {
                (Some(1), Some(4))  // 1 slice, 4 subslices, 8 EUs per subslice
            }
        },
        9 | 10 => {  // Gen9, Gen9.5
            if execution_units.unwrap_or(0) >= 48 {
                (Some(3), Some(6))  // 3 slices, 6 subslices total
            } else if execution_units.unwrap_or(0) >= 24 {
                (Some(1), Some(3))  // 1 slice, 3 subslices
            } else {
                (Some(1), Some(2))  // 1 slice, 2 subslices
            }
        },
        8 => {  // Gen8
            if execution_units.unwrap_or(0) >= 48 {
                (Some(2), Some(6))  // 2 slices, 6 subslices total
            } else {
                (Some(1), Some(3))  // 1 slice, 3 subslices
            }
        },
        7 => {  // Gen7, Gen7.5
            if execution_units.unwrap_or(0) >= 40 {
                (Some(1), Some(4))  // 1 slice, 4 subslices
            } else if execution_units.unwrap_or(0) >= 20 {
                (Some(1), Some(2))  // 1 slice, 2 subslices
            } else {
                (Some(1), Some(1))  // 1 slice, 1 subslice
            }
        },
        6 => {  // Gen6
            if execution_units.unwrap_or(0) >= 12 {
                (Some(1), Some(2))  // 1 slice, 2 subslices
            } else {
                (Some(1), Some(1))  // 1 slice, 1 subslice
            }
        },
        _ => (None, None),
    };
    
    execution_units.map(|eus| Topology {
        compute_units: eus,
        cuda_cores: None,
        tensor_cores: None,
        rt_cores: None,
        sm_count: None,
        stream_processors: None,
        rops: None,
        tmus: None,
        execution_units: Some(eus),
        slices,
        subslices,
    })
}

/// Get cache information for Intel GPUs
fn get_intel_cache(name: &str, generation: u32) -> Option<Cache> {
    let name_lower = name.to_lowercase();
    
    let l3_size = if name_lower.contains("iris xe") && (name_lower.contains("96") || name_lower.contains("80")) {
        Some(16 * 1024 * 1024)  // 16 MB for high-end Xe Graphics
    } else if name_lower.contains("iris xe") {
        Some(8 * 1024 * 1024)   // 8 MB for other Xe Graphics
    } else if name_lower.contains("iris plus g7") {
        Some(1 * 1024 * 1024)   // ~1 MB for Iris Plus G7
    } else if name_lower.contains("iris plus") {
        Some(768 * 1024)        // ~768 KB for other Iris Plus
    } else if name_lower.contains("iris pro") {
        Some(128 * 1024 * 1024) // 128 MB eDRAM for Iris Pro
    } else if name_lower.contains("iris graphics") {
        Some(48 * 1024 * 1024)  // 48 MB eDRAM for Iris Graphics
    } else {
        None
    };
    
    let l2_size = match generation {
        12 => Some(2 * 1024 * 1024),  // 2 MB for Gen12
        11 => Some(1 * 1024 * 1024),  // 1 MB for Gen11
        9 | 10 => Some(768 * 1024),   // 768 KB for Gen9/Gen9.5
        8 => Some(512 * 1024),        // 512 KB for Gen8
        7 => Some(256 * 1024),        // 256 KB for Gen7/Gen7.5
        6 => Some(128 * 1024),        // 128 KB for Gen6
        _ => None,
    };
    
    Some(Cache {
        l1_size: None,  // Intel doesn't typically publish L1 cache sizes
        l2_size,
        l3_size,
    })
}
