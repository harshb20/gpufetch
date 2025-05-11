use anyhow::Result;
use colored::{Color, Colorize};
use std::io::{self, Write};

use crate::args::{ColorScheme, LogoVariant};
use crate::gpu::common::{GpuInfo, GpuVendor};

/// ASCII art logos for different vendors
const NVIDIA_LOGO: &str = r#"
               ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
               ⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
           .⣿⣿⣿.     ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
      .⣿⣿⣿⣿.   ,⣿⣿⣿.     ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
   ,⣿⣿⣿'      ⣿.   ⣿⣿⣿⣿:     ⣿⣿⣿⣿⣿⣿⣿⣿⣿
.⣿⣿⣿⣿    ⣿⣿⣿⣿⣿ .      .⣿⣿⣿.    ⣿⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿   :⣿⣿,    ⣿⣿⣿.    ⣿⣿⣿⣿    :⣿⣿⣿⣿⣿⣿⣿
 ⣿⣿⣿⣿   ⣿⣿⣿.   ⣿⣿⣿⣿⣿.⣿⣿⣿⣿    :⣿⣿⣿⣿⣿⣿⣿⣿
  :⣿⣿⣿   ,⣿⣿⣿.  ⣿⣿⣿⣿⣿⣿⣿.   .⣿⣿⣿⣿.     ⣿⣿⣿⣿
    ⣿⣿⣿⣿.   .⣿⣿⣿.       ,⣿⣿⣿⣿⣿        .⣿⣿⣿⣿
      ⣿⣿⣿⣿⣿:.    ,⣿⣿⣿⣿::::::::::⣿⣿⣿.        :⣿⣿⣿⣿⣿⣿⣿⣿
         ⣿⣿⣿⣿⣿⣿⣿⣿⣿.            '⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
               ⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿

######.  ##   ##  ##  ######   ##    ###    
##   ##  ##   ##  ##  ##   ##  ##   #: :#   
##   ##   ## ##   ##  ##   ##  ##  #######  
##   ##    ###    ##  ######   ## ##     ## "#;

const NVIDIA_LOGO_SHORT: &str = r#"
               ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
               ⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
           .⣿⣿⣿.     ⣿⣿⣿⣿⣿⣿⣿⣿
      .⣿⣿⣿⣿.   ,⣿⣿⣿.     ⣿⣿⣿⣿
   ,⣿⣿⣿'      ⣿.   ⣿⣿⣿⣿:     ⣿
.⣿⣿⣿⣿    ⣿⣿⣿⣿⣿ .      .⣿⣿⣿.    
⣿⣿⣿⣿   :⣿⣿,    ⣿⣿⣿.    ⣿⣿⣿⣿    
 ⣿⣿⣿⣿   ⣿⣿⣿.   ⣿⣿⣿⣿⣿.⣿⣿⣿⣿    
  :⣿⣿⣿   ,⣿⣿⣿.  ⣿⣿⣿⣿⣿⣿⣿.   .⣿⣿
    ⣿⣿⣿⣿.   .⣿⣿⣿.       ,⣿⣿⣿⣿
               ⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿"#;

const INTEL_LOGO: &str = r#"
                   .#################.          
              .####                   ####.     
          .##                             ###   
       ##                          :##     ###  
    #                ##            :##      ##  
  ##   ##  ######.   ####  ######  :##      ##  
 ##    ##  ##:  ##:  ##   ##   ### :##     ###  
##     ##  ##:  ##:  ##  :######## :##    ##    
##     ##  ##:  ##:  ##   ##.   .  :## ####     
##      #  ##:  ##:  ####  #####:   ##          
 ##                                             
  ###.                         ..o####.         
   ######oo...         ..oo#######              
          o###############o                     "#;

const INTEL_LOGO_SHORT: &str = r#"
                   .########.          
              .####         ####.     
          .##                   ###   
       ##                 ##     ###  
  ##   ##  ######. ####  ##      ##  
 ##    ##  ##:  ##  ##  ###     ###  
##     ##  ##:  ##  ##  ##    ##    
##     ##  ##:  ##  ##   ## ####     
 ##                                   
   ######o..     ..o#######           "#;

const AMD_LOGO: &str = r#"
                  :+++++++++++++++++:                  
              -++++.                .++++:              
          .++++.                        -++++.          
       -++++-                               :++++-       
     -+++:                                     -+++-     
   .+++.                                         .+++.   
  -++-                                             -++-  
 -++-                                               -++- 
.++:                                                 :++.
+++                                                   +++
+++                                                   +++
+++                                                   +++
+++               ..-=++++++++==-..                   +++
+++            -+++=:.          .:=+++=.              +++
+++         .+++:                     :+++.           +++
+++        =++-                         -++=          +++
+++       =++.                           .++=         +++
+++       +++                             +++         +++
+++       =++.                           .++=         +++
+++        =++-                         -++=          +++
+++         .+++:                     :+++.           +++
+++            -+++=:.          .:=+++=.              +++
+++               ..-=++++++++==-..                   +++
+++                                                   +++
+++                                                   +++
+++                                                   +++
.++:                                                 :++.
 -++-                                               -++- 
  -++-                                             -++-  
   .+++.                                         .+++.   
     -+++:                                     -+++-     
       -++++-                               :++++-       
          .++++.                        -++++.          
              -++++.                .++++:              
                  :+++++++++++++++++:                  "#;

const AMD_LOGO_SHORT: &str = r#"
                  :++++++++++++:                  
              -++++.        .++++:              
          .++++.                -++++.          
       -++++-                       :++++-       
  -++-                                 -++-  
.++:                                     :++.
+++                                       +++
+++         ..-=++++++++==-..            +++
+++      -+++=:.          .:=+++=.       +++
+++   .+++:                     :+++.    +++
+++   =++-                         -++=  +++
+++   +++                             +++ +++
+++   =++-                         -++=  +++
+++   .+++:                     :+++.    +++
+++      -+++=:.          .:=+++=.       +++
+++         ..-=++++++++==-..            +++
.++:                                     :++.
  -++-                                 -++-   "#;

/// Print gpufetch output for a GPU
pub fn print_gpufetch(gpu: &GpuInfo, color_scheme: ColorScheme, logo_variant: LogoVariant) -> Result<()> {
    // Determine colors based on vendor and color scheme
    let (logo_color, text_color) = get_colors(gpu, color_scheme);
    
    // Get appropriate ASCII art
    let ascii_art = get_ascii_art(gpu, logo_variant);
    
    if logo_variant != LogoVariant::None {
        // Print ASCII art with info
        print_with_info(gpu, ascii_art, logo_color, text_color)?;
    } else {
        // Print info only
        print_info_only(gpu, text_color)?;
    }
    
    Ok(())
}

/// Get appropriate colors based on vendor and color scheme
fn get_colors(gpu: &GpuInfo, color_scheme: ColorScheme) -> (Color, Color) {
    match color_scheme {
        ColorScheme::System => match gpu.vendor {
            GpuVendor::Nvidia => (Color::Green, Color::White),
            GpuVendor::Amd => (Color::Red, Color::White),
            GpuVendor::Intel => (Color::Cyan, Color::White),
            _ => (Color::White, Color::White),
        },
        ColorScheme::Nvidia => (Color::Green, Color::White),
        ColorScheme::Amd => (Color::Red, Color::White),
        ColorScheme::Intel => (Color::Cyan, Color::White),
        ColorScheme::Custom => (Color::Green, Color::White), // Custom colors would be handled separately
    }
}

/// Get ASCII art for the given GPU vendor and logo variant
fn get_ascii_art(gpu: &GpuInfo, logo_variant: LogoVariant) -> &'static str {
    match logo_variant {
        LogoVariant::None => "",
        LogoVariant::Short => match gpu.vendor {
            GpuVendor::Nvidia => NVIDIA_LOGO_SHORT,
            GpuVendor::Amd => AMD_LOGO_SHORT,
            GpuVendor::Intel => INTEL_LOGO_SHORT,
            _ => NVIDIA_LOGO_SHORT, // Default
        },
        _ => match gpu.vendor {
            GpuVendor::Nvidia => NVIDIA_LOGO,
            GpuVendor::Amd => AMD_LOGO,
            GpuVendor::Intel => INTEL_LOGO,
            _ => NVIDIA_LOGO, // Default
        },
    }
}

/// Print GPU info alongside ASCII art
fn print_with_info(gpu: &GpuInfo, ascii_art: &str, logo_color: Color, text_color: Color) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let ascii_lines: Vec<&str> = ascii_art.lines().collect();
    let info_lines = create_info_lines(gpu);
    
    // Print empty line for spacing
    writeln!(handle)?;
    
    // Determine the maximum number of lines between ASCII art and info
    let max_lines = ascii_lines.len().max(info_lines.len());
    
    // Calculate where to start printing info to center it with the ASCII art
    let info_start = (ascii_lines.len().saturating_sub(info_lines.len())) / 2;
    
    // Print the ASCII art and info
    for i in 0..max_lines {
        // Print ASCII line if available
        if i < ascii_lines.len() {
            write!(handle, "{}", ascii_lines[i].color(logo_color))?;
        } else {
            // Print empty space matching the width of the ASCII art
            if !ascii_lines.is_empty() {
                let max_width = ascii_lines.iter().map(|l| l.len()).max().unwrap_or(0);
                write!(handle, "{}", " ".repeat(max_width))?;
            }
        }
        
        // Print info line if available
        if i >= info_start && i - info_start < info_lines.len() {
            write!(handle, "  {}", info_lines[i - info_start].color(text_color))?;
        }
        
        writeln!(handle)?;
    }
    
    // Print empty line for spacing
    writeln!(handle)?;
    
    Ok(())
}

/// Print GPU info without ASCII art
fn print_info_only(gpu: &GpuInfo, text_color: Color) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let info_lines = create_info_lines(gpu);
    
    // Print empty line for spacing
    writeln!(handle)?;
    
    // Print the info lines
    for line in info_lines {
        writeln!(handle, "{}", line.color(text_color))?;
    }
    
    // Print empty line for spacing
    writeln!(handle)?;
    
    Ok(())
}

/// Create info lines for the given GPU
fn create_info_lines(gpu: &GpuInfo) -> Vec<String> {
    let mut lines = Vec::new();
    
    // GPU name
    lines.push(format!("{}", gpu.name));
    lines.push("-".repeat(gpu.name.len()));
    
    // Basic info
    lines.push(format!("Vendor: {}", gpu.vendor));
    lines.push(format!("Architecture: {}", gpu.architecture));
    if gpu.chip != "Unknown" {
        lines.push(format!("Chip: {}", gpu.chip));
    }
    if let Some(process) = gpu.process_nm {
        lines.push(format!("Process: {} nm", process));
    }
    
    // Memory info
    if let Some(ref memory) = gpu.memory {
        let size_readable = gpu.get_memory_size_readable();
        lines.push(format!("Memory: {} {}", size_readable, memory.memory_type));
        lines.push(format!("Memory Bus: {} bit", memory.bus_width));
    }
    
    // Frequency info
    lines.push(format!("Core Clock: {} MHz", gpu.freq_mhz));
    if gpu.max_freq_mhz > gpu.freq_mhz {
        lines.push(format!("Boost Clock: {} MHz", gpu.max_freq_mhz));
    }
    
    // Compute info
    if let Some(ref topology) = gpu.topology {
        lines.push(gpu.get_compute_units_readable());
        
        match gpu.vendor {
            GpuVendor::Nvidia => {
                if let Some(sm_count) = topology.sm_count {
                    lines.push(format!("Streaming Multiprocessors: {}", sm_count));
                }
                if let Some(tensor_cores) = topology.tensor_cores {
                    lines.push(format!("Tensor Cores: {}", tensor_cores));
                }
                if let Some(rt_cores) = topology.rt_cores {
                    lines.push(format!("RT Cores: {}", rt_cores));
                }
            },
            GpuVendor::Amd => {
                if let Some(compute_units) = Some(topology.compute_units) {
                    lines.push(format!("Compute Units: {}", compute_units));
                }
                if let Some(rops) = topology.rops {
                    lines.push(format!("ROPs: {}", rops));
                }
                if let Some(tmus) = topology.tmus {
                    lines.push(format!("TMUs: {}", tmus));
                }
            },
            GpuVendor::Intel => {
                if let Some(slices) = topology.slices {
                    if let Some(subslices) = topology.subslices {
                        lines.push(format!("Slices: {} (Subslices: {})", slices, subslices));
                    } else {
                        lines.push(format!("Slices: {}", slices));
                    }
                }
            },
            _ => {}
        }
    }
    
    // Cache info
    if let Some(ref cache) = gpu.cache {
        if let Some(l2_size) = cache.l2_size {
            let l2_mb = l2_size as f64 / (1024.0 * 1024.0);
            if l2_mb >= 1.0 {
                lines.push(format!("L2 Cache: {:.1} MB", l2_mb));
            } else {
                let l2_kb = l2_size as f64 / 1024.0;
                lines.push(format!("L2 Cache: {:.0} KB", l2_kb));
            }
        }
        
        if let Some(l3_size) = cache.l3_size {
            let l3_mb = l3_size as f64 / (1024.0 * 1024.0);
            if l3_mb >= 1.0 {
                lines.push(format!("L3 Cache: {:.0} MB", l3_mb));
            } else {
                let l3_kb = l3_size as f64 / 1024.0;
                lines.push(format!("L3 Cache: {:.0} KB", l3_kb));
            }
        }
    }
    
    // Performance info
    if let Some(perf) = gpu.peak_performance_gflops {
        if perf >= 1000.0 {
            lines.push(format!("Peak Performance: {:.2} TFLOPS", perf / 1000.0));
        } else {
            lines.push(format!("Peak Performance: {:.1} GFLOPS", perf));
        }
    }
    
    // Driver info
    if let Some(ref driver) = gpu.driver_version {
        lines.push(format!("Driver: {}", driver));
    }
    
    lines
}
