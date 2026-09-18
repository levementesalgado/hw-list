# hw-list

Lista todos os componentes de hardware do sistema via Linux sysfs/procfs.

## Compilar

```bash
cargo build --release
./target/release/hw-list
```

## Saída

```
═══════════════════════════════════════════════════════
  Hardware Inventory — hostname
═══════════════════════════════════════════════════════

▸ CPU
  Modelo:     Intel(R) Core(TM) i5 CPU M 540 @ 2.53GHz
  Cores:      4
  Frequência: 2792.933 MHz

▸ Memória
  Total:      5.6 GB
  Livre:      267.2 MB
  Disponível: 1.3 GB

▸ Armazenamento
  sr0           1024.0 MB  DVDRAM GU40N
  sda            111.8 GB  CT120BX500SSD1

▸ Rede
  wlp3s0       MAC: 18:3d:a2:07:33:d0  [up]

▸ USB
  147e:2016  Biometric Coprocessor

▸ PCI (27 dispositivos)
▸ GPU
▸ Kernel
```

## Fontes

- `/proc/cpuinfo` — CPU
- `/proc/meminfo` — Memória
- `/sys/block` — Discos
- `/sys/class/net` — Rede
- `/sys/bus/usb/devices` — USB
- `/proc/bus/pci/devices` — PCI
- `/sys/class/drm` — GPU
- `/proc/modules` — Módulos kernel
