use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorScheme {
    /// Use system colors
    System,
    /// NVIDIA green colors
    Nvidia,
    /// AMD red colors
    Amd,
    /// Intel blue colors
    Intel,
    /// Custom color scheme (format: "r,g,b:r,g,b:r,g,b:r,g,b")
    Custom,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum LogoVariant {
    /// Regular sized logo
    Normal,
    /// Short variant of the logo
    Short,
    /// Long variant of the logo
    Long,
    /// No logo, information only
    None,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Which GPU to display (default: 0, -1 for all GPUs)
    #[arg(short, long, default_value_t = 0)]
    pub gpu_index: i32,

    /// Just list available GPUs and exit
    #[arg(short = 'l', long)]
    pub list_only: bool,

    /// Color scheme to use
    #[arg(short, long, value_enum, default_value_t = ColorScheme::System)]
    pub color_scheme: ColorScheme,

    /// Custom colors in RGB format: "r,g,b:r,g,b:r,g,b:r,g,b" 
    /// (4 colors: logo primary, logo secondary, text primary, text secondary)
    #[arg(short = 'C', long)]
    pub custom_colors: Option<String>,

    /// Logo size variant
    #[arg(short = 'L', long, value_enum, default_value_t = LogoVariant::Normal)]
    pub logo_variant: LogoVariant,

    /// Display detailed information
    #[arg(short, long)]
    pub detailed: bool,

    /// Disable color output
    #[arg(long)]
    pub no_color: bool,

    /// Enable verbose output with debugging information
    #[arg(short, long)]
    pub verbose: bool,
}
