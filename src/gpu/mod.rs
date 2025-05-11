pub mod common;
pub mod pci;
pub mod amd;
pub mod intel;
pub mod nvidia;

use anyhow::{Context, Result};
use common::{GpuInfo, GpuVendor};

/// Manager for GPU detection and information gathering
pub struct GpuManager {
    pub verbose: bool,
}

impl GpuManager {
    /// Create a new GPU manager instance
    pub fn new() -> Result<Self> {
        Ok(GpuManager { verbose: false })
    }
    
    /// Set verbosity level
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Detect available GPUs on the system
    pub fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();
        
        // Detect NVIDIA GPUs
        match nvidia::detect_nvidia_gpus() {
            Ok(mut nvidia_gpus) => gpus.append(&mut nvidia_gpus),
            Err(e) => {
                if self.verbose {
                    eprintln!("Failed to detect NVIDIA GPUs: {}", e);
                }
            }
        }
        
        // Detect AMD GPUs
        match amd::detect_amd_gpus() {
            Ok(mut amd_gpus) => gpus.append(&mut amd_gpus),
            Err(e) => {
                if self.verbose {
                    eprintln!("Failed to detect AMD GPUs: {}", e);
                }
            }
        }
        
        // Detect Intel GPUs
        match intel::detect_intel_gpus() {
            Ok(mut intel_gpus) => gpus.append(&mut intel_gpus),
            Err(e) => {
                if self.verbose {
                    eprintln!("Failed to detect Intel GPUs: {}", e);
                }
            }
        }
        
        // Fallback to PCI detection if no GPUs found
        if gpus.is_empty() {
            let pci_gpus = pci::detect_gpus_from_pci().context("Failed to detect GPUs from PCI")?;
            gpus.extend(pci_gpus);
        }
        
        Ok(gpus)
    }
}
