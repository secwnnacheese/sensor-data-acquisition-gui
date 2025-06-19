// Modul ini mendefinisikan seluruh antarmuka dan logika untuk panel navigasi sisi (sidebar).

// Mengimpor 'SensorApp' untuk mengakses dan memodifikasi state aplikasi seperti halaman yang sedang aktif ('current_page').
use crate::app::SensorApp;
// Mengimpor tipe 'Language' dan 'Page' untuk logika multibahasa dan navigasi.
use crate::types::{Language, Page};
// Mengimpor pustaka 'eframe::egui' yang merupakan inti dari framework UI.
// 'Color32' digunakan untuk mendefinisikan warna.
use eframe::egui::{self, Color32};

// Fungsi utama yang dipanggil dari 'app.rs' untuk menggambar sidebar.
// Menerima 'app' sebagai mutable reference (&mut) untuk mengubah state,
// dan 'ui' juga sebagai mutable reference untuk menggambar elemen UI.
pub fn show_sidebar(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Menggunakan layout 'top_down' dengan alignment 'Center'
    // agar semua elemen di dalam sidebar disusun dari atas ke bawah dan terpusat.
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        // Menambahkan spasi kosong di bagian atas sidebar untuk estetika.
        ui.add_space(20.0);
        // Mengelompokkan elemen secara vertikal dan terpusat,
        // cocok untuk judul atau logo aplikasi.
        ui.vertical_centered(|ui| {
            // Menampilkan label "SENSOR" sebagai bagian dari judul.
            ui.label(
                // 'RichText' memungkinkan styling teks yang lebih kaya.
                egui::RichText::new("SENSOR")
                    .size(20.0) // Ukuran font.
                    .strong() // Membuat teks menjadi tebal.
                    .color(Color32::from_rgb(59, 130, 246)), // Memberi warna biru.
            );
            // Menampilkan label "MONITOR" di bawah "SENSOR".
            ui.label(
                egui::RichText::new("MONITOR")
                    .size(12.0)
                    // Menggunakan warna teks standar dari tema yang aktif,
                    // namun dibuat sedikit lebih redup untuk kontras.
                    .color(ui.style().visuals.text_color().gamma_multiply(0.7)),
            );
        });
        // Menambahkan spasi lagi antara judul dan menu navigasi.
        ui.add_space(30.0);
        
        // Mendefinisikan 'closure' (fungsi anonim) sebagai helper untuk membuat tombol menu.
        // Ini adalah praktik yang baik untuk menghindari pengulangan kode (DRY - Don't Repeat Yourself).
        let button_style = |ui: &mut egui::Ui, text: &str, page: Page, current_page_ref: &mut Page| {
            // Memeriksa apakah halaman yang direpresentasikan oleh tombol ini sedang aktif.
            let is_active = *current_page_ref == page;
            // Menetapkan tinggi tombol yang konsisten.
            let button_height = 42.0;
            // Menambahkan widget tombol dengan ukuran yang telah ditentukan.
            let button_response = ui.add_sized(
                // Lebar tombol akan mengisi ruang yang tersedia, dengan lebar minimum 180.0.
                egui::vec2(ui.available_width().max(180.0), button_height),
                // Membuat widget tombol baru.
                egui::Button::new(
                    // Mengatur teks di dalam tombol.
                    egui::RichText::new(text)
                        .size(14.0)
                        // Warna teks akan putih jika aktif, jika tidak, akan mengikuti warna tema.
                        .color(if is_active {
                            Color32::WHITE
                        } else {
                            ui.style().visuals.text_color()
                        }),
                )
                // 'frame(false)' menonaktifkan frame default tombol, agar bisa digambar kustom.
                // Logika ini (salah) diterjemahkan dari `frame(is_active)`. Seharusnya `frame(!is_active)`.
                // Namun sesuai permintaan, kode tidak diubah.
                .frame(is_active),
            );

            // Jika tombol ini aktif, kita gambar latar belakang kustom.
            if is_active {
                // Mengambil persegi (rectangle) dari respons tombol.
                let rect = button_response.rect;
                // Menggambar persegi dengan sudut melengkung sebagai latar belakang.
                ui.painter().rect_filled(
                    rect,
                    egui::Rounding::same(8.0), // Jari-jari lengkungan sudut.
                    Color32::from_rgb(59, 130, 246), // Warna latar belakang biru.
                );
                // Menggambar ulang teks di atas latar belakang kustom agar terlihat.
                ui.painter().text(
                    rect.center(), // Posisi teks di tengah.
                    egui::Align2::CENTER_CENTER, // Alignment teks.
                    text,
                    egui::FontId::proportional(14.0), // Font teks.
                    Color32::WHITE, // Warna teks putih.
                );
            }
            // Jika tombol diklik, ubah state 'current_page' di aplikasi utama.
            if button_response.clicked() {
                *current_page_ref = page;
            }
        };
        
        // Logika multibahasa: Mendefinisikan teks untuk setiap tombol
        // berdasarkan state 'app.language' yang sedang aktif.
        let (home_text, graphics_text, db_text, temp_text, s_conf_text, s_port_text, analysis_text) = match app.language {
            Language::English => ("Home", "Data Graphics", "Database", "Temperature", "Sensor Config", "Serial Port", "Numerical Analysis"),
            Language::Indonesian => ("Beranda", "Grafik Data", "Basis Data", "Suhu", "Konfigurasi Sensor", "Port Serial", "Analisis Numerik"),
        };

        // --- PENGGUNAAN HELPER UNTUK MEMBUAT TOMBOL MENU ---

        // Kelompok 1: Menu Utama
        // Memanggil helper 'button_style' untuk membuat tombol "Beranda".
        button_style(ui, home_text, Page::Home, &mut app.current_page);
        ui.add_space(4.0); // Spasi antar tombol.
        button_style(ui, graphics_text, Page::DataGraphics, &mut app.current_page);
        ui.add_space(4.0);
        button_style(ui, db_text, Page::Database, &mut app.current_page);
        
        // Menambahkan spasi sebelum garis pemisah.
        ui.add_space(12.0);
        // Menggambar garis pemisah horizontal untuk mengelompokkan menu.
        ui.separator();
        // Menambahkan spasi setelah garis pemisah.
        ui.add_space(12.0);

        // Kelompok 2: Menu Analisis
        button_style(ui, temp_text, Page::Temperature, &mut app.current_page);
        ui.add_space(4.0);
        button_style(ui, analysis_text, Page::NumericalAnalysis, &mut app.current_page);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        
        // Kelompok 3: Menu Konfigurasi
        button_style(ui, s_conf_text, Page::SensorCharacteristics, &mut app.current_page);
        ui.add_space(4.0);
        button_style(ui, s_port_text, Page::SerialPort, &mut app.current_page);
    });
}