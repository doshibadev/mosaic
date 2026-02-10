use colored::*;
use console::Term;
use std::fmt::Display;

pub struct Logger;

impl Logger {
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
            let colored_line = if line.contains('/') || line.contains('|') {
                line.truecolor(125, 59, 155).bold().to_string()
            } else {
                line.truecolor(14, 173, 221).bold().to_string()
            };
            println!("{:^width$}", colored_line, width = width);
        }
        println!();
    }

    pub fn info<T: Display>(msg: T) {
        println!("{} {}", "•".truecolor(14, 173, 221).bold(), msg);
    }

    pub fn success<T: Display>(msg: T) {
        println!("{} {}", "✔".green().bold(), msg);
    }

    pub fn error<T: Display>(msg: T) {
        println!("{} {}", "✖".red().bold(), msg);
    }

    pub fn header<T: Display>(msg: T) {
        println!(
            "\n{}",
            msg.to_string().truecolor(125, 59, 155).bold().underline()
        );
    }

    pub fn command<T: Display>(cmd: &str, msg: T) {
        println!(
            "{} {}",
            cmd.truecolor(125, 59, 155).bold(),
            msg.to_string().dimmed()
        );
    }

    pub fn highlight<T: Display>(msg: T) -> String {
        msg.to_string().truecolor(14, 173, 221).bold().to_string()
    }

    pub fn brand_text<T: Display>(msg: T) -> String {
        msg.to_string().truecolor(125, 59, 155).bold().to_string()
    }

    pub fn dim<T: Display>(msg: T) -> String {
        msg.to_string().dimmed().to_string()
    }
}
