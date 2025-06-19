# Aplikasi Monitor Sensor NTC

Proyek ini adalah aplikasi Human-Machine Interface (HMI) yang dibuat menggunakan Rust dan `egui` untuk memonitor, memvisualisasikan, dan menyimpan data dari sensor NTC Thermistor yang terhubung ke Arduino Nano.

## Fitur Utama

- **Visualisasi Real-time**: Menampilkan data suhu, ADC, tegangan, dan resistansi dalam bentuk kartu informasi dan grafik tren.
- **Database Historis**: Menyimpan setiap data yang masuk ke dalam database MongoDB dan menampilkannya dalam format tabel.
- **Verifikasi Numerik**: Membandingkan hasil perhitungan suhu dari metode analitik (rumus Steinhart-Hart) dengan metode numerik (Newton-Raphson) untuk validasi akurasi.
- **Ekspor Data**: Fungsionalitas untuk mengekspor data historis ke dalam format file `.csv`.
- **Konfigurasi Fleksibel**: Pengaturan port serial, ambang batas peringatan, dan konfigurasi sensor dapat diubah melalui antarmuka.

## Cara Menjalankan

1.  Pastikan Anda memiliki Rust dan Cargo terinstal.
2.  Hubungkan perangkat Arduino yang sudah diprogram ke komputer.
3.  Jalankan aplikasi dari direktori root proyek:
    ```bash
    cargo run
    ```