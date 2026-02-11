use colored::*;
use console::Term;
use std::fmt::Display;

/// Logging utilities with branded colors and emoji.
/// All methods are static because we never instantiate this—it's just a namespace for logging functions.
/// Color scheme: blue (14, 173, 221) for info, purple (125, 59, 155) for brand.
/// Yes, I hardcoded the RGB values instead of using constants. Don't @ me.
pub struct Logger;

impl Logger {
    /// Prints the mosaic banner at startup.
    /// Centers it on the terminal, colors the slashes/pipes purple and the letters blue.
    /// Looks cool. Makes people think we're professional. It's working so far.
    pub fn banner() {
        let term = Term::stdout();
        let width = term.size().1 as usize;

        let banner = r#"
    __  ___                      _      
   /  |/  /___  _________ ______(_)____ 
  / /|_/ / __ \/ ___/ __ `/ __  / / ___/
 / /  / / /_/ (__  ) /_/ / /_/ / / /__  
/_/  /_/\____/____/\__,_/\__,_/_/\___/  
"#;

        for line in banner.lines() {
            if line.trim().is_empty() {
                continue;
            }
            // Color slashes/pipes purple to match the brand, everything else blue.
            // This is petty and unnecessary but I like how it looks.
            let colored_line = if line.contains('/') || line.contains('|') {
                line.truecolor(125, 59, 155).bold().to_string()
            } else {
                line.truecolor(14, 173, 221).bold().to_string()
            };
            println!("{:^width$}", colored_line, width = width);
        }
        println!();
    }

    /// Prints an info message with a blue bullet point.
    /// Use this for general information that doesn't fit the other categories.
    pub fn info<T: Display>(msg: T) {
        println!("{} {}", "•".truecolor(14, 173, 221).bold(), msg);
    }

    /// Prints a success message with a green checkmark.
    /// Feels good when operations complete. Users expect this emoji.
    pub fn success<T: Display>(msg: T) {
        println!("{} {}", "✔".green().bold(), msg);
    }

    /// Prints an error message with a red X.
    /// Something went wrong and the user needs to know.
    pub fn error<T: Display>(msg: T) {
        println!("{} {}", "✖".red().bold(), msg);
    }

    /// Prints a warning with a yellow warning symbol.
    /// Use sparingly—overuse makes people ignore warnings.
    pub fn warn<T: Display>(msg: T) {
        println!("{} {}", "⚠".yellow().bold(), msg);
    }

    /// Prints a section header in purple with underline.
    /// Breaks up the output so users can follow along.
    /// The leading newline prevents it from running into previous output.
    pub fn header<T: Display>(msg: T) {
        println!(
            "\n{}",
            msg.to_string().truecolor(125, 59, 155).bold().underline()
        );
    }

    /// Prints a command being executed (like a label for what's happening).
    /// Format: "command description" where command is purple and description is dimmed.
    /// Looks nice in output logs.
    pub fn command<T: Display>(cmd: &str, msg: T) {
        println!(
            "{} {}",
            cmd.truecolor(125, 59, 155).bold(),
            msg.to_string().dimmed()
        );
    }

    /// Returns a string colored in brand blue (for inline use).
    /// Used in formatted strings where you need to highlight something.
    pub fn highlight<T: Display>(msg: T) -> String {
        msg.to_string().truecolor(14, 173, 221).bold().to_string()
    }

    /// Returns a string colored in brand purple (for inline use).
    /// Used for package names, versions, and other important metadata.
    pub fn brand_text<T: Display>(msg: T) -> String {
        msg.to_string().truecolor(125, 59, 155).bold().to_string()
    }

    /// Returns a dimmed string (less important text).
    /// Use for secondary information or file paths in descriptions.
    pub fn dim<T: Display>(msg: T) -> String {
        msg.to_string().dimmed().to_string()
    }
}
