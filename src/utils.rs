use std::fs;
use std::path::Path;
use std::process::Command;

/// Format file sizes in a human-readable format
pub fn format_size(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.2} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.2} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", size_bytes)
    }
}

/// Read a file's contents as a string, returning an empty string if an error occurs
pub fn read_file_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Run a command and get its output as a string
pub fn run_command(command: &str, args: &[&str]) -> Option<String> {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
}

/// Check if a command is available in the system
pub fn is_command_available(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Convert a hexadecimal string to a u64
pub fn hex_to_u64(hex: &str) -> Option<u64> {
    let hex = hex.trim().trim_start_matches("0x");
    u64::from_str_radix(hex, 16).ok()
}

/// Get the terminal width
pub fn get_terminal_width() -> usize {
    if let Some(dims) = term_size::dimensions() {
        dims.0
    } else {
        80 // Default terminal width
    }
}

/// Check if running in a terminal with color support
pub fn has_color_support() -> bool {
    std::env::var("NO_COLOR").is_err() && 
    std::env::var("TERM").map(|term| term != "dumb").unwrap_or(true)
}

/// Find a file with the given name in a directory and its subdirectories
pub fn find_file_in_dir(dir: &Path, filename: &str) -> Option<String> {
    if !dir.exists() || !dir.is_dir() {
        return None;
    }
    
    let entries = fs::read_dir(dir).ok()?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(filename) {
                return path.to_str().map(|s| s.to_owned());
            } else if path.is_dir() {
                if let Some(found) = find_file_in_dir(&path, filename) {
                    return Some(found);
                }
            }
        }
    }
    
    None
}
