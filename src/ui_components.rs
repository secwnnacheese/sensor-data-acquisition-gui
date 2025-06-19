// Mengimpor komponen dan tipe data yang dibutuhkan dari egui.
use eframe::egui::{self, Color32, Vec2, Stroke, pos2};

// Fungsi untuk menggambar kartu modern. 
// Catatan: Fungsi ini ada di kode, tapi tidak dipanggil secara aktif di file pages.rs.
// `pages.rs` menggunakan `draw_info_card` yang didefinisikan secara lokal di `show_home_page`.
pub fn draw_modern_card(ui: &mut egui::Ui, title: &str, value: &str, status_text_str: &str, status_color: egui::Color32) {
    // Definisikan ukuran kartu.
    let card_size = egui::Vec2::new(200.0, 120.0);
    // Alokasikan ruang di UI untuk kartu ini. `Sense::hover` mendeteksi saat mouse di atasnya.
    let (rect, response) = ui.allocate_exact_size(card_size, egui::Sense::hover());

    // Hanya gambar jika persegi panjang kartu terlihat di layar (optimisasi).
    if ui.is_rect_visible(rect) {
        // Gambar latar belakang dan bingkai kartu.
        ui.painter().rect(
            rect,
            egui::Rounding::same(12.0), // Sudut melengkung.
            // Ubah warna latar belakang saat di-hover.
            if response.hovered() { ui.style().visuals.widgets.hovered.bg_fill.gamma_multiply(1.2) } else { ui.style().visuals.widgets.inactive.bg_fill },
            egui::Stroke::new(1.0, ui.style().visuals.widgets.inactive.bg_stroke.color), // Bingkai tipis.
        );

        // Buat area konten di dalam kartu dengan margin.
        let content_rect = rect.shrink(16.0);
        ui.allocate_ui_at_rect(content_rect, |ui_card_content| {
            // Susun konten secara vertikal.
            ui_card_content.vertical(|ui_vertical_layout| {
                // Gambar judul.
                ui_vertical_layout.label(egui::RichText::new(title).size(14.0).color(ui_vertical_layout.style().visuals.text_color().gamma_multiply(0.7)));
                ui_vertical_layout.add_space(8.0);
                // Gambar nilai utama.
                ui_vertical_layout.label(egui::RichText::new(value).size(28.0).strong().color(ui_vertical_layout.style().visuals.text_color()));
                ui_vertical_layout.add_space(8.0);

                // Buat frame berbentuk kapsul untuk status.
                let capsule_frame = egui::Frame::none()
                    .fill(status_color)
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin { left: 8.0, right: 8.0, top: 4.0, bottom: 4.0 });

                // Tampilkan konten di dalam kapsul.
                capsule_frame.show(ui_vertical_layout, |ui_capsule_content| {
                    ui_capsule_content.horizontal(|ui_inner_capsule| {
                        // Gambar titik putih di dalam kapsul.
                        let dot_size = 8.0;
                        let (dot_rect, _) = ui_inner_capsule.allocate_exact_size(Vec2::splat(dot_size), egui::Sense::hover());
                        ui_inner_capsule.painter().circle_filled(dot_rect.center(), dot_size / 2.0, Color32::WHITE);
                        // Tampilkan teks status.
                        ui_inner_capsule.label(egui::RichText::new(status_text_str).size(12.0).color(Color32::WHITE));
                    });
                });
            });
        });
    }
}

// Fungsi untuk menggambar grafik garis mini (sparkline).
pub fn draw_mini_chart(ui: &mut egui::Ui, data: &[f64], title: &str, color: egui::Color32, unit: &str) {
    let chart_height = 60.0;
    // Lebar grafik mengisi ruang yang tersedia.
    let chart_width = ui.available_width().max(300.0); 

    // `ui.group` memberikan latar belakang dan bingkai untuk grup widget.
    ui.group(|ui| { 
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).strong());
            if !data.is_empty() {
                // Alokasikan ruang untuk area gambar grafik.
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(chart_width, chart_height), egui::Sense::hover());
                // Cari nilai minimum dan maksimum dari data untuk penskalaan sumbu Y.
                let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let range = if max_val > min_val { max_val - min_val } else { 1.0 }; // Hindari pembagian dengan nol.
                
                // Konversi data numerik menjadi titik-titik koordinat (Pos2) pada UI.
                let points: Vec<egui::Pos2> = data.iter().enumerate().map(|(i, &val)| {
                        // Sumbu X: dipetakan dari indeks array ke lebar persegi.
                        let x = rect.left() + (i as f32 / (data.len() - 1).max(1) as f32) * rect.width();
                        // Sumbu Y: normalisasikan nilai data ke rentang [0, 1], lalu petakan ke tinggi persegi.
                        let normalized_y = ((val - min_val) / range) as f32;
                        let y = rect.bottom() - normalized_y * rect.height(); // Sumbu Y egui terbalik (0 di atas).
                        egui::Pos2::new(x, y)
                    }).collect();

                // Gambar latar belakang area grafik.
                ui.painter().rect_filled(rect, egui::Rounding::same(4.0), ui.style().visuals.extreme_bg_color);
                
                // Jika ada lebih dari satu titik, gambar garis yang menghubungkannya.
                if points.len() > 1 {
                    for i in 0..points.len() - 1 {
                        ui.painter().line_segment([points[i], points[i + 1]], egui::Stroke::new(2.0, color));
                    }
                }
                // Gambar lingkaran kecil di setiap titik data.
                for point in &points {
                    ui.painter().circle_filled(*point, 2.0, color);
                }
                // Tampilkan nilai data terakhir di bawah grafik.
                if let Some(&current_val) = data.last() {
                    ui.label(egui::RichText::new(format!("Current: {:.1}{}", current_val, unit)).size(12.0).color(ui.style().visuals.text_color().gamma_multiply(0.7)));
                }
            } else {
                ui.label("No data to display.");
            }
        });
    });
}

// Fungsi untuk menggambar grafik yang membandingkan dua set data.
pub fn draw_comparison_chart(
    ui: &mut egui::Ui,
    title: &str,
    data1: &[f64],
    name1: &str,
    color1: Color32,
    data2: &[f64],
    name2: &str,
    color2: Color32,
) {
    let chart_height = 250.0;
    let chart_width = ui.available_width().max(600.0);

    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).size(18.0).strong());
            ui.add_space(10.0);

            // Gambar legenda untuk menjelaskan setiap garis.
            ui.horizontal(|ui| {
                // Legenda untuk data 1
                ui.painter().line_segment([pos2(0.0,0.0), pos2(10.0,0.0)], Stroke::new(2.0, color1));
                ui.add_space(15.0);
                ui.label(name1);
                ui.add_space(30.0);
                // Legenda untuk data 2
                ui.painter().line_segment([pos2(0.0,0.0), pos2(10.0,0.0)], Stroke::new(2.0, color2));
                ui.add_space(15.0);
                ui.label(name2);
            });
            ui.add_space(5.0);
            
            if data1.is_empty() || data2.is_empty() {
                ui.label("Waiting for data...");
                return;
            }

            // Alokasikan ruang untuk area grafik.
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(chart_width, chart_height), egui::Sense::hover());
            ui.painter().rect_filled(rect, egui::Rounding::same(4.0), ui.style().visuals.extreme_bg_color);
            
            // Cari nilai min/max dari KEDUA set data agar skala sumbu Y konsisten.
            let combined_data = data1.iter().chain(data2.iter());
            let min_val = combined_data.clone().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = combined_data.fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let range = if max_val > min_val { max_val - min_val } else { 1.0 };

            // Fungsi helper untuk menggambar satu garis dari sekumpulan titik.
            let draw_line = |points: Vec<egui::Pos2>, color: Color32| {
                if points.len() > 1 {
                    ui.painter().add(egui::Shape::line(points, Stroke::new(2.0, color)));
                }
            };
            
            // Konversi dan gambar garis untuk data pertama (numerik).
            let points1: Vec<egui::Pos2> = data1.iter().enumerate().map(|(i, &val)| {
                let x = rect.left() + (i as f32 / (data1.len() - 1).max(1) as f32) * rect.width();
                let normalized_y = ((val - min_val) / range) as f32;
                let y = rect.bottom() - normalized_y * rect.height(); 
                egui::Pos2::new(x, y)
            }).collect();
            draw_line(points1, color1);

            // Konversi dan gambar garis untuk data kedua (analitis).
            let points2: Vec<egui::Pos2> = data2.iter().enumerate().map(|(i, &val)| {
                let x = rect.left() + (i as f32 / (data2.len() - 1).max(1) as f32) * rect.width();
                let normalized_y = ((val - min_val) / range) as f32;
                let y = rect.bottom() - normalized_y * rect.height(); 
                egui::Pos2::new(x, y)
            }).collect();
            draw_line(points2, color2);
        });
    });
}