// Mendeklarasikan modul-modul yang akan digunakan dalam proyek.
// Setiap `mod` akan merujuk ke file `nama_modul.rs` atau direktori `nama_modul/mod.rs`.
mod app;
mod data_handler;
mod db_handler;
mod serial_thread;
mod types;
mod ui;
mod ui_components;
mod analysis;

// Mengimpor struct `SensorApp` dari modul `app` agar bisa digunakan di `main`.
use app::SensorApp;

// Fungsi `main` adalah fungsi pertama yang dieksekusi saat program berjalan.
// Mengembalikan `Result` untuk menangani error yang mungkin terjadi saat inisialisasi eframe.
fn main() -> Result<(), eframe::Error> {
    // Mendefinisikan opsi-opsi untuk jendela aplikasi (native window).
    let options = eframe::NativeOptions {
        // Mengkonfigurasi 'viewport' atau area tampilan jendela.
        viewport: eframe::egui::ViewportBuilder::default()
            // Menetapkan ukuran awal jendela saat pertama kali dibuka.
            .with_inner_size([1280.0, 800.0])
            // Menetapkan ukuran minimum jendela agar tidak terlalu kecil saat di-resize.
            .with_min_inner_size([960.0, 720.0]),
        // `..Default::default()` mengisi sisa field di `NativeOptions` dengan nilai defaultnya.
        ..Default::default()
    };
    // Memulai dan menjalankan aplikasi native (desktop).
    eframe::run_native(
        // Argumen 1: Judul jendela aplikasi.
        "Sensor Monitoring",
        // Argumen 2: Opsi jendela yang sudah kita definisikan.
        options,
        // Argumen 3: Sebuah closure (fungsi anonim) yang membuat instance dari aplikasi kita.
        // `Box::new(|cc| ...)` adalah cara eframe untuk mengelola state aplikasi.
        // `SensorApp::new(cc)` memanggil konstruktor yang kita definisikan di `app.rs`.
        Box::new(|cc| Box::new(SensorApp::new(cc))),
    )
}