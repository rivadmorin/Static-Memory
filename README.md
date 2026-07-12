# 🧠 Static-Memory: Context-Aware Local Activity Logger

**Static-Memory** adalah sistem pencatatan aktivitas lokal yang ultra-efisien, dirancang khusus untuk pengguna yang mengutamakan kedaulatan data dan privasi total. Dibangun dengan Rust dan runtime asinkron Tokio, proyek ini menangkap setiap keystroke dan metrik sistem dengan presisi tinggi, memetakannya ke konteks jendela yang aktif tanpa mengorbankan performa sistem.

---

## 🏗️ System Architecture & Performance Guarantees

Static-Memory beroperasi menggunakan **Daemon-Client Model** untuk memastikan pemisahan tugas yang bersih antara perekaman data dan visualisasi antarmuka.

*   **Background Daemon**: Berjalan sebagai layanan latar belakang yang persisten. Daemon ini menangani semua operasi input OS, filter privasi, dan manajemen database SQLite secara eksklusif untuk mengeliminasi masalah *database locking*.
*   **Thin TUI Client**: Antarmuka berbasis terminal yang ringan (~10-15 MB footprint aktif). Client berkomunikasi dengan Daemon melalui Local IPC (Unix Domain Sockets di Linux `~/.local/share/static-memory/daemon.sock` dan Named Pipes di Windows). Saat dilepaskan (detached), proses UI mati dan konsumsi sumber daya kembali ke nol, sementara Daemon tetap merekam.

| Metrik | Target Performa |
| :--- | :--- |
| **Penggunaan RAM** | 50 - 60 MB (Steady State Daemon) |
| **Penggunaan CPU** | < 1% (Bahkan saat aktivitas input tinggi) |
| **Disk I/O** | Minimal (Optimasi via SQLite WAL & Batch Flushing) |
| **Database Mode** | PRAGMA journal_mode = WAL, PRAGMA synchronous = NORMAL |

---

## 🚀 Fitur Unggulan (Advanced Features)

*   **Idle/AFK Detection**: Pemicu inaktivitas 3 menit yang secara otomatis menghentikan perekaman KPM (Keystrokes Per Minute). Engine akan beralih ke status `[IDLE]` (ditandai dengan badge merah pada StatusBar) dan menghitung durasi AFK secara akurat hanya saat pengguna kembali aktif.
*   **Data Retention & SQLite Log Rotation**: Menggunakan strategi "Vacuum & Fresh Start". Saat database mencapai **50 MB**, sistem akan mengarsipkannya ke `activity.[timestamp].db.bak` dan memulai database baru. Background worker secara periodik menghapus backup lama berdasarkan kebijakan retensi di `config.toml`.
*   **Hot-Reloading config.toml**: Menggunakan watcher konfigurasi dengan overhead rendah (polling `std::fs::metadata` setiap 60 detik). Arsitektur `Arc<RwLock>` memungkinkan perubahan aturan privasi atau filter jendela diterapkan secara *real-time* tanpa perlu merestart daemon atau UI.
*   **Linux Input Resilience**: Handler stream `evdev` asinkron yang tangguh. Dilengkapi dengan loop koneksi ulang 5 detik untuk memulihkan diri secara dinamis dari kesalahan *hot-plug* atau diskoneksi perangkat periferal.

---

## ⌨️ TUI Control & Navigation Matrix

Berikut adalah panduan lengkap skema kontrol dan interaksi sistem:

| Hotkey / Trigger | TUI Client Interaction | Core Engine State | Terminal Behavior / Impact |
| :--- | :--- | :--- | :--- |
| `static-memory` | Invokes & attaches interactive UI | Connected via IPC -> [RECORDING] | Terminal enters Alternative Raw Mode |
| `Space` or `p` | Freezes/unfreezes UI stream | Switches to [PAUSED] | Halts render loop for easy data scrolling |
| `Tab` / `Shift+Tab` | Shifts focus across UI panels | No Change | Navigates between active UI elements |
| `Right` / `Left` / `h` / `l`| Switches active layout Tabs | No Change | Toggles between Tab 1 (Timeline) & Tab 2 (Analytics) |
| `d` or `Ctrl + D` | Detaches interface safely | Automatically Resumes -> [RECORDING] | Restores terminal mode instantly, UI process dies |
| `q` or `Q` | Issues total Hard Shutdown | Sends KILL signal -> [SHUTDOWN] | Gracefully cleans buffers and terminates all processes |
| `Up` / `Down` / `j` / `k` | Scrolls lists line-by-line | No Change (Only in [PAUSED] mode) | Allows granular historical exploration |
| `PageUp` / `PageDown` | Jumps 10 lines at a time | No Change (Only in [PAUSED] mode) | Allows rapid historical exploration |
| `/` | Spawns interactive filter bar | No Change (Only in [PAUSED] mode) | Filters timeline apps/windows in real time |
| `Ctrl + E` | Displays Data Export Modal | No Change | Prompts for target format (.txt/.csv) and dates |
| `Ctrl + X` | Displays Data Purge Modal | No Change | Asks for confirmation to wipe all local records |
| `Esc` | Dimisses active Modal window | No Change | Returns focus safely back to the main layout screen |

---

## 📂 Struktur Direktori & Standar Kepatuhan

Static-Memory mematuhi standar jalur sistem operasi untuk menjaga kebersihan dan keamanan data:

### Linux (XDG Compliance)
*   **Konfigurasi**: `~/.config/static-memory/config.toml`
*   **Data & Sockets**: `~/.local/share/static-memory/`

### Windows (Standard AppData)
*   **Konfigurasi & Data**: `$env:APPDATA\Static-Memory\`

---

## 🛠️ Otomatisasi Instalasi & Lifecycle

*   **Single-Word Command**: Integrasi perintah tunggal `static-memory` yang disuntikkan langsung ke shell profile (Linux) atau System PATH (Windows).
*   **Safe Execution Order**: Skrip instalasi memvalidasi dependensi sistem (seperti `libx11-dev`) melalui pengecekan kompilasi multi-tahap (`cargo build --release`) sebelum memodifikasi sistem. Tersedia rutinitas **Rollback** otomatis jika terjadi kegagalan di tengah proses.
*   **Flexible Uninstaller**: Skrip uninstalasi memberikan pilihan eksplisit kepada pengguna untuk menghapus biner dan layanan latar belakang saja, atau menghapus seluruh database riwayat secara permanen.

---

## 🛡️ Lisensi & Etika
Static-Memory dilisensikan di bawah MIT. Proyek ini dibuat untuk produktivitas pribadi dan analisis diri (*self-quantified*). Harap gunakan dengan bijak dan hormati privasi orang lain.
