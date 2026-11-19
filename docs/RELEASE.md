# NoxL Release Build

All 16 binaries compile in release mode on Arch Linux.

## Binary sizes

| Binary | Size | Role |
|---|---|---|
| `noxl-compositor` | 6.3 MB | Wayland compositor |
| `noxl-panel` | 2.9 MB | Taskbar |
| `noxl-launcher` | 2.6 MB | App launcher |
| `noxl-notifications` | 4.3 MB | Notification centre |
| `noxl-lockscreen` | 2.5 MB | Lock screen |
| `noxl-settings` | 2.4 MB | Control Centre |
| `noxl-shell` | 1.9 MB | Session supervisor |
| `noxl-widgets` | 2.6 MB | Widget engine |
| `noxl-terminal` | 2.1 MB | Terminal emulator |
| `noxl-files` | 2.1 MB | File manager |
| `noxl-text` | 2.1 MB | Text editor |
| `noxl-monitor` | 2.3 MB | System monitor |
| `noxl-screenshot` | 2.4 MB | Screenshot utility |
| `noxl-archive` | 2.1 MB | Archive manager |
| `noxl-software` | 2.0 MB | Software centre |
| `noxl-installer` | 3.9 MB | Graphical installer |
| **Total** | **~42 MB** | Full desktop OS |

## Build

```bash
# Desktop
cd desktop && cargo build --release

# Applications
cd applications && cargo build --release

# Installer
cd installer && cargo build --release
```

## Verify

```bash
# All binaries respond to --help
./target/release/noxl-compositor --help
./target/release/noxl-terminal --help
./target/release/noxl-files --help
./target/release/noxl-monitor --help
./target/release/noxl-installer --help
```

## Install to system

```bash
sudo cp target/release/noxl-* /usr/local/bin/
```