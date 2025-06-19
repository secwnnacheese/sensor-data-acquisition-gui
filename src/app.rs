// Mengimpor modul-modul yang diperlukan.
use crate::analysis; // Modul untuk perhitungan numerik.
use crate::db_handler; // Modul untuk interaksi dengan database.
use crate::serial_thread::SerialControl; // Enum untuk mengontrol thread serial.
use crate::types::{AlertThresholds, Language, Page, SensorCharacteristics, SensorReading, Theme}; // Tipe data kustom.
use chrono::{DateTime, FixedOffset, Utc}; // Untuk manajemen waktu dan tanggal.
use eframe::egui::{self, Align, Color32, Layout}; // Komponen UI dari eframe/egui.
use notify_rust::Notification; // Untuk mengirim notifikasi desktop.
use rodio::{source::SineWave, OutputStream, Sink, Source}; // Untuk memutar suara peringatan.
use std::collections::VecDeque; // Struktur data antrian dua arah, efisien untuk data time-series.
use std::sync::mpsc::{self, Receiver, Sender}; // Untuk komunikasi antar thread (channel).
use std::thread::{self, JoinHandle}; // Untuk manajemen thread.
use std::time::{Duration, Instant}; // Untuk mengukur waktu.

// Struct untuk melacak status aktif dari peringatan (alert).
// Ini mencegah notifikasi dikirim berulang kali untuk kondisi yang sama.
#[derive(Debug, Default)]
struct AlertState {
    temp_high_active: bool,
    temp_low_active: bool,
}

// Struct utama yang menyimpan seluruh state aplikasi.
pub struct SensorApp {
    // --- Data Sensor ---
    pub readings: VecDeque<SensorReading>, // Menyimpan 1000 data sensor terakhir.
    pub analytical_temps_from_r: VecDeque<f64>, // Menyimpan hasil perhitungan suhu analitik.
    pub numerical_temps_from_r: VecDeque<analysis::CalculationResult>, // Menyimpan hasil perhitungan suhu numerik.
    
    // --- Konfigurasi & State UI ---
    pub characteristics: SensorCharacteristics, // Karakteristik sensor yang dapat diubah.
    pub current_page: Page, // Halaman yang sedang ditampilkan di UI.
    pub is_connected: bool, // Status koneksi ke perangkat sensor.
    pub alert_thresholds: AlertThresholds, // Batas suhu untuk peringatan.
    pub export_path: String, // Path file untuk ekspor data CSV.
    last_update_display: String, // Teks timestamp pembaruan terakhir di UI.
    pub export_status_message: Option<String>, // Pesan status setelah ekspor (berhasil/gagal).
    pub theme: Theme, // Tema UI (Terang/Gelap).
    pub language: Language, // Bahasa UI (Inggris/Indonesia).

    // --- Komunikasi & Threading ---
    sensor_rx: Receiver<SensorReading>, // Menerima data dari thread serial.
    pub sensor_tx: Sender<SensorReading>, // Mengirim data (dipakai oleh thread serial).
    pub db_tx: Sender<SensorReading>, // Mengirim data ke thread database.
    last_data_received_at: Instant, // Waktu terakhir data diterima untuk cek koneksi putus.
    
    // --- Notifikasi & Suara ---
    alert_state: AlertState, // State untuk peringatan suhu.
    _stream: OutputStream, // Handle untuk stream audio (harus disimpan agar tidak di-drop).
    sink: Sink, // Mengontrol pemutaran suara.

    // --- Manajemen Port Serial ---
    pub available_ports: Vec<String>, // Daftar port serial yang tersedia.
    pub selected_port: Option<String>, // Port yang dipilih oleh pengguna.
    pub serial_thread_handle: Option<JoinHandle<()>>, // Handle ke thread serial yang sedang berjalan.
    pub control_tx: Option<Sender<SerialControl>>, // Mengirim sinyal (misal: shutdown) ke thread serial.
    pub connection_status_message: String, // Pesan status koneksi di UI.
}

// Fungsi helper untuk memformat timestamp UTC ke zona waktu WIB (UTC+7).
fn format_timestamp_english(dt_utc: DateTime<Utc>) -> String {
    // Mencoba membuat offset waktu untuk WIB (7 jam * 3600 detik).
    if let Some(wib) = FixedOffset::east_opt(7 * 3600) {
        // Mengkonversi waktu UTC ke zona waktu WIB.
        let dt_wib = dt_utc.with_timezone(&wib);
        // Memformatnya menjadi string yang mudah dibaca.
        dt_wib.format("%B %d, %Y, %I:%M:%S %p WIB").to_string()
    } else {
        // Jika gagal (sangat jarang terjadi), gunakan format standar RFC3339.
        dt_utc.to_rfc3339()
    }
}

// Implementasi metode-metode untuk SensorApp.
impl SensorApp {
    // Konstruktor: dipanggil sekali saat aplikasi dimulai.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Membuat channel untuk komunikasi antara thread utama (UI) dan thread serial.
        let (sensor_tx, sensor_rx) = mpsc::channel();
        // Membuat channel untuk komunikasi antara thread utama (UI) dan thread database.
        let (db_tx, db_rx) = mpsc::channel();
        
        // Menjalankan thread baru untuk menangani penulisan data ke database.
        thread::spawn(move || db_handler::run_db_writer(db_rx));
        
        // Inisialisasi output stream audio untuk suara peringatan.
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Membuat sink untuk memainkan suara pada stream tersebut.
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.pause(); // Jeda pemutaran secara default.

        // Mendapatkan daftar port serial yang tersedia di sistem saat startup.
        let available_ports = serialport::available_ports()
            .unwrap_or_default() // Jika gagal, kembalikan daftar kosong.
            .iter()
            .map(|p| p.port_name.clone())
            .collect();

        // Mengembalikan instance `SensorApp` dengan nilai-nilai awal.
        Self {
            readings: VecDeque::with_capacity(1000),
            analytical_temps_from_r: VecDeque::with_capacity(1000),
            numerical_temps_from_r: VecDeque::with_capacity(1000),
            characteristics: SensorCharacteristics {
                sensor_type: "NTC Thermistor".to_string(),
                model: "Arduino Nano".to_string(),
                accuracy: 1.0,
                range_min: -50.0,
                range_max: 150.0,
                calibration_date: Utc::now(),
            },
            current_page: Page::Home,
            is_connected: false,
            alert_thresholds: AlertThresholds {
                temp_min: 15.0,
                temp_max: 35.0,
            },
            export_path: "ntc_sensor_data.csv".to_string(),
            last_update_display: format_timestamp_english(Utc::now()),
            export_status_message: None,
            sensor_rx,
            sensor_tx,
            db_tx,
            last_data_received_at: Instant::now(),
            alert_state: AlertState::default(),
            _stream,
            sink,
            available_ports,
            selected_port: None,
            serial_thread_handle: None,
            control_tx: None,
            connection_status_message: "Please select a port and connect.".to_string(),
            theme: Theme::Dark,
            language: Language::Indonesian,
        }
    }
    
    // Fungsi untuk memeriksa data sensor dan memicu peringatan jika perlu.
    fn check_and_trigger_alerts(&mut self, reading: &SensorReading) {
        // Cek peringatan suhu tinggi.
        if reading.temperature > self.alert_thresholds.temp_max {
            // Hanya picu jika peringatan belum aktif.
            if !self.alert_state.temp_high_active {
                // Tentukan teks notifikasi berdasarkan bahasa yang dipilih.
                let (summary, body) = match self.language {
                    Language::English => ("High Temperature Alert", format!("Temperature exceeded {:.1}°C", reading.temperature)),
                    Language::Indonesian => ("Peringatan Suhu Tinggi", format!("Suhu melebihi {:.1}°C", reading.temperature)),
                };
                self.trigger_alert(summary, &body);
                self.alert_state.temp_high_active = true; // Set status peringatan menjadi aktif.
            }
        } else {
            self.alert_state.temp_high_active = false; // Reset status jika suhu kembali normal.
        }

        // Cek peringatan suhu rendah (logika serupa dengan suhu tinggi).
        if reading.temperature < self.alert_thresholds.temp_min {
            if !self.alert_state.temp_low_active {
                 let (summary, body) = match self.language {
                    Language::English => ("Low Temperature Alert", format!("Temperature dropped below {:.1}°C", reading.temperature)),
                    Language::Indonesian => ("Peringatan Suhu Rendah", format!("Suhu turun di bawah {:.1}°C", reading.temperature)),
                };
                self.trigger_alert(summary, &body);
                self.alert_state.temp_low_active = true;
            }
        } else {
            self.alert_state.temp_low_active = false;
        }
    }

    // Fungsi untuk menampilkan notifikasi desktop dan memainkan suara.
    fn trigger_alert(&self, summary: &str, body: &str) {
        // Mencoba mengirim notifikasi desktop.
        if let Err(e) = Notification::new().summary(summary).body(body).icon("dialog-warning").urgency(notify_rust::Urgency::Critical).show() {
            eprintln!("Failed to send desktop notification: {}", e);
        }
        // Memainkan suara 'bip'.
        self.sink.clear(); // Bersihkan antrian suara sebelumnya.
        // Buat gelombang sinus (nada) dengan durasi 0.5 detik.
        let source = SineWave::new(880.0).take_duration(Duration::from_millis(500)).amplify(0.20);
        self.sink.append(source); // Tambahkan suara ke sink.
        self.sink.play(); // Mainkan suara.
    }
}

// Implementasi trait `eframe::App` yang merupakan loop utama aplikasi.
impl eframe::App for SensorApp {
    // Fungsi `update` dipanggil oleh eframe pada setiap frame (misalnya, 60 kali per detik).
    // Di sinilah semua logika UI dan pembaruan state terjadi.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Terapkan tema visual (gelap/terang) berdasarkan state.
        match self.theme {
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
        }
        
        // Loop untuk memproses semua data baru yang diterima dari thread serial.
        // `try_recv` tidak memblokir, ia langsung kembali jika tidak ada data.
        while let Ok(new_reading) = self.sensor_rx.try_recv() {
            // Jika menerima data default (kosong), anggap koneksi gagal.
            if new_reading.adc_value == 0 && new_reading.temperature == 0.0 {
                 self.is_connected = false;
                 continue; // Lanjut ke iterasi berikutnya.
            }

            // Periksa apakah data baru ini memicu peringatan.
            self.check_and_trigger_alerts(&new_reading);
            
            // --- Kalkulasi Analitik & Numerik ---
            // Ambil nilai resistansi dari data baru.
            let r = new_reading.resistance;
            // Definisikan konstanta untuk model sensor NTC (Thermistor).
            let t0_kelvin = 25.0 + 273.15; // Suhu nominal (25°C) dalam Kelvin.
            let r0 = 10000.0; // Resistansi nominal (10k Ohm).
            let beta = 3950.0; // Koefisien Beta dari datasheet thermistor.

            // Hitung suhu menggunakan rumus analitik (Steinhart-Hart disederhanakan).
            let ln_r_r0 = (r / r0).ln();
            let t_analytical_kelvin = 1.0 / ( (1.0 / t0_kelvin) + (ln_r_r0 / beta) );
            let t_analytical_celsius = t_analytical_kelvin - 273.15;
            // Tambahkan hasil ke depan VecDeque.
            self.analytical_temps_from_r.push_front(t_analytical_celsius);

            // Hitung suhu menggunakan metode numerik (Newton-Raphson).
            if let Some(numerical_result) = analysis::newton_raphson_temperature(r, beta, r0, t0_kelvin) {
                // Tambahkan hasil (suhu, iterasi, error) ke depan VecDeque.
                self.numerical_temps_from_r.push_front(numerical_result);
            }

            // Kirim data baru ke thread database untuk disimpan.
            if let Err(e) = self.db_tx.send(new_reading.clone()) {
                eprintln!("[DB THREAD] Failed to send data to database: {}", e);
            }
            // Tambahkan data sensor utama ke depan VecDeque.
            self.readings.push_front(new_reading);
            // Jaga agar ukuran VecDeque tidak melebihi 1000 dengan menghapus data terlama.
            if self.readings.len() > 1000 {
                self.readings.pop_back();
                self.analytical_temps_from_r.pop_back();
                self.numerical_temps_from_r.pop_back();
            }
            // Catat waktu data diterima untuk mendeteksi koneksi putus.
            self.last_data_received_at = Instant::now();
        }

        // --- Logika Status Koneksi ---
        // Anggap terhubung jika thread ada DAN data terakhir diterima < 5 detik yang lalu.
        self.is_connected = self.serial_thread_handle.is_some() && self.last_data_received_at.elapsed() < Duration::from_secs(5);
        // Jika thread seharusnya berjalan tapi koneksi putus, tampilkan pesan error.
        if self.serial_thread_handle.is_some() && !self.is_connected {
            self.connection_status_message = match self.language {
                Language::English => "Connection lost. Check device.",
                Language::Indonesian => "Koneksi terputus. Periksa perangkat.",
            }.to_string();
        }

        // Perbarui timestamp di footer UI.
        self.last_update_display = format_timestamp_english(Utc::now());
        // Minta egui untuk menggambar ulang frame berikutnya setelah 500ms.
        // Ini memastikan UI tetap responsif tanpa membebani CPU.
        ctx.request_repaint_after(Duration::from_millis(500));

        // --- Menggambar UI ---

        // 1. Gambar panel sisi kiri (sidebar) untuk navigasi.
        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            // Panggil fungsi dari modul `ui::sidebar` untuk menggambar kontennya.
            crate::ui::sidebar::show_sidebar(self, ui);
        });
        
        // 2. Gambar panel bawah (footer) untuk status dan kontrol.
        egui::TopBottomPanel::bottom("combined_bottom_panel").show(ctx, |ui| {
            let text_color = ui.style().visuals.text_color().gamma_multiply(0.7);
            ui.horizontal(|ui| {
                // Tampilkan status koneksi dengan warna (Hijau/Merah).
                let (status_text, status_color) = if self.is_connected { 
                    (match self.language {
                        Language::English => "Connected",
                        Language::Indonesian => "Terhubung"
                    }, Color32::GREEN) 
                } else { 
                    (match self.language {
                        Language::English => "Disconnected",
                        Language::Indonesian => "Terputus"
                    }, Color32::RED) 
                };
                let mut checked = self.is_connected;
                ui.add_enabled(false, egui::Checkbox::new(&mut checked, "")); // Checkbox non-interaktif.
                ui.colored_label(status_color, status_text);
                ui.separator();
                ui.label(format!("{}: {}", match self.language {
                    Language::English => "Readings",
                    Language::Indonesian => "Data Masuk"
                }, self.readings.len())); // Tampilkan jumlah data yang disimpan.
                
                // Gunakan layout kanan-ke-kiri untuk menempatkan item di ujung kanan footer.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui_right| {
                    ui_right.add_space(10.0);
                    ui_right.label(egui::RichText::new(&self.last_update_display).size(12.0).color(text_color));
                    
                    ui_right.separator();
                    // Tombol untuk mengganti tema (terang/gelap).
                    let theme_icon = if self.theme == Theme::Dark { "🌙" } else { "☀️" };
                    if ui_right.button(theme_icon).on_hover_text(match self.language {
                        Language::English => "Switch light/dark mode",
                        Language::Indonesian => "Ganti mode terang/gelap"
                    }).clicked() {
                        self.theme = if self.theme == Theme::Dark {
                            Theme::Light
                        } else {
                            Theme::Dark
                        };
                    }

                    ui_right.separator();
                    // Tombol untuk mengganti bahasa.
                    let lang_text = match self.language {
                        Language::English => "ID",
                        Language::Indonesian => "EN"
                    };
                    if ui_right.button(lang_text).on_hover_text("Ganti Bahasa").clicked() {
                        self.language = match self.language {
                            Language::English => Language::Indonesian,
                            Language::Indonesian => Language::English,
                        };
                    }
                });
            });
        });
        
        // 3. Gambar panel tengah sebagai area konten utama.
        egui::CentralPanel::default().show(ctx, |ui| {
            // Gunakan `match` untuk me-render halaman yang sesuai dengan state `current_page`.
            match self.current_page {
                Page::Home => crate::ui::pages::show_home_page(self, ui),
                Page::DataGraphics => crate::ui::pages::show_data_graphics(self, ui), 
                Page::Database => crate::ui::pages::show_database(self, ui),
                Page::SensorCharacteristics => crate::ui::pages::show_sensor_characteristics(self, ui),
                Page::Temperature => crate::ui::pages::show_temperature_page(self, ui),
                Page::SerialPort => crate::ui::pages::show_serial_config_page(self, ui),
                Page::NumericalAnalysis => crate::ui::pages::show_numerical_analysis_page(self, ui),
            }
        });
    }
}