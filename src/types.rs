// Mengimpor tipe data yang dibutuhkan dari library lain.
use chrono::{DateTime, Utc}; // Untuk timestamp dengan zona waktu.
use serde::{Deserialize, Serialize}; // Untuk serialisasi dan deserialisasi data (misal: ke/dari BSON untuk MongoDB).

// Struct untuk merepresentasikan satu pembacaan data dari sensor.
// `#[derive(...)]` secara otomatis mengimplementasikan trait-trait standar.
// `Serialize`, `Deserialize`: untuk konversi ke format lain seperti JSON/BSON.
// `Clone`, `Debug`, `Default`: trait-trait utilitas umum di Rust.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SensorReading {
    pub timestamp: DateTime<Utc>, // Waktu pembacaan data.
    pub temperature: f64,         // Suhu dalam Celsius.
    pub adc_value: u16,           // Nilai mentah dari Analog-to-Digital Converter.
    pub voltage: f64,             // Tegangan yang terukur.
    pub resistance: f64,          // Resistansi yang terhitung.
}

// Struct untuk menyimpan karakteristik atau metadata dari sensor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorCharacteristics {
    pub sensor_type: String, // Tipe sensor, misal: "NTC Thermistor".
    pub model: String,       // Model atau platform, misal: "Arduino Nano".
    pub accuracy: f64,       // Tingkat akurasi sensor.
    pub range_min: f64,      // Rentang operasi minimum.
    pub range_max: f64,      // Rentang operasi maksimum.
    pub calibration_date: DateTime<Utc>, // Tanggal kalibrasi terakhir.
}

// Struct untuk menyimpan batas ambang (threshold) untuk peringatan (alert).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub temp_min: f64, // Batas suhu minimum.
    pub temp_max: f64, // Batas suhu maksimum.
}

// Enum untuk merepresentasikan halaman-halaman yang ada di UI.
// Ini digunakan untuk navigasi dan routing tampilan di `app.rs`.
// `PartialEq` diperlukan untuk membandingkan halaman (misal: `app.current_page == Page::Home`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Page {
    Home,
    DataGraphics,
    Database,
    SensorCharacteristics,
    Temperature,
    SerialPort,
    NumericalAnalysis, // Halaman untuk verifikasi metode numerik.
}

// Enum untuk tema tampilan UI (terang atau gelap).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

// Enum untuk pilihan bahasa pada UI.
// `Copy` memungkinkan tipe ini disalin dengan mudah (karena ukurannya kecil).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Language {
    English,
    Indonesian,
}

// Enum untuk menentukan tipe grafik tren yang akan digambar.
// `Eq` adalah versi lebih ketat dari `PartialEq`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrendGraphType {
    Temperature,
    AdcValue,
    Voltage,
}