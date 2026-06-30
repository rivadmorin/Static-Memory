# 🧠 Static-Memory: Context-Aware Local Activity Logger

**Static-Memory** adalah sistem pencatatan aktivitas lokal yang ultra-efisien, dirancang khusus untuk pengguna yang mengutamakan kedaulatan data dan privasi total. Dibangun dengan bahasa pemrograman Rust dan runtime asinkron Tokio, proyek ini mampu menangkap setiap keystroke dan metrik sistem dengan presisi tinggi, memetakannya ke konteks jendela yang aktif tanpa mengorbankan performa sistem.

---

## ⚡ Proposisi Nilai & Jaminan Privasi

Kami menjamin bahwa data Anda adalah milik Anda sepenuhnya. Static-Memory beroperasi dengan prinsip **Zero-Telemetry**.

| Metrik | Target Performa |
| :--- | :--- |
| **Penggunaan RAM** | 50 - 60 MB (Steady State) |
| **Penggunaan CPU** | < 1% (Bahkan saat aktivitas input tinggi) |
| **Disk I/O** | Minimal (Optimasi via SQLite WAL & Batch Flushing) |
| **Kedaulatan Data** | 100% Lokal (Tidak ada pengiriman data ke awan) |
| **Keamanan** | Enkripsi database opsional & pembersihan otomatis |

---

## 🏗️ Arsitektur Sistem (Data Flow)

Static-Memory menggunakan pipeline pemrosesan data multi-threaded yang tidak memblokir antarmuka pengguna atau latensi input OS.

```text
[ Hardware Interrupt ]
          │
          ▼
[ OS Hook Loop ] ──────────────► [ Raw Event Capture ]
          │                       (SetWindowsHookExW / evdev)
          ▼
[ SmallVec Stack Buffer ] ─────► [ Logika Koreksi Input ]
          │                       (Backspace/Delete Handling)
          ▼
[ Bounded Tokio MPSC ] ────────► [ Context Engine ]
          │                       (Window Title Validation & Filter)
          ▼
[ Single-Writer Thread ] ──────► [ SQLite (WAL Mode) ]
          │                       (Persistensi Sinkron & Berurutan)
          ▼
[ UI Event Loop ] ─────────────► [ TUI Dashboard ]
                                  (Visualisasi Real-time via Ratatui)
```

---

## 🗺️ Peta Codebase Menyeluruh (Technical Blueprint)

### Core Configurations & Orchestration

*   **`Cargo.toml`**: Mengelola dependensi minimal untuk menjaga *footprint* memori. Menggunakan `tokio` dengan fitur terbatas (macros, rt-multi-thread, signal), `ratatui` & `tui-realm` untuk UI, `rusqlite` untuk penyimpanan, serta `smallvec` dan `smol_str` untuk eliminasi alokasi heap yang tidak perlu.
*   **`src/main.rs`**: Jantung dari aplikasi. Menginisialisasi runtime Tokio, mengatur *Custom Panic Hook* menggunakan `std::panic::set_hook` untuk memastikan terminal kembali ke mode normal jika terjadi crash via `crossterm`, serta mengorkestrasi channel asinkron antar komponen (Collector -> Engine -> Storage).

### Abstraksi & Implementasi OS (`src/os/`)

*   **`src/os/mod.rs`**: Mendefinisikan trait `AsyncCollector` (atau `OSInterface`) sebagai abstraksi lintas platform untuk manajemen siklus penangkapan event dan perolehan informasi jendela aktif.
*   **`src/os/windows.rs`**: Implementasi spesifik Windows. Menggunakan Win32 API seperti `GetForegroundWindow` dan `GetWindowTextW` untuk konteks aplikasi, serta `SetWindowsHookExW` (WH_KEYBOARD_LL) untuk menangkap input keyboard tingkat rendah.
*   **`src/os/linux.rs`**: Implementasi Linux. Menggunakan `x11-dl` untuk melacak jendela aktif di X11. Untuk input keyboard, terdapat logika *auto-detection* file descriptor di `/dev/input/` (evdev) dengan melakukan parsing pada `/proc/bus/input/devices` sebagai mekanisme *fallback* yang andal untuk lingkungan Wayland/X11.

### Engine & Buffer Logic (`src/engine/`)

*   **`src/engine/mod.rs`**: Bertanggung jawab atas logika *State Manager*. Memvalidasi judul jendela terhadap `exclude_list` dan mengelola siklus hidup buffer teks.
*   **`src/engine/buffer.rs`**: Implementasi **Stateful Text Buffer**. Menggunakan `SmallVec<[char; 64]>` untuk menyimpan input secara efisien di stack. Menangani tombol kontrol seperti `Backspace` dan `Delete` secara akurat untuk memastikan log mencerminkan teks final yang diketik pengguna. Mengimplementasikan strategi **"Immediate Flush & Swap"** saat terdeteksi perpindahan jendela aktif.

### Storage Layer (`src/storage/`)

*   **`src/storage/mod.rs`**: Implementasi pola **Single-Writer Thread**. Memastikan semua operasi penulisan ke database dilakukan oleh satu thread khusus untuk menghindari *locking* pada runtime asinkron.
*   **`src/storage/db.rs`**: Konfigurasi SQLite tingkat lanjut. Menggunakan `PRAGMA journal_mode = WAL`, `PRAGMA synchronous = NORMAL`, dan `PRAGMA cache_size = -2000` (pembatasan ~2MB). Skema mencakup indeks pada kolom `timestamp` dan `app_name` untuk pencarian cepat, serta logika rotasi otomatis jika file database mendekati ambang batas 50 MB.

### User Interface (`src/ui/`)

*   **`src/ui/mod.rs`**: Arsitektur UI berbasis komponen menggunakan `tui-realm` yang mengikuti pola *The Elm Architecture* (Model-Update-View).
*   **`src/ui/components/`**: Bedah komponen spesifik:
    *   **Activity Timeline**: Visualisasi urutan waktu aktivitas lengkap dengan metrik KPM (Keystrokes Per Minute).
    *   **Word Detail Panel**: Menampilkan daftar kata yang ditangkap menggunakan `smol_str` untuk optimasi memori pada string pendek.
    *   **Export Form Modal**: Antarmuka berbasis form untuk mengatur rentang tanggal dan format ekspor data.
    *   **Purge Dialog Modal**: Dialog konfirmasi untuk pembersihan log secara aman.

---

## ⚙️ Panduan Konfigurasi (`config.toml`)

Contoh konfigurasi untuk kustomisasi penuh:

```toml
[storage]
db_path = "activity_log.db"          # Lokasi database SQLite
rotation_size_mb = 50                # Batas ukuran rotasi log
rotation_interval_days = 30          # Batas waktu penyimpanan log

[privacy]
# Daftar proses yang tidak akan pernah dicatat (misal: password manager)
exclude_processes = ["bitwarden.exe", "keepassxc", "1password"]
# Kata kunci judul jendela yang memicu jeda pencatatan
exclude_titles = ["Incognito", "Private Browsing", "Banking", "KeePass"]

[linux]
# Jalur manual perangkat input jika auto-detection tidak diinginkan
# keyboard_device_path = "/dev/input/event3"
```

---

## 🚀 Otomatisasi Operasional

### Linux (`install.sh` / `uninstall.sh`)
Skrip instalasi melakukan:
1.  Pemasangan prasyarat: `libx11-dev`, `libxtst-dev`, `libxi-dev`.
2.  Konfigurasi grup `input` via aturan `udev` agar aplikasi dapat mengakses `/dev/input/`.
3.  Kompilasi biner dengan flag `--release` dan optimasi ukuran.
4.  Penyematan unit service ke `systemctl --user` untuk *auto-start* saat login.

### Windows (`install.ps1` / `uninstall.ps1`)
Skrip PowerShell melakukan:
1.  Verifikasi hak akses Administrator.
2.  Penempatan biner pada PATH sistem.
3.  Injeksi Registry Run Key (`HKCU\...\Run`) untuk memastikan aplikasi berjalan saat startup tanpa prompt UAC.

---

## ⌨️ Navigasi TUI (Shortcut Cheatsheet)

| Tombol | Tindakan |
| :--- | :--- |
| `Tab` / `Shift+Tab` | Pindah antar Panel (Timeline, Details, Metrics) |
| `/` | Masuk ke mode Pencarian / Filter History |
| `Ctrl + E` | Membuka Modal Ekspor Data |
| `Ctrl + X` | Membuka Dialog Pembersihan Data (Purge) |
| `Esc` | Menutup Modal atau Membatalkan Pencarian |
| `Q` | Keluar dari Aplikasi |

---

## 🛡️ Lisensi & Etika
Static-Memory dilisensikan di bawah MIT. Proyek ini dibuat untuk produktivitas pribadi dan analisis diri (*self-quantified*). Harap gunakan dengan bijak dan hormati privasi orang lain.
