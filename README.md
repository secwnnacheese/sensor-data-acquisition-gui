# Aplikasi Monitor Sensor NTC

Proyek ini adalah aplikasi Human-Machine Interface (HMI) yang dibuat menggunakan Rust dan `egui` untuk memonitor, memvisualisasikan, dan menyimpan data dari sensor NTC Thermistor yang terhubung ke Arduino Nano.

## Fitur Utama

- **Visualisasi Real-time**: Menampilkan data suhu, ADC, tegangan, dan resistansi dalam bentuk kartu informasi dan grafik tren.
- **Database Historis**: Menyimpan setiap data yang masuk ke dalam database MongoDB dan menampilkannya dalam format tabel.
- **Verifikasi Numerik**: Membandingkan hasil perhitungan suhu dari metode analitik (persamaan Beta/Steinhart-Hart) dengan metode numerik (Newton-Raphson) untuk validasi akurasi.
- **Peringatan Ambang Batas**: Notifikasi desktop dan bunyi peringatan (beep) ketika suhu keluar dari rentang aman yang dikonfigurasi (bawaan: 15–35 °C).
- **Ekspor Data**: Fungsionalitas untuk mengekspor data historis ke dalam format file `.csv`.
- **Konfigurasi Fleksibel**: Pengaturan port serial, ambang batas peringatan, dan konfigurasi sensor dapat diubah melalui antarmuka.

## Teknologi yang Digunakan

- **Rust** (edition 2021) dengan framework GUI `eframe`/`egui` 0.27
- `serialport` 4.2 untuk komunikasi serial dengan Arduino
- Driver `mongodb` 2.8 (runtime Tokio) untuk penyimpanan data historis
- `chrono` untuk penanda waktu, `serde` untuk serialisasi data
- `notify-rust` untuk notifikasi desktop dan `rodio` untuk suara peringatan

## Prasyarat

1. **Rust dan Cargo** terinstal (https://rustup.rs).
2. **MongoDB berjalan secara lokal** — aplikasi terhubung ke `mongodb://localhost:27017` dan menyimpan data ke database `sensor_db`. Jalankan MongoDB terlebih dahulu (misalnya `mongod` atau MongoDB sebagai service) sebelum memakai fitur database historis.
3. **Arduino Nano** (atau board kompatibel) yang sudah diprogram untuk membaca NTC thermistor dan mengirim data melalui serial sesuai protokol di bawah.

## Protokol Data Serial

Aplikasi berkomunikasi dengan Arduino melalui port serial pada **baud rate 9600 bps**. Arduino harus mengirim **satu baris data per pembacaan**, berisi empat nilai yang dipisahkan koma, diakhiri newline:

```
suhu,adc,tegangan,resistansi
```

| Kolom | Isi | Tipe |
|:------|:----|:-----|
| 1 | Suhu (°C) | desimal |
| 2 | Nilai ADC mentah | integer 0–65535 |
| 3 | Tegangan (V) | desimal |
| 4 | Resistansi thermistor (Ω) | desimal |

Contoh baris data yang valid:

```
25.31,512,2.50,9987.4
```

## Cara Menjalankan

1. Pastikan MongoDB sudah berjalan di komputer (lihat Prasyarat).
2. Hubungkan perangkat Arduino yang sudah diprogram ke komputer.
3. Jalankan aplikasi dari direktori root proyek:
    ```bash
    cargo run --release
    ```
4. Di dalam aplikasi, pilih port serial Arduino dari daftar port yang tersedia, lalu mulai pemantauan. Data yang masuk tampil secara real-time dan tersimpan ke MongoDB melalui thread database terpisah.

## Konfigurasi

- **Port serial**: dipilih dari daftar port yang tersedia di sistem melalui antarmuka aplikasi; baud rate tetap 9600 bps mengikuti protokol di atas.
- **Ambang batas peringatan**: suhu minimum 15 °C dan maksimum 35 °C secara bawaan; keduanya dapat diubah melalui antarmuka. Suhu di luar rentang memicu notifikasi desktop dan bunyi peringatan.
- **Parameter sensor**: perhitungan suhu memakai persamaan Beta dengan koefisien β = 3950 dan suhu nominal T0 = 25 °C, sesuai datasheet thermistor NTC yang umum dipakai.

## Verifikasi Perhitungan Suhu

Setiap pembacaan resistansi dihitung suhunya dengan dua cara independen:

1. **Metode analitik** — rumus Beta/Steinhart-Hart secara langsung: 1/T = 1/T0 + (1/β) · ln(R/R0).
2. **Metode numerik** — iterasi Newton-Raphson yang menyelesaikan persamaan yang sama.

Kedua hasil ditampilkan berdampingan beserta grafik perbandingannya, sehingga penyimpangan perhitungan dapat langsung terlihat.

## Ekspor Data

Data historis dapat diekspor ke file CSV melalui antarmuka aplikasi. Nama file bawaan adalah `ntc_sensor_data.csv`.

## Struktur Proyek

```
src/
├── main.rs           — titik masuk aplikasi
├── app.rs            — state aplikasi, logika utama, dan pemrosesan pembacaan
├── serial_thread.rs  — thread pembaca port serial dan parser data CSV
├── db_handler.rs     — thread database: koneksi MongoDB dan penyimpanan
├── data_handler.rs   — ekspor data ke CSV
├── analysis.rs       — perhitungan suhu Newton-Raphson
├── types.rs          — tipe data (SensorReading, konfigurasi, ambang batas)
├── ui_components.rs  — komponen grafik dan kartu informasi
└── ui/               — halaman antarmuka (pages.rs) dan sidebar (sidebar.rs)
```
