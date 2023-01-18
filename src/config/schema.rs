use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RustbaseConfig {
    pub net: Net,
    pub database: Database,
    pub auth: Option<Auth>,
    pub tls: Option<Tls>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Net {
    pub host: String,
    pub port: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Database {
    pub path: std::path::PathBuf,
    pub cache_size: usize,
    pub threads: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TlsType {
    Required,
    Optional,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tls {
    pub ca_file: String,
    pub pem_key_file: String,
    #[serde(rename = "type")]
    pub type_: TlsType,
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

    format!("{:.2} {} ({} Bytes)", m_size, unit, size)
}

const IDENT: &str = "    ";

impl Display for RustbaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.net)?;
        writeln!(f, "{}", self.database)?;

        if let Some(auth) = &self.auth {
            writeln!(f, "{}", auth)?;
        }

        if let Some(tls) = &self.tls {
            writeln!(f, "{}", tls)?;
        }
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

impl Display for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "\nAuthentication configuration: ")?;
        writeln!(f, "{}username: {}", IDENT, self.username.cyan())?;
        let mut password = self.password.clone();
        password.replace_range(0..password.len(), "*".repeat(password.len()).as_str());

        writeln!(f, "{}password: {}", IDENT, password.cyan())?;
        Ok(())
    }
}

impl Display for Tls {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "\nTLS configuration: ")?;
        writeln!(f, "{}ca file: {}", IDENT, self.ca_file.cyan())?;
        writeln!(f, "{}pem key file: {}", IDENT, self.pem_key_file.cyan())?;
        writeln!(f, "{}type: {}", IDENT, self.type_.to_string().cyan())?;
        Ok(())
    }
}

impl Display for TlsType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TlsType::Required => write!(f, "required"),
            TlsType::Optional => write!(f, "optional"),
        }
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
        writeln!(
            f,
            "{}path: {}",
            IDENT,
            self.path.display().to_string().cyan()
        )?;
        Ok(())
    }
}
