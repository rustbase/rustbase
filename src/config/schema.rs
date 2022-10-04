use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone)]
pub struct RustbaseConfig {
    pub net: Net,
    pub database: Database,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Net {
    pub host: String,
    pub port: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    pub path: String,
    pub cache_size: usize,
}

fn parse_size_to_string(size: usize) -> String {
    let mut m_size = size as f64;
    let mut unit = "B";

    if m_size > 1024.0 {
        m_size /= 1024.0;
        unit = "KB";
    }

    if m_size > 1024.0 {
        m_size /= 1024.0;
        unit = "MB";
    }

    if m_size > 1024.0 {
        m_size /= 1024.0;
        unit = "GB";
    }

    if m_size > 1024.0 {
        m_size /= 1024.0;
        unit = "TB";
    }

    format!("{:.2} {} ({} B)", m_size, unit, size)
}

const IDENT: &str = "    ";

impl Display for RustbaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.net)?;
        writeln!(f, "{}", self.database)?;
        Ok(())
    }
}

impl Display for Net {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "\nNetwork configuration: ")?;
        writeln!(f, "{}port: {}", IDENT, self.port.cyan())?;
        writeln!(f, "{}host: {}", IDENT, self.host.cyan())?;
        Ok(())
    }
}

impl Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "\nDatabase configuration: ")?;
        writeln!(
            f,
            "{}cache_size: {}",
            IDENT,
            parse_size_to_string(self.cache_size).cyan()
        )?;
        writeln!(f, "{}path: {}", IDENT, self.path.cyan())?;
        Ok(())
    }
}
