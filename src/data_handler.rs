// Mengimpor tipe `SensorReading` agar tahu struktur data yang akan diekspor.
use crate::types::SensorReading;
// Mengimpor VecDeque untuk menangani koleksi data.
use std::collections::VecDeque;
// Mengimpor `File` untuk membuat file baru.
use std::fs::File;
// Mengimpor `IoError` untuk penanganan error I/O dan `Write` trait untuk menulis ke file.
use std::io::{Error as IoError, Write};

// Fungsi untuk mengekspor data dari VecDeque<SensorReading> ke sebuah file CSV.
pub fn export_data_as_csv(
    readings: &VecDeque<SensorReading>, // Referensi ke data sensor.
    export_path: &str,                  // Path (lokasi dan nama file) tujuan ekspor.
) -> Result<(), IoError> { // Mengembalikan Result, Ok(()) jika berhasil, Err(IoError) jika gagal.
    
    // Mencoba membuat file baru di path yang ditentukan. Tanda `?` akan langsung
    // mengembalikan error jika `File::create` gagal.
    let mut file = File::create(export_path)?;
    
    // Menulis baris header ke dalam file CSV. `writeln!` adalah makro yang
    // menambahkan newline secara otomatis di akhir.
    writeln!(file, "Timestamp,Temperature,ADCValue,Voltage,Resistance")?;
    
    // Melakukan iterasi (loop) pada setiap `reading` di dalam `VecDeque`.
    for reading in readings {
        // Menulis setiap field dari struct `reading` ke dalam file, dipisahkan oleh koma.
        writeln!(
            file,
            "{},{},{},{},{}",
            reading.timestamp.to_rfc3339(), // Format timestamp ke string standar ISO 8601.
            reading.temperature,
            reading.adc_value,
            reading.voltage,
            reading.resistance
        )?;
    }
    // Jika semua proses tulis berhasil, kembalikan Ok.
    Ok(())
}