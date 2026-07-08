#!/bin/bash
cat << 'README_EOF' > README.md
# 🧠 Static-Memory: Context-Aware Local Activity Logger

**Static-Memory** adalah sistem perekaman dan analitik aktivitas sistem lokal yang sangat efisien dan aman. Aplikasi ini dirancang untuk berjalan di latar belakang (daemon) dengan konsumsi memori dan CPU seminimal mungkin, sambil mencatat aktivitas produktivitas, ketikan, dan pergantian aplikasi (window) secara akurat. Data sepenuhnya disimpan secara lokal di mesin Anda dan tidak pernah dikirim ke server eksternal, menjamin kedaulatan data dan privasi 100%.

Proyek ini dibangun menggunakan **Rust** dengan runtime asinkron **Tokio**, memberikan presisi tinggi dalam pencatatan peristiwa tanpa menyebabkan *input lag* (jeda ketikan) pada sistem operasi.

---

## ✨ Fitur Utama (Core Features)

1. **Arsitektur Daemon-Client (Kinerja Maksimal)**
   Sistem dibagi menjadi dua bagian:
   * **Background Daemon**: Layanan tanpa antarmuka grafis yang terus berjalan di latar belakang, menangani operasi I/O, deteksi input OS, penerapan aturan privasi, dan interaksi dengan database SQLite.
   * **Thin TUI Client (Terminal UI)**: Antarmuka yang sangat ringan (hanya memakan 10-15 MB RAM) yang dapat dipanggil saat Anda ingin melihat statistik. Berkomunikasi dengan daemon via *Local IPC (Unix Domain Sockets di Linux atau Named Pipes di Windows)*. Saat UI ditutup (`Ctrl+D`), layanan daemon tetap berjalan dengan tenang.

2. **Kinerja Ultra-Efisien (Sumber Daya Rendah)**
   * **Memori**: Menargetkan **50 - 60 MB RAM** saat status stabil (Steady State Daemon).
   * **CPU**: Kurang dari **1%**, bahkan saat mengetik dengan cepat.
   * **Optimalisasi Buffer**: Algoritma cerdas secara otomatis melakukan *flush* teks ke database secara berkala (standar 512 karakter) atau langsung saat mendeteksi perpindahan *window*, menjaga konsumsi memori tetap stabil.
   * **Ukuran Biner Terkompresi**: Dibuat menggunakan tingkat optimasi tertinggi (`opt-level = 'z'`) dipadukan dengan LTO (Link Time Optimization) menghasilkan biner tunggal mandiri (~2.3 MB).

3. **Context-Awareness (Linux & Windows)**
   * **Linux**: Menggunakan integrasi X11 dan membaca `/proc/<pid>/comm` untuk menentukan PID (*Process ID*) aktif serta merekam nama aplikasi dengan presisi tinggi. Mendukung deteksi keyboard dinamis via antarmuka asinkron `evdev`.
   * **Windows**: Memanfaatkan integrasi `windows-sys` secara native tanpa memerlukan dependensi pihak ketiga, serta mampu bekerja sebagai servis "silent" (*no console window*).

4. **Keamanan & Manajemen Data (Database Management)**
   * Menggunakan **SQLite** dengan konfigurasi WAL (*Write-Ahead Logging*) untuk meminimalisasi *disk I/O* dan mempercepat performa transaksi.
   * **Database Rotation (Rotasi Cerdas)**: Jika basis data menyentuh ambang batas **50 MB**, file otomatis diarsip ke `.bak` dan database segar akan dibuat untuk menjaga kecepatan pencarian.
   * **Kebijakan Retensi (Auto-Purge)**: Data lama yang melewati batas hari (default: 7 hari) dihapus otomatis secara berkala.
   * **Ekspor & Pembersihan Total**: Menyediakan opsi *Command Line Interface (CLI)* untuk ekspor langsung ke bentuk `.csv` atau `.txt`, dan perintah `--purge` untuk menyapu bersih semua data dengan aman.

5. **Privasi & Hot-Reloading Konfigurasi**
   * Menyediakan fitur daftar pengecualian (Exclude List). Perekaman dinonaktifkan secara instan jika mendeteksi *window title* (misalnya: "Incognito") atau proses aplikasi (misalnya: "bitwarden.exe") yang masuk daftar privasi Anda.
   * **Hot-Reloading**: Aturan privasi baru yang ditambahkan di `config.toml` akan diterapkan *real-time* dalam waktu kurang dari 60 detik (menggunakan pemantauan metadata asinkron tanpa *overhead* berat), tanpa perlu *restart* daemon.

6. **Analitik TUI Lanjutan (Dasbor Kinerja)**
   Menyediakan laporan real-time tanpa delay melalui IPC:
   * 📊 **Grafik Aktivitas Per Jam (Hourly Activity Chart)**
   * 🏆 **Top 5 Aplikasi Paling Produktif**
   * ⏱️ **Deteksi IDLE / AFK Otomatis** (Otomatis masuk mode istirahat jika tidak ada input setelah 3 menit).

---

## ⌨️ Panduan Hotkey (Matrix Navigasi TUI)

Berikut adalah kontrol antarmuka berbasis terminal (*TUI Client*) setelah Anda menjalankan `static-memory`:

| Tombol (Hotkey) | Reaksi Antarmuka | Fungsi pada Mesin Utama |
| :--- | :--- | :--- |
| `static-memory` | Membuka TUI Client interaktif | Melakukan koneksi IPC -> Mode Perekaman Aktif |
| `Space` atau `p` | Menjeda antarmuka (Pause UI) | Menjeda aliran data agar Anda bisa mem-filter / scroll |
| `Tab` / `Shift+Tab` | Mengubah fokus widget UI | - |
| `Kanan/Kiri/h/l` | Pindah tab panel utama | Berpindah antara *Timeline Data* & *Dashboard Analytics* |
| `d` atau `Ctrl+D` | Menutup antarmuka terminal | UI Mati -> Daemon Latar Belakang terus mencatat (*Detached*) |
| `q` atau `Q` | Shutdown / Matikan Total | Mengirim Sinyal Pembunuhan (KILL) -> Semua proses dihentikan dengan aman |
| `Atas/Bawah/j/k` | Scroll data secara spesifik | (*Hanya aktif saat antarmuka dijeda*) |
| `/` | Mengaktifkan Mode Pencarian | Mencari aplikasi spesifik di dalam riwayat timeline |
| `Ctrl + E` | Memunculkan Dialog Ekspor | Pembuatan laporan (Laporan Harian ke TXT / Laporan Penuh ke CSV) |
| `Ctrl + X` | Memunculkan Dialog Hapus | Fitur *Panic Button* (Menghapus seluruh jejak SQLite secara instan) |
| `Esc` | Menutup panel dialog | - |

---

## 📂 Struktur Data dan Standar Jalur Sistem

Berdasarkan sistem operasi yang digunakan, aplikasi secara cerdas memisahkan antara Data (Database/Sockets) dan Konfigurasi (File `config.toml`).

### 🐧 Lingkungan Linux (Standar Kepatuhan XDG)
* **File Konfigurasi**: `~/.config/static-memory/config.toml`
* **File Database & Socket**: `~/.local/share/static-memory/`

### 🪟 Lingkungan Windows
* **File Konfigurasi & Database**: `%APPDATA%\Static-Memory\`

---

## 🛠️ Otomatisasi Siklus Hidup (Life Cycle & Install)

* **Skrip Instalasi Cerdas (Installation Scripts)**:
   Aplikasi mencakup berkas instalasi interaktif (`install.sh` untuk Linux, `install.ps1` untuk Windows) yang:
   1. Mengecek dependensi (misal: memeriksa ketersediaan `libx11-dev` dan `libxtst-dev` pada Debian/Ubuntu).
   2. Melakukan proses kompilasi kode biner (`cargo build --release`).
   3. Mengatur agar `static-memory` dieksekusi secara otomatis setiap PC menyala melalui pengaturan *systemd* user lokal atau *Windows Registry Run*.
* **Otomatisasi Rollback (Kegagalan Aman)**: Jika proses perpindahan *binary* ke `/usr/local/bin` atau modifikasi profil gagal, skrip akan mengembalikan kondisi sistem ke semula (*idempotent*).
* **Uninstaller Lengkap**: Menggunakan `uninstall.sh` (Linux) atau `uninstall.ps1` (Windows), memungkinkan Anda untuk sekadar mencabut *binary* dari sistem atau menghapus total seluruh *database riwayat* hingga ke akar.

---

## 🛡️ Standar Privasi dan Lisensi
**Static-Memory** sepenuhnya mematuhi prinsip lisensi MIT.
Proyek ini diciptakan khusus untuk analisis personal *self-quantified* secara etik dan bertanggung jawab, menghormati kedaulatan perangkat klien tanpa komunikasi web atau pelacak eksternal (*tracker*).
README_EOF
