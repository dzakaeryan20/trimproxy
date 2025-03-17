use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Backend {
    pub servers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub frontend_bind: String,
    pub backends: HashMap<String, Backend>,
}

pub fn parse_config(file_path: &str) -> ProxyConfig {
    let file = File::open(file_path).expect("Failed to open config file");
    let reader = BufReader::new(file);
    
    let mut frontend_bind = String::new();
    let mut backends = HashMap::new();
    let mut current_backend = None;

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "frontend" => frontend_bind = String::new(), // Reset frontend
            "bind" => frontend_bind = parts[1].to_string(),
            "backend" => {
                current_backend = Some(parts[1].to_string());
                backends.insert(parts[1].to_string(), Backend { servers: vec![] });
            }
            "server" => {
                if let Some(backend_name) = &current_backend {
                    if let Some(backend) = backends.get_mut(backend_name) {
                        backend.servers.push(parts[2].to_string());
                    }
                }
            }
            _ => {}
        }
    }

    ProxyConfig { frontend_bind, backends }
}

pub fn load_config(file_path: &str) -> Result<ProxyConfig, &'static str> {
    let config = parse_config(file_path);
    if config.frontend_bind.is_empty() || config.backends.is_empty() {
        Err("Invalid config")
    } else {
        Ok(config)
    }
}