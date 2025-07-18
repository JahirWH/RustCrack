use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;


/// Punto de entrada principal
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

/// Orquesta el flujo principal del programa
fn run() -> io::Result<()> {
        
    println!("RustCrack base");
    let path = "passwords.txt";
    println!("Se usara: {} Para el hasheo de la contraseñas", path);

    for line in read_lines(path)? {
        let line = line?;
        let hash = hash_password(&line);
        println!("Contraseña: {} | Hash: {}", line, hash);
    }
    Ok(())
}

fn scan_wifi_ssids() -> Vec<String> {
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "SSID", "dev", "wifi"])
        .output()
        .expect("Failed to execute nmcli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn scan_wifi_ssids_and_bssids() -> Vec<(String, String)> {
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "SSID,BSSID", "dev", "wifi"])
        .output()
        .expect("Failed to execute nmcli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            let ssid = parts.next()?.trim().to_string();
            let bssid = parts.next()?.trim().to_string();
            if !ssid.is_empty() && !bssid.is_empty() {
                Some((ssid, bssid))
            } else {
                None
            }
        })
        .collect()
}

/// Lee líneas de un archivo y retorna un iterador
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Hashea una contraseña usando SHA256
fn hash_password(password: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

