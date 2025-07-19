import subprocess
import ipaddress
import re

def scan_nmcli():
    try:
        result = subprocess.run(
            ["nmcli", "-t", "-f", "SSID", "dev", "wifi"],
            capture_output=True, text=True, check=True
        )
        ssids = [line.strip() for line in result.stdout.splitlines() if line.strip()]
        return ssids
    except Exception as e:
        print(f"Error ejecutando nmcli: {e}")
        return []

def scan_iwlist():
    try:
        result = subprocess.run(
            "iwlist scan 2>/dev/null | grep 'ESSID:'",
            shell=True, capture_output=True, text=True, check=True
        )
        ssids = []
        for line in result.stdout.splitlines():
            if '"' in line:
                ssid = line.split('"')[1]
                if ssid:
                    ssids.append(ssid)
        return ssids
    except Exception as e:
        print(f"Error ejecutando iwlist: {e}")
        return []

def save_ssids(filename, ssids):
    with open(filename, "w") as f:
        for ssid in sorted(set(ssids)):
            f.write(ssid + "\n")

def load_ssids(filename):
    try:
        with open(filename, "r") as f:
            return [line.strip() for line in f if line.strip()]
    except Exception as e:
        print(f"No se pudo leer {filename}: {e}")
        return []

def get_current_ssid():
    try:
        result = subprocess.run(["nmcli", "-t", "-f", "active,ssid", "dev", "wifi"], capture_output=True, text=True)
        for line in result.stdout.splitlines():
            if line.startswith("yes:"):
                return line.split(":", 1)[1]
    except Exception as e:
        print(f"Error obteniendo SSID actual: {e}")
    return None

def get_ip_and_mask():
    try:
        result = subprocess.run(["ip", "addr", "show"], capture_output=True, text=True)
        for line in result.stdout.splitlines():
            line = line.strip()
            if line.startswith("inet ") and not line.startswith("inet 127."):
                # Ejemplo: inet 192.168.1.10/24 brd 192.168.1.255 scope global dynamic noprefixroute wlp2s0
                match = re.search(r"inet (\d+\.\d+\.\d+\.\d+)/(\d+)", line)
                if match:
                    ip = match.group(1)
                    mask = int(match.group(2))
                    return ip, mask
    except Exception as e:
        print(f"Error obteniendo IP y máscara: {e}")
    return None, None

def calc_subnet(ip, mask):
    try:
        net = ipaddress.ip_network(f"{ip}/{mask}", strict=False)
        return str(net)
    except Exception as e:
        print(f"Error calculando subred: {e}")
        return None

def ping_sweep(network):
    alive_hosts = []
    net = ipaddress.ip_network(network, strict=False)
    print(f"Escaneando red {network} (esto puede tardar)...")
    for ip in net.hosts():
        ip_str = str(ip)
        result = subprocess.run(['ping', '-c', '1', '-W', '1', ip_str], stdout=subprocess.DEVNULL)
        if result.returncode == 0:
            alive_hosts.append(ip_str)
    return alive_hosts

def get_mac(ip):
    try:
        result = subprocess.run(['arp', '-n', ip], capture_output=True, text=True)
        for line in result.stdout.splitlines():
            if ip in line:
                parts = line.split()
                for part in parts:
                    if ':' in part and len(part) == 17:
                        return part
    except Exception as e:
        print(f"Error obteniendo MAC de {ip}: {e}")
    return None

def main():
    print("Escaneando redes WiFi...")
    ssids = scan_nmcli()
    ssids += scan_iwlist()
    save_ssids("wifis.txt", ssids)
    print("Redes WiFi guardadas en wifis.txt\n")

    ssids = load_ssids("wifis.txt")
    if not ssids:
        print("No se encontraron redes WiFi en wifis.txt")
        return
    print("Redes WiFi disponibles:")
    for idx, ssid in enumerate(ssids):
        print(f"{idx+1}. {ssid}")
    try:
        sel = int(input("\nSelecciona el número de la red WiFi a escanear (por ejemplo, 1): "))
        if not (1 <= sel <= len(ssids)):
            print("Selección inválida.")
            return
    except Exception:
        print("Entrada inválida.")
        return
    selected_ssid = ssids[sel-1]
    print(f"\nSeleccionaste: {selected_ssid}")

    current_ssid = get_current_ssid()
    if current_ssid and selected_ssid == current_ssid:
        ip, mask = get_ip_and_mask()
        if ip and mask:
            network = calc_subnet(ip, mask)
            print(f"Detectada subred actual: {network}")
        else:
            print("No se pudo detectar la subred automáticamente.")
            network = input("Ingresa la subred a escanear (ejemplo: 192.168.1.0/24): ")
    else:
        print("No estás conectado a esa red WiFi. Debes ingresar la subred manualmente.")
        network = input("Ingresa la subred a escanear (ejemplo: 192.168.1.0/24): ")

    hosts = ping_sweep(network)
    print("\nDispositivos activos en la red:")
    for host in hosts:
        mac = get_mac(host)
        print(f"{host} - MAC: {mac if mac else 'No encontrada'}")

if __name__ == "__main__":
    main()
    