use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

fn main() {
    println!("RustCrack (base)");
    // Ruta al archivo de contraseñas
    let path = "passwords.txt";
    if let Ok(lines) = read_lines(path) {
        for line in lines.flatten() {
            let hash = hash_password(&line);
            println!("Contraseña: {} | Hash: {}", line, hash);
        }
    } else {
        println!("No se pudo abrir el archivo");
    }
}

// Función para leer líneas de un archivo
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// Función base 
fn hash_password(password: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
