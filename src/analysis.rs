// Struktur untuk menyimpan hasil dari kalkulasi metode numerik.
// Ini memungkinkan kita mengembalikan beberapa nilai (suhu, iterasi, error) dari satu fungsi.
// `#[derive(Debug, Clone, Copy)]` memungkinkan struct ini untuk mudah diduplikasi dan di-debug.
#[derive(Debug, Clone, Copy)]
pub struct CalculationResult {
    pub temperature: f64, // Suhu hasil perhitungan dalam Celsius.
    pub iterations: u32,  // Jumlah iterasi yang diperlukan hingga solusi konvergen.
    pub error: f64,       // Nilai error terakhir, menunjukkan seberapa dekat f(T) ke nol.
}

// Implementasi metode Newton-Raphson untuk mencari suhu (T) dari persamaan Steinhart-Hart.
// Tujuannya adalah mencari akar dari persamaan f(T) = 0.
pub fn newton_raphson_temperature(
    resistance: f64,           // (R) Resistansi NTC terukur, sebagai input.
    beta: f64,                 // (B) Koefisien Beta dari datasheet thermistor.
    r_nominal: f64,            // (R0) Resistansi nominal pada suhu nominal.
    t_nominal_kelvin: f64,     // (T0) Suhu nominal dalam Kelvin.
) -> Option<CalculationResult> { // Mengembalikan `Option` karena metode ini bisa gagal konvergen.

    // Mendefinisikan f(T), yaitu persamaan Steinhart-Hart yang diatur ulang agar sama dengan nol.
    // f(T) = (1/T0) + (1/B) * ln(R/R0) - (1/T) = 0
    // Kita mencari nilai T yang membuat persamaan ini benar.
    let f = |t_kelvin: f64| -> f64 {
        (1.0 / t_nominal_kelvin) + (1.0 / beta) * (resistance / r_nominal).ln() - (1.0 / t_kelvin)
    };

    // Mendefinisikan turunan dari f(T) terhadap T, yaitu f'(T).
    // Turunan dari -1/T adalah 1/T^2.
    let df = |t_kelvin: f64| -> f64 {
        1.0 / t_kelvin.powi(2)
    };

    // Iterasi Newton-Raphson: T_next = T_current - f(T_current) / f'(T_current)

    let mut t_current = t_nominal_kelvin; // Tebakan awal untuk suhu adalah suhu nominal.
    let tolerance = 1e-6; // Toleransi error. Jika perubahan antar iterasi lebih kecil dari ini, kita berhenti.
    let max_iterations = 20; // Batas maksimum iterasi untuk mencegah loop tak terbatas.

    // Loop iterasi dari 0 hingga maksimum.
    for i in 0..max_iterations {
        let fx = f(t_current);
        let dfx = df(t_current);

        // Pencegahan pembagian dengan nol jika turunan sangat kecil.
        if dfx.abs() < 1e-10 {
            return None; // Gagal, tidak bisa melanjutkan.
        }

        // Hitung tebakan suhu berikutnya menggunakan formula Newton-Raphson.
        let t_next = t_current - fx / dfx;

        // Cek konvergensi: jika selisih antara tebakan baru dan lama sangat kecil, solusi ditemukan.
        if (t_next - t_current).abs() < tolerance {
            // Jika konvergen, kembalikan hasilnya dalam struct `CalculationResult`.
            return Some(CalculationResult {
                temperature: t_next - 273.15, // Konversi suhu dari Kelvin ke Celsius.
                iterations: i + 1,            // Jumlah iterasi yang telah dilakukan.
                error: fx,                    // Nilai f(T) terakhir (seharusnya sangat dekat dengan 0).
            });
        }
        // Jika belum konvergen, perbarui tebakan saat ini untuk iterasi berikutnya.
        t_current = t_next;
    }

    // Jika loop selesai tanpa konvergen, kembalikan `None`.
    None
}