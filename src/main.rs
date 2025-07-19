use std::fs::{OpenOptions, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::Command;
use std::net::Ipv4Addr;

fn main() -> io::Result<()> {
    println!("bienvenido a mi programa para escanear redes WiFi y guardarlas en un archivo.");
    println!("Presiona Enter para comenzar, o D para salir.");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().eq_ignore_ascii_case("d") {
        println!("Programa finalizado.");
        return Ok(());
    }

    let mut ssids_bssids = scan_wifi_nmcli_bssid();
    let mut iwlist_ssids = scan_wifi_iwlist();
    for ssid in iwlist_ssids {
        if !ssids_bssids.iter().any(|(s, _)| s == &ssid) {
            ssids_bssids.push((ssid, String::from("(sin BSSID)")));
        }
    }
    ssids_bssids.sort_by(|a, b| a.0.cmp(&b.0));
    ssids_bssids.dedup_by(|a, b| a.0 == b.0);
    save_to_txt_bssid("wifis.txt", &ssids_bssids)?;
    println!("Redes WiFi guardadas en wifis.txt\n");

    let ssids_bssids = load_ssids_bssid("wifis.txt")?;
    if ssids_bssids.is_empty() {
        println!("No se encontraron redes WiFi en wifis.txt");
        return Ok(());
    }
    println!("Redes WiFi disponibles:");
    for (idx, (ssid, bssid)) in ssids_bssids.iter().enumerate() {
        println!("{}. {}  [BSSID: {}]", idx + 1, ssid, bssid);
    }
    println!("\nSelecciona el número de la red WiFi a escanear (por ejemplo, 1): ");
    let mut sel = String::new();
    io::stdin().read_line(&mut sel)?;
    let sel: usize = match sel.trim().parse() {
        Ok(num) if num > 0 && num <= ssids_bssids.len() => num,
        _ => {
            println!("Selección inválida.");
            return Ok(());
        }
    };
    let (selected_ssid, selected_bssid) = &ssids_bssids[sel - 1];
    println!("\nSeleccionaste: {}  [BSSID: {}]", selected_ssid, selected_bssid);

    let current_ssid = get_current_ssid();
    let network = if let Some(cur_ssid) = &current_ssid {
        if cur_ssid == selected_ssid {
            if let Some((ip, mask)) = get_ip_and_mask() {
                if let Some(subnet) = calc_subnet(&ip, mask) {
                    println!("Detectada subred actual: {}", subnet);
                    subnet
                } else {
                    println!("No se pudo calcular la subred automáticamente.");
                    ask_subnet()
                }
            } else {
                println!("No se pudo detectar la subred automáticamente.");
                ask_subnet()
            }
        } else {
            println!("No estás conectado a esa red WiFi. Debes ingresar la subred manualmente.");
            ask_subnet()
        }
    } else {
        println!("No se pudo detectar el SSID actual. Ingresa la subred manualmente.");
        ask_subnet()
    };

    let hosts = ping_sweep(&network);
    println!("\nDispositivos activos en la red:");
    for host in hosts {
        let mac = get_mac(&host);
        println!("{} - MAC: {}", host, mac.unwrap_or_else(|| "No encontrada".to_string()));
    }
    Ok(())
}

fn scan_wifi_nmcli() -> Vec<String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "SSID", "dev", "wifi"])
        .output();
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        }
        Err(_) => {
            println!("No se pudo ejecutar nmcli");
            vec![]
        }
    }
}

fn scan_wifi_iwlist() -> Vec<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("iwlist scan 2>/dev/null | grep 'ESSID:'")
        .output();
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let start = line.find('"')? + 1;
                    let end = line.rfind('"')?;
                    if end > start {
                        Some(line[start..end].to_string())
                    } else {
                        None
                    }
                })
                .filter(|ssid| !ssid.is_empty())
                .collect()
        }
        Err(_) => {
            println!("No se pudo ejecutar iwlist");
            vec![]
        }
    }
}

fn scan_wifi_nmcli_bssid() -> Vec<(String, String)> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "SSID,BSSID", "dev", "wifi"])
        .output();
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let mut parts = line.splitn(2, ':');
                    let ssid = parts.next()?.trim().to_string();
                    let bssid = parts.next()?.trim().to_string();
                    if !ssid.is_empty() {
                        Some((ssid, if !bssid.is_empty() { bssid } else { String::from("(sin BSSID)") }))
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(_) => {
            println!("No se pudo ejecutar nmcli para BSSID");
            vec![]
        }
    }
}

fn save_to_txt(filename: &str, ssids: &[String]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)?;
    for ssid in ssids {
        writeln!(file, "{}", ssid)?;
    }
    Ok(())
}

fn save_to_txt_bssid(filename: &str, ssids_bssids: &[(String, String)]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)?;
    for (ssid, bssid) in ssids_bssids {
        writeln!(file, "{};{}", ssid, bssid)?;
    }
    Ok(())
}

fn load_ssids(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().filter_map(Result::ok).filter(|l| !l.trim().is_empty()).collect())
}

fn load_ssids_bssid(filename: &str) -> io::Result<Vec<(String, String)>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter_map(|l| {
            let mut parts = l.splitn(2, ';');
            let ssid = parts.next()?.to_string();
            let bssid = parts.next().unwrap_or("").to_string();
            if !ssid.trim().is_empty() {
                Some((ssid, bssid))
            } else {
                None
            }
        })
        .collect())
}

fn get_current_ssid() -> Option<String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
        .ok()?;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(rest) = line.strip_prefix("yes:") {
            return Some(rest.to_string());
        }
    }
    None
}

fn get_ip_and_mask() -> Option<(String, u8)> {
    let output = Command::new("ip")
        .args(["addr", "show"])
        .output()
        .ok()?;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        if line.starts_with("inet ") && !line.starts_with("inet 127.") {
            // inet 192.168.1.10/24 ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                if let Some((ip, mask)) = parts[1].split_once('/') {
                    if let Ok(mask) = mask.parse::<u8>() {
                        return Some((ip.to_string(), mask));
                    }
                }
            }
        }
    }
    None
}

fn calc_subnet(ip: &str, mask: u8) -> Option<String> {
    let ip: Ipv4Addr = ip.parse().ok()?;
    let mask = u32::MAX.checked_shl((32 - mask) as u32).unwrap_or(0);
    let ip_u32 = u32::from(ip);
    let net = ip_u32 & mask;
    let net_addr = Ipv4Addr::from(net);
    Some(format!("{}/{}", net_addr, mask.count_ones()))
}

fn ask_subnet() -> String {
    println!("Ingresa la subred a escanear (ejemplo: 192.168.1.0/24): ");
    let mut network = String::new();
    io::stdin().read_line(&mut network).unwrap();
    network.trim().to_string()
}

fn ping_sweep(network: &str) -> Vec<String> {
    let mut alive_hosts = Vec::new();
    let parts: Vec<&str> = network.split('/').collect();
    if parts.len() != 2 {
        return alive_hosts;
    }
    let base_ip: Ipv4Addr = match parts[0].parse() {
        Ok(ip) => ip,
        Err(_) => return alive_hosts,
    };
    let mask: u8 = match parts[1].parse() {
        Ok(m) => m,
        Err(_) => return alive_hosts,
    };
    let hosts = 2u32.pow((32 - mask) as u32) - 2; // -2 para red y broadcast
    let base = u32::from(base_ip);
    println!("Escaneando red {} (esto puede tardar)...", network);
    for i in 1..=hosts {
        let ip = Ipv4Addr::from(base + i);
        let output = Command::new("ping")
            .args(["-c", "1", "-W", "1", &ip.to_string()])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                alive_hosts.push(ip.to_string());
            }
        }
    }
    alive_hosts
}

fn get_mac(ip: &str) -> Option<String> {
    let output = Command::new("arp")
        .args(["-n", ip])
        .output()
        .ok()?;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.contains(ip) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.contains(":") && part.len() == 17 {
                    return Some(part.to_string());
                }
            }
        }
    }
    None
}


