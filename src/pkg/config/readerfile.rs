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
    pub frontend_host_rule: Option<String>,
    pub backends: HashMap<String, Backend>,
    pub destination: String,
}

pub fn parse_config(file_path: &str) -> ProxyConfig {
    let file = File::open(file_path).expect("Failed to open config file");
    let reader = BufReader::new(file);

    let mut frontend_bind = String::new();
    let mut frontend_host_rule = None;
    let mut backends = HashMap::new();
    let mut current_backend = None;
    let mut destination = String::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "frontend" if parts.len() > 1 => {
                frontend_bind = parts[1].to_string();
                current_backend = None; // Reset backend context
            }
            "use_backend" => {
                // Extract the host rule from `{ req.hdr(host) -i example.com }`
                if let Some(start) = line.find("{ req.hdr(host) -i ") {
                    let start_index = start + "{ req.hdr(host) -i ".len();
                    if let Some(end_index) = line[start_index..].find(" }") {
                        frontend_host_rule = Some(line[start_index..start_index + end_index].to_string());
                    }
                }
            }
            "backend" if parts.len() > 1 => {
                current_backend = Some(parts[1].to_string());
                backends.insert(parts[1].to_string(), Backend { servers: vec![] });
            }
            "server" if parts.len() > 2 => {
                if let Some(backend_name) = &current_backend {
                    if let Some(backend) = backends.get_mut(backend_name) {
                        backend.servers.push(parts[2].to_string());
                        destination = parts[2].to_string();
                    }
                }
            }
            _ => {}
        }
    }

    ProxyConfig {
        frontend_bind,
        frontend_host_rule,
        backends,
        destination,
    }
}

pub fn load_config(file_path: &str, host: &str) -> Result<ProxyConfig, &'static str> {
    let config = parse_config(file_path);

    if config.frontend_bind.is_empty() || config.backends.is_empty() {
        return Err("Invalid config");
    }
    let frontend_match = config
        .frontend_host_rule
        .as_ref()
        .map_or(false, |rule| rule == host);

    let backend_match = config.backends.contains_key(host);

    let host_exists = config
        .backends
        .values()
        .any(|backend| backend.servers.contains(&host.to_string()));

    Ok(config)
}
