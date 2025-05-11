mod args;
mod display;
mod gpu;
mod utils;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use display::print_gpufetch;
use gpu::GpuManager;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize the GPU manager
    let gpu_manager = GpuManager::new().context("Failed to initialize GPU manager")?;
    
    // Detect available GPUs
    let gpus = gpu_manager.detect_gpus().context("Failed to detect GPUs")?;
    
    if gpus.is_empty() {
        println!("No GPUs detected on the system");
        return Ok(());
    }
    
    // If list-only is specified, just list available GPUs and exit
    if args.list_only {
        println!("Detected GPUs:");
        for (idx, gpu) in gpus.iter().enumerate() {
            println!("{}: {} ({})", idx, gpu.name, gpu.vendor);
        }
        return Ok(());
    }

    // Choose which GPU to display
    let gpu_idx = if args.gpu_index >= 0 && args.gpu_index < gpus.len() as i32 {
        args.gpu_index as usize
    } else if args.gpu_index >= 0 {
        println!("GPU index {} out of range, falling back to GPU 0", args.gpu_index);
        0
    } else {
        // Negative values mean show all GPUs
        for (idx, gpu) in gpus.iter().enumerate() {
            print_gpufetch(gpu, args.color_scheme.clone(), args.logo_variant)?;
            
            // Print separator between GPUs
            if idx < gpus.len() - 1 {
                println!("\n{}\n", "-".repeat(40));
            }
        }
        return Ok(());
    };
    
    // Display information about the selected GPU
    print_gpufetch(&gpus[gpu_idx], args.color_scheme, args.logo_variant)?;
    
    Ok(())
}
