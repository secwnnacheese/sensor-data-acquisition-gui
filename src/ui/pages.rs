// Mengimpor semua tipe dan fungsi yang dibutuhkan untuk halaman-halaman ini.
use crate::app::SensorApp; // Untuk akses ke state aplikasi.
use crate::data_handler::export_data_as_csv; // Untuk fungsi ekspor CSV.
use crate::serial_thread::{self, SerialControl}; // Untuk manajemen thread serial.
use crate::ui_components::{draw_comparison_chart, draw_mini_chart}; // Komponen UI kustom.
use crate::types::Language; // Untuk logika multibahasa.
use chrono::FixedOffset; // Untuk menampilkan timestamp dalam zona waktu lokal.
use eframe::egui::{self, Color32, ComboBox, Layout, Vec2}; // Komponen-komponen UI egui.
use std::sync::mpsc; // Untuk membuat channel baru saat koneksi serial.
use std::thread; // Untuk spawning thread serial baru.

// --- Halaman Beranda (Home) ---
// Menampilkan ringkasan data sensor terkini dalam bentuk kartu informasi.
pub fn show_home_page(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Gunakan layout dari atas ke bawah, dengan konten rata kiri.
    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
        // Logika untuk menengahkan konten secara vertikal.
        let content_height = 120.0;
        let available_height = ui.available_height();
        let top_margin = (available_height - content_height) / 10.0;
        if top_margin > 0.0 {
            ui.add_space(top_margin);
        }
        
        // Hanya tampilkan kartu jika ada data sensor yang diterima.
        if let Some(latest) = app.readings.front() {
            // Susun kartu-kartu secara horizontal.
            ui.horizontal(|ui| {
                ui.add_space(180.0); // Spasi di kiri untuk penataan.
                ui.spacing_mut().item_spacing.x = 50.0; // Jarak antar kartu.
                
                // Closure helper untuk menggambar satu kartu informasi.
                let draw_info_card =
                    |ui: &mut egui::Ui, title: &str, value: &str, status: (&str, Color32)| {
                        // `Frame::group` memberikan latar belakang dan bingkai standar.
                        egui::Frame::group(ui.style())
                            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                            .rounding(egui::Rounding::same(12.0))
                            .show(ui, |ui| {
                                ui.set_width(140.0); // Lebar kartu tetap.
                                ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                                    // Tampilkan judul kartu.
                                    ui.label(
                                        egui::RichText::new(title)
                                            .size(14.0)
                                            .color(ui.style().visuals.text_color().gamma_multiply(0.8)),
                                    );
                                    ui.add_space(8.0);
                                    // Tampilkan nilai utama dengan font besar dan tebal.
                                    ui.label(egui::RichText::new(value).size(36.0).strong());
                                    ui.add_space(8.0);
                                    // Buat "kapsul" status di bagian bawah kartu.
                                    egui::Frame::none()
                                        .fill(status.1.gamma_multiply(0.2)) // Latar belakang transparan.
                                        .rounding(egui::Rounding::same(12.0))
                                        .inner_margin(egui::Margin { left: 8.0, right: 10.0, top: 4.0, bottom: 4.0 })
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Gambar lingkaran berwarna sebagai indikator status.
                                                let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
                                                ui.painter().circle_filled(rect.center(), 5.0, status.1);
                                                ui.add_space(2.0);
                                                // Tampilkan teks status.
                                                ui.label(
                                                    egui::RichText::new(status.0)
                                                        .size(13.0)
                                                        .strong()
                                                        .color(status.1.gamma_multiply(1.5)),
                                                );
                                            });
                                        });
                                });
                            });
                    };
                
                // Ambil teks UI berdasarkan bahasa yang dipilih.
                let (temp_card_title, res_card_title, volt_card_title, normal_text, high_text, low_text, stable_text, raw_text) = match app.language {
                    Language::English => ("Temperature", "Resistance", "Voltage", "Normal", "High", "Low", "Stable", "Raw Data"),
                    Language::Indonesian => ("Suhu", "Resistansi", "Tegangan", "Normal", "Tinggi", "Rendah", "Stabil", "Data Mentah"),
                };

                // Tentukan status suhu (Tinggi, Rendah, Normal) berdasarkan threshold.
                let temp_status = if latest.temperature > app.alert_thresholds.temp_max {
                    (high_text, Color32::RED)
                } else if latest.temperature < app.alert_thresholds.temp_min {
                    (low_text, Color32::BLUE)
                } else {
                    (normal_text, Color32::GREEN)
                };

                // Gambar kartu-kartu informasi dengan memanggil helper.
                draw_info_card(ui, temp_card_title, &format!("{:.1}°C", latest.temperature), temp_status);
                draw_info_card(ui, "ADC Value", &format!("{}", latest.adc_value), (raw_text, Color32::GRAY));
                draw_info_card(ui, res_card_title, &format!("{:.0} Ω", latest.resistance), ("Ohm", Color32::from_rgb(245, 158, 11)));
                draw_info_card(ui, volt_card_title, &format!("{:.2}V", latest.voltage), (stable_text, Color32::GREEN));
            });
        } else {
            // Jika tidak ada data, tampilkan pesan tunggu.
            ui.vertical_centered_justified(|ui| {
                let waiting_text = match app.language {
                    Language::English => "Waiting for sensor data...",
                    Language::Indonesian => "Menunggu data sensor...",
                };
                ui.label(egui::RichText::new(waiting_text).size(16.0));
            });
        }
    });
}

// --- Halaman Grafik Data ---
// Menampilkan visualisasi data dalam bentuk grafik mini.
pub fn show_data_graphics(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Judul halaman.
    let title_text = match app.language {
        Language::English => "Data Visualization",
        Language::Indonesian => "Visualisasi Data",
    };
    ui.label(egui::RichText::new(title_text).size(32.0).strong());
    ui.add_space(15.0);

    // Gunakan ScrollArea jika konten melebihi tinggi layar.
    egui::ScrollArea::vertical().show(ui, |ui| {
        if !app.readings.is_empty() {
            let label_text = match app.language {
                Language::English => "Short-Term Data Trends (Last 20 Readings)",
                Language::Indonesian => "Tren Data Jangka Pendek (20 Data Terakhir)",
            };
            ui.label(label_text);
            ui.add_space(10.0);
            
            // Siapkan data untuk grafik suhu (ambil 20 data terakhir, balik urutannya).
            let temp_data: Vec<f64> = app.readings.iter().take(20).map(|r| r.temperature).rev().collect();
            // Gambar grafik mini untuk suhu.
            ui.group(|ui| { draw_mini_chart(ui, &temp_data, match app.language {
                Language::English => "Temperature Trend",
                Language::Indonesian => "Tren Suhu"
            }, Color32::RED, "°C"); });
            ui.add_space(10.0);
            
            // Lakukan hal yang sama untuk nilai ADC.
            let adc_data: Vec<f64> = app.readings.iter().take(20).map(|r| r.adc_value as f64).rev().collect();
            ui.group(|ui| { draw_mini_chart(ui, &adc_data, match app.language {
                Language::English => "ADC Value Trend",
                Language::Indonesian => "Tren Nilai ADC"
            }, Color32::BLUE, ""); });
            ui.add_space(10.0);
            
            // Lakukan hal yang sama untuk tegangan.
            let voltage_data: Vec<f64> = app.readings.iter().take(20).map(|r| r.voltage).rev().collect();
            ui.group(|ui| { draw_mini_chart(ui, &voltage_data, match app.language {
                Language::English => "Voltage Trend",
                Language::Indonesian => "Tren Tegangan"
            }, Color32::YELLOW, "V"); });

        } else {
            // Tampilkan pesan jika tidak ada data.
            ui.centered_and_justified(|ui| {
                let no_data_text = match app.language {
                    Language::English => "No data available for visualization.",
                    Language::Indonesian => "Tidak ada data untuk divisualisasikan.",
                };
                ui.label(no_data_text);
            });
        }
    });
}

// --- Halaman Database ---
// Menampilkan data historis dalam bentuk tabel dan menyediakan fungsionalitas ekspor.
pub fn show_database(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Teks UI multibahasa.
    let (title, export_path_label, export_btn, clear_btn, time_h, temp_h, adc_h, volt_h, res_h) = match app.language {
        Language::English => ("Data Recordings", "Export Path:", "Export CSV", "Clear Data", "Time", "Temperature (°C)", "ADC Value", "Voltage (V)", "Resistance (Ω)"),
        Language::Indonesian => ("Rekaman Data", "Lokasi Ekspor:", "Ekspor CSV", "Hapus Data", "Waktu", "Suhu (°C)", "Nilai ADC", "Tegangan (V)", "Resistansi (Ω)"),
    };

    ui.label(egui::RichText::new(title).size(32.0).strong());
    ui.add_space(10.0);

    // Kontrol untuk ekspor dan hapus data.
    ui.horizontal(|ui| {
        ui.label(export_path_label);
        ui.text_edit_singleline(&mut app.export_path); // Input field untuk path file.
        if ui.button(export_btn).clicked() { // Tombol ekspor.
            let (success_msg, fail_msg) = match app.language {
                Language::English => ("Data successfully exported to", "Failed to export data:"),
                Language::Indonesian => ("Data berhasil diekspor ke", "Gagal mengekspor data:"),
            };
            // Panggil fungsi ekspor dan tangani hasilnya.
            match export_data_as_csv(&app.readings, &app.export_path) {
                Ok(_) => {
                    app.export_status_message = Some(format!("{} {}", success_msg, app.export_path));
                }
                Err(e) => {
                    app.export_status_message = Some(format!("{} {}", fail_msg, e));
                }
            }
        }
        if ui.button(clear_btn).clicked() { // Tombol hapus data.
            app.readings.clear(); // Hapus semua data dari VecDeque.
            app.export_status_message = Some(match app.language {
                Language::English => "All data has been cleared.",
                Language::Indonesian => "Semua data telah dihapus.",
            }.to_string());
        }
    });

    // Tampilkan pesan status setelah operasi ekspor/hapus.
    if let Some(message) = &app.export_status_message {
        ui.add_space(10.0);
        let text_color = if message.starts_with("Gagal") || message.starts_with("Failed") {
            Color32::RED
        } else {
            Color32::GREEN
        };
        ui.colored_label(text_color, message);
    }

    ui.add_space(10.0);

    // Gambar header tabel.
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .fill(ui.style().visuals.widgets.hovered.bg_fill)
        .show(ui, |ui| {
            egui::Grid::new("database_header")
                .num_columns(5)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(time_h).strong());
                    ui.label(egui::RichText::new(temp_h).strong());
                    ui.label(egui::RichText::new(adc_h).strong());
                    ui.label(egui::RichText::new(volt_h).strong());
                    ui.label(egui::RichText::new(res_h).strong());
                });
        });
    ui.separator();

    // Gambar isi tabel di dalam ScrollArea.
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("database_grid")
                .num_columns(5)
                .striped(true) // Baris belang-belang.
                .spacing([10.0, 8.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    // Tampilkan 100 data teratas.
                    for reading in app.readings.iter().take(100) {
                        // Format timestamp ke zona waktu WIB dan tampilkan.
                        ui.label(
                            reading
                                .timestamp
                                .with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap())
                                .format("%H:%M:%S")
                                .to_string(),
                        );
                        ui.label(format!("{:.1}", reading.temperature));
                        ui.label(format!("{}", reading.adc_value));
                        ui.label(format!("{:.2}", reading.voltage));
                        ui.label(format!("{:.0}", reading.resistance));
                        ui.end_row(); // Akhiri baris grid.
                    }
                });
        });
}

// --- Halaman Konfigurasi Sensor ---
// Memungkinkan pengguna melihat dan mengubah karakteristik sensor.
pub fn show_sensor_characteristics(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Teks UI multibahasa.
    let (title, type_label, model_label, acc_label, range_min_label, range_max_label, cal_label, recal_btn) = match app.language {
        Language::English => ("Sensor Configuration", "Sensor Type:", "Model:", "Accuracy:", "Operating Range (Min):", "Operating Range (Max):", "Last Calibration:", "Recalibrate Sensor"),
        Language::Indonesian => ("Konfigurasi Sensor", "Tipe Sensor:", "Model:", "Akurasi:", "Rentang Operasi (Min):", "Rentang Operasi (Maks):", "Kalibrasi Terakhir:", "Kalibrasi Ulang Sensor"),
    };

    ui.label(egui::RichText::new(title).size(32.0).strong());
    ui.add_space(30.0);
    ui.group(|ui| {
        ui.vertical(|ui| {
            // Gunakan Grid untuk menata label dan input field secara rapi.
            egui::Grid::new("char_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    ui.label(type_label);
                    ui.text_edit_singleline(&mut app.characteristics.sensor_type);
                    ui.end_row();
                    ui.label(model_label);
                    ui.text_edit_singleline(&mut app.characteristics.model);
                    ui.end_row();
                    ui.label(acc_label);
                    // `DragValue` adalah input numerik yang bisa di-drag.
                    ui.add(
                        egui::DragValue::new(&mut app.characteristics.accuracy)
                            .speed(0.01)
                            .suffix("°C/%"),
                    );
                    ui.end_row();
                    ui.label(range_min_label);
                    ui.add(
                        egui::DragValue::new(&mut app.characteristics.range_min)
                            .speed(0.1)
                            .suffix(" units"),
                    );
                    ui.end_row();
                    ui.label(range_max_label);
                    ui.add(
                        egui::DragValue::new(&mut app.characteristics.range_max)
                            .speed(0.1)
                            .suffix(" units"),
                    );
                    ui.end_row();
                    ui.label(cal_label);
                    // Tampilkan tanggal kalibrasi terakhir (hanya baca).
                    ui.label(
                        app.characteristics.calibration_date.format("%Y-%m-%d").to_string(),
                    );
                    ui.end_row();
                });
            ui.add_space(20.0);
            // Tombol untuk "mengkalibrasi ulang" (memperbarui tanggal ke waktu saat ini).
            if ui.button(recal_btn).clicked() {
                app.characteristics.calibration_date = chrono::Utc::now();
            }
        });
    });
}

// --- Halaman Monitor Suhu ---
// Halaman khusus untuk detail dan statistik terkait suhu.
pub fn show_temperature_page(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Teks UI multibahasa.
    let (title, current_label, stats_label, min_label, max_label, avg_label, alerts_label, min_temp_label, max_temp_label) = match app.language {
        Language::English => ("Temperature Monitor", "Current Temperature", "Statistics", "Min:", "Max:", "Avg:", "Temperature Alerts", "Min Temperature (°C)", "Max Temperature (°C)"),
        Language::Indonesian => ("Monitor Suhu", "Suhu Saat Ini", "Statistik", "Min:", "Maks:", "Rerata:", "Peringatan Suhu", "Suhu Minimum (°C)", "Suhu Maksimum (°C)"),
    };

    ui.label(egui::RichText::new(title).size(32.0).strong());
    ui.add_space(30.0);
    // Tampilkan hanya jika ada data.
    if let Some(latest) = app.readings.front() {
        ui.horizontal(|ui| {
            // Kotak untuk menampilkan suhu saat ini.
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(current_label).size(16.0));
                    ui.add_space(10.0);
                    // Suhu saat ini dengan font besar dan warna merah.
                    ui.label(
                        egui::RichText::new(format!("{:.1}°C", latest.temperature))
                            .size(36.0)
                            .strong()
                            .color(Color32::from_rgb(239, 68, 68)),
                    );
                    ui.add_space(10.0);
                    // Teks status (Low, High, Normal).
                    let (status_text, status_color) =
                        if latest.temperature < app.alert_thresholds.temp_min {
                            ("Low", Color32::BLUE)
                        } else if latest.temperature > app.alert_thresholds.temp_max {
                            ("High", Color32::RED)
                        } else {
                            ("Normal", Color32::GREEN)
                        };
                    ui.colored_label(status_color, status_text);
                });
            });
            ui.add_space(30.0);
            // Kotak untuk menampilkan statistik (Min, Max, Rata-rata).
            if !app.readings.is_empty() {
                // Hitung statistik dari semua data yang ada.
                let temps: Vec<f64> = app.readings.iter().map(|r| r.temperature).collect();
                let min_temp = temps.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_temp = temps.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let avg_temp = temps.iter().sum::<f64>() / temps.len() as f64;
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(stats_label).strong());
                        ui.add_space(10.0);
                        ui.label(format!("{} {:.1}°C", min_label, min_temp));
                        ui.label(format!("{} {:.1}°C", max_label, max_temp));
                        ui.label(format!("{} {:.1}°C", avg_label, avg_temp));
                    });
                });
            }
        });
    }
    ui.add_space(40.0);
    // Kotak untuk mengatur threshold peringatan suhu.
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(alerts_label).strong());
            ui.add_space(10.0);
            // Slider untuk mengatur suhu minimum.
            ui.add(
                egui::Slider::new(&mut app.alert_thresholds.temp_min, -20.0..=50.0)
                    .text(min_temp_label),
            );
            ui.add_space(5.0);
            // Slider untuk mengatur suhu maksimum.
            ui.add(
                egui::Slider::new(&mut app.alert_thresholds.temp_max, -20.0..=100.0)
                    .text(max_temp_label),
            );
        });
    });
}

// --- Halaman Konfigurasi Port Serial ---
// Halaman untuk memilih, menghubungkan, dan memutuskan koneksi dari port serial.
pub fn show_serial_config_page(app: &mut SensorApp, ui: &mut egui::Ui) {
    // Teks UI multibahasa.
    let (_, available_label, select_label, refresh_btn, connect_btn, disconnect_btn, _status_initial, status_refreshed, _status_connecting, status_disconnected) = match app.language {
        Language::English => ("Serial Port Configuration", "Available Ports:", "Select a port", "Refresh", "Connect", "Disconnect", "Please select a port and connect.", "Ports refreshed. Please select a port.", "Connecting to {}...", "Disconnected. Select a port to connect."),
        Language::Indonesian => ("Konfigurasi Port Serial", "Port Tersedia:", "Pilih port", "Segarkan", "Hubungkan", "Putuskan", "Silakan pilih port dan hubungkan.", "Port disegarkan. Silakan pilih port.", "Menghubungkan ke {}...", "Terputus. Pilih port untuk terhubung."),
    };
    
    // Kloning `Sender` untuk dikirim ke thread baru.
    let sensor_tx = app.sensor_tx.clone();
    
    let title = match app.language {
        Language::English => "Serial Port Configuration",
        Language::Indonesian => "Konfigurasi Port Serial"
    };

    ui.label(egui::RichText::new(title).size(32.0).strong());
    ui.add_space(20.0);

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label(available_label);
                // Teks yang ditampilkan di ComboBox saat tidak ada yang dipilih.
                let selected_text = app.selected_port.as_deref().unwrap_or(select_label);
                // `ComboBox` untuk memilih port serial.
                ComboBox::from_id_source("port_select")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        // Iterasi daftar port yang tersedia dan buat item yang bisa dipilih.
                        for port_name in &app.available_ports {
                            ui.selectable_value(&mut app.selected_port, Some(port_name.clone()), port_name);
                        }
                    });

                if ui.button(refresh_btn).clicked() {
                    // Pindai ulang port serial yang tersedia.
                    app.available_ports = serialport::available_ports()
                        .unwrap_or_default()
                        .iter()
                        .map(|p| p.port_name.clone())
                        .collect();
                    app.connection_status_message = status_refreshed.to_string();
                }
            });

            ui.add_space(15.0);

            ui.horizontal(|ui| {
                // Tombol "Connect" hanya aktif jika port dipilih DAN tidak ada koneksi aktif.
                let connect_enabled = app.selected_port.is_some() && app.serial_thread_handle.is_none();
                if ui.add_enabled(connect_enabled, egui::Button::new(connect_btn)).clicked() {
                    if let Some(port_name) = app.selected_port.clone() {
                        // Buat channel kontrol baru untuk thread baru ini.
                        let (control_tx, control_rx) = mpsc::channel();
                        app.control_tx = Some(control_tx);
                        
                        let new_sensor_tx = sensor_tx.clone();
                        
                        // Buat (spawn) thread serial baru.
                        app.serial_thread_handle = Some(thread::spawn(move || {
                            serial_thread::run(port_name, new_sensor_tx, control_rx);
                        }));

                        // Perbarui pesan status koneksi.
                        if let Some(port_name) = app.selected_port.as_ref() {
                            app.connection_status_message = match app.language {
                                Language::English => format!("Connecting to {}...", port_name),
                                Language::Indonesian => format!("Menyambungkan ke {}...", port_name),
                            };              
                        }
                        // Hapus data lama saat koneksi baru dibuat.
                        app.readings.clear();
                    }
                }

                // Tombol "Disconnect" hanya aktif jika ada koneksi.
                let disconnect_enabled = app.serial_thread_handle.is_some();
                if ui.add_enabled(disconnect_enabled, egui::Button::new(disconnect_btn)).clicked() {
                    // Kirim sinyal Shutdown ke thread serial.
                    if let Some(tx) = &app.control_tx {
                        let _ = tx.send(SerialControl::Shutdown);
                    }
                    // Tunggu thread selesai (join).
                    if let Some(handle) = app.serial_thread_handle.take() {
                        let _ = handle.join();
                    }
                    // Reset semua state terkait koneksi.
                    app.serial_thread_handle = None;
                    app.control_tx = None;
                    app.is_connected = false;
                    app.connection_status_message = status_disconnected.to_string();
                }
            });
            
            ui.add_space(10.0);
            // Tampilkan pesan status koneksi saat ini.
            ui.label(&app.connection_status_message);

        });
    });
}

// --- Halaman Analisis Numerik ---
// Halaman untuk membandingkan hasil perhitungan suhu secara analitik vs. numerik.
pub fn show_numerical_analysis_page(app: &mut SensorApp, ui: &mut egui::Ui) {
     // Teks UI multibahasa
    let (title, desc1, desc2, desc3, graph_title, legend_red, legend_blue, results_label, res1_label, res2_label, diff_label, iter_label, no_data) = match app.language {
        Language::English => (
            "Numerical Method Verification", 
            "This page aims to verify the accuracy of the numerical method (Newton-Raphson) we are using.",
            "Verification is performed by directly comparing its results against the analytical solution (exact formula).", 
            "If both lines on the graph overlap, it proves that our numerical method implementation is highly accurate.",
            "Temperature Comparison: Analytical vs. Numerical Solution",
            "Red Line: Temp. from Analytical Solution (Exact)", 
            "Blue Line: Temp. from Numerical Method (Approximation)", 
            "Latest Calculation Results:", "Analytical Result:", "Numerical Result:", "Difference (Error):", "Iterations to Converge:", "No data available for analysis."
        ),
        Language::Indonesian => (
            "Verifikasi Metode Numerik",
            "Halaman ini bertujuan untuk memverifikasi akurasi dari metode numerik (Newton-Raphson) yang kita gunakan.",
            "Verifikasi dilakukan dengan membandingkan hasilnya secara langsung dengan solusi analitik (rumus pasti) dari model sensor.",
            "Jika kedua garis pada grafik tumpang tindih, maka implementasi metode numerik kita terbukti sangat akurat.",
            "Perbandingan Suhu: Solusi Analitik vs. Solusi Numerik",
            "Garis Merah: Suhu dari Solusi Analitik (Eksak)",
            "Garis Biru: Suhu dari Metode Numerik (Aproksimasi)",
            "Hasil Kalkulasi Terkini:", "Hasil Analitis:", "Hasil Numerik:", "Selisih (Error):", "Iterasi untuk Konvergen:", "Tidak ada data untuk dianalisis."
        ),
    };

    // Tampilkan teks deskripsi.
    ui.label(egui::RichText::new(title).size(32.0).strong());
    ui.add_space(10.0);
    ui.label(desc1);
    ui.label(desc2);
    ui.label(egui::RichText::new(desc3).strong());
    ui.add_space(20.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        if !app.readings.is_empty() {
            // Siapkan data untuk grafik: 200 data terakhir, urutan dibalik.
            let numerical_data : Vec<f64> = app.analytical_temps_from_r.iter().take(200).cloned().rev().collect();
            let analytical_data: Vec<f64> = app.numerical_temps_from_r.iter().take(200).map(|r| r.temperature).rev().collect();
            
            // Panggil komponen untuk menggambar grafik perbandingan.
            draw_comparison_chart(
                ui,
                graph_title,
                &numerical_data,  // Data untuk garis pertama (seharusnya numerik).
                legend_red,       // Legenda untuk garis pertama.
                Color32::from_rgb(60, 130, 246), // WARNA BIRU.
                &analytical_data, // Data untuk garis kedua (seharusnya analitik).
                legend_blue,      // Legenda untuk garis kedua.
                Color32::from_rgb(239, 68, 68)  // WARNA MERAH.
            );
            
            ui.add_space(20.0);

            // Tampilkan hasil perhitungan terakhir dalam bentuk tabel.
            ui.label(results_label);
            ui.group(|ui| {
                if let (Some(analytical), Some(numerical)) = (app.analytical_temps_from_r.front(), app.numerical_temps_from_r.front()) {
                    egui::Grid::new("results_grid").num_columns(2).show(ui, |ui| {
                        ui.label(res1_label); // "Hasil Analitis:"
                        ui.label(format!("{:.4} °C", analytical));
                        ui.end_row();

                        ui.label(res2_label); // "Hasil Numerik:"
                        ui.label(format!("{:.4} °C", numerical.temperature));
                        ui.end_row();
                        
                        ui.label(diff_label); // "Selisih (Error):"
                        ui.label(format!("{:.1E}", (analytical - numerical.temperature).abs())); // Format eksponensial.
                        ui.end_row();

                        ui.label(iter_label); // "Iterasi:"
                        ui.label(format!("{}", numerical.iterations));
                        ui.end_row();
                    });
                }
            });

        } else {
            // Tampilkan pesan jika tidak ada data.
            ui.centered_and_justified(|ui| {
                ui.label(no_data);
            });
        }
    });
}