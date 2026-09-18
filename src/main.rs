use std::fs;
use std::path::PathBuf;

fn read_sys(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn read_sys_file(path: PathBuf) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn format_bytes(bytes: u64) -> String {
    match bytes {
        b if b >= 1_073_741_824 => format!("{:.1} GB", b as f64 / 1_073_741_824.0),
        b if b >= 1_048_576 => format!("{:.1} MB", b as f64 / 1_048_576.0),
        b if b >= 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{} B", b),
    }
}

fn pci_class(code: &str) -> &'static str {
    match code {
        "0001" => "VGA-Compatible",
        "0100" => "RAID", "0106" => "SATA", "0107" => "SAS", "0108" => "NVMe",
        "0200" => "Ethernet", "0300" => "VGA", "0302" => "3D Controller",
        "0401" => "Multimedia Audio", "0600" => "Host Bridge", "0601" => "ISA Bridge",
        "0603" => "PCI-PCI Bridge", "0805" => "SATA (AHCI)", "1180" => "DMA Controller",
        _ => "Device",
    }
}

fn entry_name(path: &PathBuf) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn sysfs_entries(path: &str) -> Vec<PathBuf> {
    fs::read_dir(path)
        .map(|entries| entries.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default()
}

struct Hardware {
    cpu_model: String,
    cpu_cores: usize,
    cpu_mhz: String,
    ram_total: String,
    ram_free: String,
    ram_avail: String,
    hostname: String,
    storage: Vec<(String, String, String)>,
    network: Vec<(String, String, String, String)>,
    usb: Vec<(String, String, String)>,
    pci: Vec<(String, String, String)>,
    gpu: Vec<(String, String, String)>,
    modules: usize,
}

impl Hardware {
    fn scan() -> Self {
        let cpu_info = read_sys("/proc/cpuinfo");
        let mem_info = read_sys("/proc/meminfo");

        let mut cpu_model = String::new();
        let mut cpu_cores = 0;
        let mut cpu_mhz = String::new();

        for line in cpu_info.lines() {
            if line.starts_with("model name") {
                cpu_model = line.split(": ").nth(1).unwrap_or("").to_string();
            }
            if line.starts_with("cpu MHz") {
                cpu_mhz = line.split(": ").nth(1).unwrap_or("").to_string();
            }
            if line.starts_with("processor") {
                cpu_cores += 1;
            }
        }

        let mut mem_lines = mem_info.lines();
        let ram_total = mem_lines.next().unwrap_or("").split_whitespace().nth(1).unwrap_or("0").parse::<u64>().unwrap_or(0);
        let ram_free = mem_lines.next().unwrap_or("").split_whitespace().nth(1).unwrap_or("0").parse::<u64>().unwrap_or(0);
        let ram_avail = mem_lines.next().unwrap_or("").split_whitespace().nth(1).unwrap_or("0").parse::<u64>().unwrap_or(0);

        // Storage
        let mut storage = Vec::new();
        for entry in sysfs_entries("/sys/block") {
            let name = entry_name(&entry);
            if name.starts_with("loop") || name.starts_with("ram") { continue; }
            let size_sectors = read_sys_file(entry.join("size")).parse::<u64>().unwrap_or(0);
            let size = format_bytes(size_sectors * 512);
            let model = read_sys_file(entry.join("device/model"));
            storage.push((name, size, model));
        }

        // Network
        let mut network = Vec::new();
        for entry in sysfs_entries("/sys/class/net") {
            let name = entry_name(&entry);
            if name == "lo" { continue; }
            let mac = read_sys_file(entry.join("address"));
            let speed = read_sys_file(entry.join("speed"));
            let state = read_sys_file(entry.join("operstate"));
            network.push((name, mac, speed, state));
        }

        // USB
        let mut usb = Vec::new();
        for entry in sysfs_entries("/sys/bus/usb/devices") {
            let name = entry_name(&entry);
            if !name.contains('-') || name.contains(':') { continue; }
            let product = read_sys_file(entry.join("product"));
            let vendor = read_sys_file(entry.join("idVendor"));
            let id_product = read_sys_file(entry.join("idProduct"));
            if !product.is_empty() {
                usb.push((format!("{}:{}", vendor, id_product), product, String::new()));
            }
        }

        // PCI
        let mut pci = Vec::new();
        if let Ok(info) = fs::read_to_string("/proc/bus/pci/devices") {
            for line in info.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let class = pci_class(parts[2]);
                    pci.push((parts[0].to_string(), parts[1].to_string(), class.to_string()));
                }
            }
        }

        // GPU
        let mut gpu = Vec::new();
        for entry in sysfs_entries("/sys/class/drm") {
            let name = entry_name(&entry);
            if name.starts_with("card") && !name.contains('-') {
                let vendor = read_sys_file(entry.join("device/vendor"));
                let device = read_sys_file(entry.join("device/device"));
                gpu.push((name, vendor, device));
            }
        }

        let modules = read_sys("/proc/modules").lines().count();

        Hardware {
            cpu_model, cpu_cores, cpu_mhz,
            ram_total: format_bytes(ram_total * 1024),
            ram_free: format_bytes(ram_free * 1024),
            ram_avail: format_bytes(ram_avail * 1024),
            hostname: read_sys("/proc/sys/kernel/hostname"),
            storage, network, usb, pci, gpu, modules,
        }
    }

    fn print(&self) {
        println!("═══════════════════════════════════════════════════════");
        println!("  Hardware Inventory — {}", self.hostname);
        println!("═══════════════════════════════════════════════════════");

        println!("\n▸ CPU");
        println!("  Modelo:     {}", self.cpu_model);
        println!("  Cores:      {}", self.cpu_cores);
        println!("  Frequência: {} MHz", self.cpu_mhz);

        println!("\n▸ Memória");
        println!("  Total:      {}", self.ram_total);
        println!("  Livre:      {}", self.ram_free);
        println!("  Disponível: {}", self.ram_avail);

        println!("\n▸ Armazenamento");
        for (name, size, model) in &self.storage {
            println!("  {:<12} {:>10}  {}", name, size, model);
        }

        println!("\n▸ Rede");
        for (name, mac, speed, state) in &self.network {
            println!("  {:<12} MAC: {}  {} Mb/s  [{}]", name, mac, speed, state);
        }

        println!("\n▸ USB");
        for (id, product, _) in &self.usb {
            println!("  {}  {}", id, product);
        }

        println!("\n▸ PCI ({} dispositivos)", self.pci.len());
        for (bdf, vendprod, class) in &self.pci {
            println!("  {} {} — {}", bdf, vendprod, class);
        }

        println!("\n▸ GPU");
        for (name, vendor, device) in &self.gpu {
            println!("  {}  vendor:{} device:{}", name, vendor, device);
        }

        println!("\n▸ Kernel");
        println!("  {} módulos carregados", self.modules);

        println!("\n═══════════════════════════════════════════════════════");
    }
}

fn main() {
    let hw = Hardware::scan();
    hw.print();
}
