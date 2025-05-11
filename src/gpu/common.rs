use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Arm,
    Other(String),
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Arm => write!(f, "ARM"),
            GpuVendor::Other(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MemoryType {
    Ddr3,
    Ddr4,
    Gddr5,
    Gddr5X,
    Gddr6,
    Gddr6X,
    Hbm,
    Hbm2,
    Unknown,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::Ddr3 => write!(f, "DDR3"),
            MemoryType::Ddr4 => write!(f, "DDR4"),
            MemoryType::Gddr5 => write!(f, "GDDR5"),
            MemoryType::Gddr5X => write!(f, "GDDR5X"),
            MemoryType::Gddr6 => write!(f, "GDDR6"),
            MemoryType::Gddr6X => write!(f, "GDDR6X"),
            MemoryType::Hbm => write!(f, "HBM"),
            MemoryType::Hbm2 => write!(f, "HBM2"),
            MemoryType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub size_bytes: u64,
    pub memory_type: MemoryType,
    pub bus_width: u32,
    pub clock_mhz: u32,
}

#[derive(Debug, Clone)]
pub struct Topology {
    // Common fields
    pub compute_units: u32,
    
    // NVIDIA specific 
    pub cuda_cores: Option<u32>,
    pub tensor_cores: Option<u32>,
    pub rt_cores: Option<u32>,
    pub sm_count: Option<u32>,
    
    // AMD specific
    pub stream_processors: Option<u32>,
    pub rops: Option<u32>,
    pub tmus: Option<u32>,
    
    // Intel specific
    pub execution_units: Option<u32>,
    pub slices: Option<u32>,
    pub subslices: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub l1_size: Option<u64>,
    pub l2_size: Option<u64>,
    pub l3_size: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PciInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u16,
    pub domain: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub architecture: String,
    pub chip: String,
    pub process_nm: Option<u32>,
    pub freq_mhz: u32,
    pub max_freq_mhz: u32,
    pub memory: Option<Memory>,
    pub topology: Option<Topology>,
    pub cache: Option<Cache>,
    pub pci_info: Option<PciInfo>,
    pub driver_version: Option<String>,
    pub compute_capability: Option<String>, // For NVIDIA
    pub opengl_version: Option<String>,
    pub vulkan_version: Option<String>,
    pub opencl_version: Option<String>,
    pub peak_performance_gflops: Option<f64>,
    pub is_integrated: bool,
}

impl GpuInfo {
    pub fn new(name: &str, vendor: GpuVendor) -> Self {
        GpuInfo {
            name: name.to_string(),
            vendor,
            architecture: String::from("Unknown"),
            chip: String::from("Unknown"),
            process_nm: None,
            freq_mhz: 0,
            max_freq_mhz: 0,
            memory: None,
            topology: None,
            cache: None,
            pci_info: None,
            driver_version: None,
            compute_capability: None,
            opengl_version: None,
            vulkan_version: None,
            opencl_version: None,
            peak_performance_gflops: None,
            is_integrated: false,
        }
    }
    
    pub fn get_memory_size_readable(&self) -> String {
        if let Some(ref memory) = self.memory {
            let size_mb = memory.size_bytes / 1024 / 1024;
            if size_mb >= 1024 {
                format!("{:.1} GB", size_mb as f64 / 1024.0)
            } else {
                format!("{} MB", size_mb)
            }
        } else {
            String::from("Unknown")
        }
    }
    
    pub fn get_process_readable(&self) -> String {
        match self.process_nm {
            Some(nm) => format!("{} nm", nm),
            None => String::from("Unknown"),
        }
    }
    
    pub fn get_compute_units_readable(&self) -> String {
        if let Some(ref topology) = self.topology {
            match self.vendor {
                GpuVendor::Nvidia => {
                    if let Some(cores) = topology.cuda_cores {
                        format!("{} CUDA Cores", cores)
                    } else {
                        String::from("Unknown CUDA Cores")
                    }
                }
                GpuVendor::Amd => {
                    if let Some(sps) = topology.stream_processors {
                        format!("{} Stream Processors", sps)
                    } else {
                        String::from("Unknown Stream Processors")
                    }
                }
                GpuVendor::Intel => {
                    if let Some(eus) = topology.execution_units {
                        format!("{} Execution Units", eus)
                    } else {
                        String::from("Unknown Execution Units")
                    }
                }
                _ => format!("{} Compute Units", topology.compute_units),
            }
        } else {
            String::from("Unknown Compute Units")
        }
    }
}
