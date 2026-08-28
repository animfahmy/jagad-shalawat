# Panduan Deployment Tagih Otomatis Blog (GCP VPS + OpenLiteSpeed)

Tahap ini mencakup persiapan server, instalasi dependensi (Redis), konfigurasi daemon systemd, dan konfigurasi *Reverse Proxy* di OpenLiteSpeed agar blog Rust kita bisa diakses melalui `jagadshalawat.org/blog`.

## 1. Persiapan File & Upload ke VPS

Pertama, kita perlu mengunggah *binary* hasil kompilasi dan aset web ke server VPS.
Saya saat ini sedang melakukan *compile* binary rilis di latar belakang. Setelah selesai, file akan tersedia di folder komputer lokal Bapak: `c:\wamp64\www\jagad-shalawat\target\release\jagad-shalawat`.

Bapak perlu mengunggah file dan folder berikut ke VPS (ke dalam folder `/var/www/jagad-shalawat`):
1. File binary `target/release/jagad-shalawat`
2. Folder `src/templates`
3. Folder `src/static`
4. File `.env` (pastikan konfigurasi database dan koneksi Redis sudah disesuaikan)

> [!TIP]
> Bapak bisa menggunakan WinSCP atau `scp` untuk mentransfer file-file tersebut.
> Pastikan struktur foldernya di VPS menjadi seperti ini:
> `/var/www/jagad-shalawat/jagad-shalawat` (binary file)
> `/var/www/jagad-shalawat/src/templates/`
> `/var/www/jagad-shalawat/src/static/`
> `/var/www/jagad-shalawat/.env`

Atur *permission* agar dapat dieksekusi oleh sistem:
```bash
sudo chmod +x /var/www/jagad-shalawat/jagad-shalawat
sudo chown -R nobody:nogroup /var/www/jagad-shalawat
```

## 2. Instalasi Redis di VPS

Sistem cache dan *rate limiter* kita menggunakan Redis.
Silakan login ke VPS Bapak via SSH dan jalankan perintah berikut:

```bash
sudo apt update
sudo apt install redis-server -y
sudo systemctl enable redis-server
sudo systemctl start redis-server
```

Bapak bisa memastikan Redis berjalan dengan perintah: `redis-cli ping` (Seharusnya membalas `PONG`).

## 3. Membuat Systemd Service (Daemon)

Agar aplikasi Rust terus berjalan di latar belakang dan otomatis menyala kembali (restart) ketika server direstart, kita akan membuat sebuah service `systemd`.

Buat file service baru:
```bash
sudo nano /etc/systemd/system/tagih-blog.service
```

Isi dengan konfigurasi berikut:
```ini
[Unit]
Description=Tagih Otomatis Blog (Rust)
After=network.target mysql.service redis-server.service

[Service]
User=nobody
Group=nogroup
WorkingDirectory=/var/www/jagad-shalawat
ExecStart=/var/www/jagad-shalawat/jagad-shalawat
Restart=always
RestartSec=5
Environment="RUST_LOG=info"
# Atur limit memori (opsional, karena memori kita hanya 1GB)
MemoryMax=150M

[Install]
WantedBy=multi-user.target
```

Simpan dan jalankan daemon:
```bash
sudo systemctl daemon-reload
sudo systemctl enable tagih-blog
sudo systemctl start tagih-blog
sudo systemctl status tagih-blog
```
Jika statusnya *active (running)*, maka blog backend sudah berhasil menyala di port `8080`.

## 4. Konfigurasi OpenLiteSpeed (Reverse Proxy)

Sekarang kita perlu memberitahu OpenLiteSpeed untuk meneruskan (forward) semua permintaan yang masuk ke `jagadshalawat.org/blog` menuju aplikasi Rust kita di port `8080`.

1. Login ke OpenLiteSpeed WebAdmin Console (biasanya di port `7080`).
2. Masuk ke **Virtual Hosts** -> Pilih Virtual Host untuk `jagadshalawat.org`.
3. Buka tab **External App**.
4. Klik tombol tambah `+` untuk membuat eksternal app baru.
   - Pilih tipe: **Web Server**
   - **Name**: `rust_blog_backend`
   - **Address**: `127.0.0.1:8080`
   - **Max Connections**: `100`
   - Simpan.
5. Pindah ke tab **Context** (Masih di dalam Virtual Host).
6. Klik tombol tambah `+` untuk membuat Context baru.
   - Pilih tipe: **Proxy**
   - **URI**: `/blog/`
   - **Web Server**: Pilih `[Server Level]: rust_blog_backend` (atau nama yang Bapak buat tadi)
   - Simpan.
7. Lakukan **Graceful Restart** pada OpenLiteSpeed.

## 5. Sinkronisasi Migrasi Database

Karena aplikasi blog kita membutuhkan beberapa tabel tambahan (seperti `blog_posts`, `blog_comments`, dll), kita butuh menjalankan migrasinya di database produksi MariaDB.

Cara termudah, kita menggunakan `sqlx` CLI, atau saya bisa menyediakan skrip SQL (Data Definition Language) murninya kepada Bapak agar bisa di-import langsung melalui phpMyAdmin/DBeaver.

Apakah Bapak lebih suka menjalankan file `.sql` mentah langsung di phpMyAdmin untuk VPS-nya? Jika ya, saya akan siapkan skrip `init_blog_tables.sql` untuk Bapak.
