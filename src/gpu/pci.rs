use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use pci_ids::{Device, FromId, Subclass, Vendor};
use std::fs;
use std::path::{Path, PathBuf};

use crate::gpu::common::{GpuInfo, GpuVendor, PciInfo};

// PCI vendor IDs
const PCI_VENDOR_ID_NVIDIA: u16 = 0x10de;
const PCI_VENDOR_ID_AMD: u16 = 0x1002;
const PCI_VENDOR_ID_ATI: u16 = 0x1002; // ATI was acquired by AMD
const PCI_VENDOR_ID_INTEL: u16 = 0x8086;

// PCI device class codes for GPUs
const PCI_CLASS_DISPLAY_VGA: u16 = 0x0300;
const PCI_CLASS_DISPLAY_3D: u16 = 0x0302;

lazy_static! {
    static ref PCI_SYS_PATH: PathBuf = PathBuf::from("/sys/bus/pci/devices");
}

/// Detect GPUs using the PCI subsystem
pub fn detect_gpus_from_pci() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();
    
    // Check if PCI path exists
    if !PCI_SYS_PATH.exists() {
        return Err(anyhow!("PCI sysfs path not found"));
    }
    
    // Iterate through PCI devices
    for entry in fs::read_dir(&*PCI_SYS_PATH).context("Failed to read PCI devices directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let device_path = entry.path();
        
        // Try to read device info from sysfs
        if let Ok(pci_info) = read_pci_info(&device_path) {
            // Check if this is a display adapter
            if is_display_adapter(pci_info.class_id) {
                // Create GPU info based on vendor
                if let Some(gpu_info) = create_gpu_info_from_pci(&pci_info, &device_path) {
                    gpus.push(gpu_info);
                }
            }
        }
    }
    
    Ok(gpus)
}

/// Read PCI device information from sysfs
fn read_pci_info(device_path: &Path) -> Result<PciInfo> {
    // Parse PCI location from path name
    let device_name = device_path
        .file_name()
        .context("Invalid device path")?
        .to_string_lossy();
    
    // Parse PCI location (e.g., "0000:01:00.0")
    let parts: Vec<&str> = device_name.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid PCI path format"));
    }
    
    let domain = u16::from_str_radix(parts[0], 16).context("Invalid domain")?;
    
    let bus_dev_fn: Vec<&str> = parts[1].split('.').collect();
    if bus_dev_fn.len() != 2 {
        return Err(anyhow!("Invalid bus/device/function format"));
    }
    
    let bus_dev: Vec<&str> = bus_dev_fn[0].split(':').collect();
    let bus = if bus_dev.len() > 1 {
        u8::from_str_radix(bus_dev[1], 16).context("Invalid bus")?
    } else {
        u8::from_str_radix(bus_dev[0], 16).context("Invalid bus")?
    };
    
    let device = if bus_dev.len() > 1 {
        u8::from_str_radix(bus_dev[1], 16).context("Invalid device")?
    } else {
        0
    };
    
    let function = u8::from_str_radix(bus_dev_fn[1], 16).context("Invalid function")?;
    
    // Read device vendor and device ID
    let vendor_id = read_hex_file(&device_path.join("vendor"))?;
    let device_id = read_hex_file(&device_path.join("device"))?;
    let class_id = read_hex_file(&device_path.join("class"))? >> 8; // Class is in the top 16 bits
    
    Ok(PciInfo {
        vendor_id,
        device_id,
        class_id: class_id as u16,
        domain,
        bus,
        device,
        function,
    })
}

/// Read a hex value from a sysfs file
fn read_hex_file(path: &Path) -> Result<u16> {
    let content = fs::read_to_string(path).context("Failed to read file")?;
    let hex_str = content.trim().trim_start_matches("0x");
    u16::from_str_radix(hex_str, 16).context("Invalid hex value")
}

/// Check if a PCI class ID is a display adapter
fn is_display_adapter(class_id: u16) -> bool {
    class_id == PCI_CLASS_DISPLAY_VGA || class_id == PCI_CLASS_DISPLAY_3D
}

/// Create a GPU info structure from PCI information
fn create_gpu_info_from_pci(pci_info: &PciInfo, device_path: &Path) -> Option<GpuInfo> {
    let vendor = match pci_info.vendor_id {
        PCI_VENDOR_ID_NVIDIA => GpuVendor::Nvidia,
        PCI_VENDOR_ID_AMD | PCI_VENDOR_ID_ATI => GpuVendor::Amd,
        PCI_VENDOR_ID_INTEL => GpuVendor::Intel,
        _ => GpuVendor::Other(format!("Unknown (0x{:04x})", pci_info.vendor_id)),
    };
    
    // Try to get device name from pci.ids database
    let vendor_info = Vendor::from_id(pci_info.vendor_id);
    
    // Handle the different API for the pci-ids crate
    let device_name = if let Some(v) = vendor_info {
        if let Some(d) = v.devices().find(|d| d.id() == pci_info.device_id) {
            format!("{} {}", v.name(), d.name())
        } else {
            format!("{} Device {:04x}", v.name(), pci_info.device_id)
        }
    } else {
        format!("Unknown Device {:04x}:{:04x}", pci_info.vendor_id, pci_info.device_id)
    };
    
    // Create basic GPU info
    let mut gpu_info = GpuInfo::new(&device_name, vendor);
    gpu_info.pci_info = Some(pci_info.clone());
    
    // Try to read some additional info from sysfs
    if let Ok(freq) = fs::read_to_string(device_path.join("drm").join("card0").join("device").join("pp_dpm_sclk")) {
        // AMD-style frequency info
        if let Some(max_freq) = freq.lines().last() {
            if let Some(mhz_str) = max_freq.split_whitespace().nth(1) {
                if let Ok(mhz) = mhz_str.trim_end_matches("Mhz").parse::<u32>() {
                    gpu_info.freq_mhz = mhz;
                    gpu_info.max_freq_mhz = mhz;
                }
            }
        }
    } else if let Ok(freq) = fs::read_to_string(device_path.join("drm").join("card0").join("gt_max_freq_mhz")) {
        // Intel-style frequency info
        if let Ok(mhz) = freq.trim().parse::<u32>() {
            gpu_info.freq_mhz = mhz;
            gpu_info.max_freq_mhz = mhz;
        }
    }
    
    Some(gpu_info)
}
