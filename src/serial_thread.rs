// Mengimpor tipe `SensorReading` untuk membuat instance data baru.
use crate::types::SensorReading;
// Mengimpor `Utc` dari `chrono` untuk memberikan timestamp pada data.
use chrono::Utc;
// Mengimpor trait dari `std::io` untuk membaca data baris per baris.
use std::io::{self, BufRead};
// Mengimpor `Receiver` dan `Sender` untuk komunikasi antar-thread.
use std::sync::mpsc::{Receiver, Sender};
// Mengimpor `Duration` untuk mengatur timeout pada pembacaan port serial.
use std::time::Duration;

// Enum ini mendefinisikan pesan kontrol yang bisa dikirim ke thread serial.
pub enum SerialControl {
    Shutdown, // Sinyal untuk memberitahu thread agar berhenti dan membersihkan resource.
}

// Fungsi utama yang akan dijalankan dalam sebuah thread terpisah.
// Menerima nama port, `Sender` untuk mengirim data ke thread utama, dan `Receiver` untuk menerima sinyal kontrol.
pub fn run(port_name: String, tx: Sender<SensorReading>, control_rx: Receiver<SerialControl>) {
    println!("[SERIAL THREAD] Attempting to connect to port: '{}'", port_name);

    // Mencoba untuk membuka koneksi ke port serial dengan konfigurasi yang diberikan.
    if let Ok(port) = serialport::new(&port_name, 9600) // Baudrate 9600 bps, umum untuk Arduino.
        .timeout(Duration::from_millis(100)) // Set timeout agar `read_line` tidak memblokir selamanya.
        .open()
    {
        println!("[SERIAL THREAD] Successfully opened serial port '{}'", port_name);
        // Bungkus port dengan `BufReader` untuk efisiensi pembacaan baris per baris.
        let mut reader = io::BufReader::new(port);

        // Loop utama thread untuk terus menerus membaca data.
        loop {
            // Cek apakah ada sinyal kontrol (misalnya `Shutdown`) dari thread utama.
            // `try_recv` tidak memblokir, sehingga loop bisa terus berjalan.
            if let Ok(SerialControl::Shutdown) = control_rx.try_recv() {
                println!("[SERIAL THREAD] Shutdown signal received. Closing port.");
                break; // Keluar dari loop untuk menghentikan thread.
            }

            let mut line_buf = String::new();
            // Mencoba membaca satu baris data dari port serial (diakhiri dengan newline).
            match reader.read_line(&mut line_buf) {
                // Jika berhasil membaca (bahkan jika kosong), proses datanya.
                Ok(_) => {
                    // Panggil fungsi `parse_and_send` untuk mem-parsing dan mengirim data.
                    // `trim()` menghapus spasi atau newline di awal/akhir string.
                    let _ = parse_and_send(line_buf.trim(), &tx);
                }
                // Jika error adalah `TimedOut`, ini normal. Cukup lanjutkan loop.
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                // Jika terjadi error lain (misal: perangkat dicabut), cetak error dan keluar dari loop.
                Err(e) => {
                    eprintln!("[SERIAL THREAD] Failed to read from port: {}. Closing connection.", e);
                    break;
                }
            }
        }
    } else {
        // Jika gagal membuka port di awal, cetak pesan error.
        eprintln!("[SERIAL THREAD] Failed to open serial port '{}'. Thread will exit.", port_name);
        // Kirim data default (kosong) untuk memberi sinyal ke thread utama bahwa koneksi gagal.
        let _ = tx.send(SensorReading::default());
    }

    println!("[SERIAL THREAD] Thread for port '{}' is terminating.", port_name);
}

// Fungsi untuk mem-parsing string data dari sensor dan mengirimkannya melalui channel.
fn parse_and_send(line: &str, tx: &Sender<SensorReading>) -> Result<(), ()> {
    // Jika baris kosong, abaikan.
    if line.is_empty() { return Ok(()); }
    
    // Pisahkan string berdasarkan koma. Contoh format: "25.5,512,2.50,10000.0"
    let parts: Vec<&str> = line.split(',').collect();
    // Pastikan ada tepat 4 bagian data.
    if parts.len() == 4 {
        // Coba parsing setiap bagian ke tipe data yang sesuai.
        if let (Ok(temp), Ok(adc), Ok(volt), Ok(res)) = (
            parts[0].parse::<f64>(), // Suhu
            parts[1].parse::<u16>(), // Nilai ADC (unsigned 16-bit integer)
            parts[2].parse::<f64>(), // Tegangan
            parts[3].parse::<f64>()  // Resistansi
        ) {
            // Jika semua parsing berhasil, buat struct `SensorReading` baru.
            let reading = SensorReading {
                timestamp: Utc::now(), // Gunakan waktu saat ini sebagai timestamp.
                temperature: temp, 
                adc_value: adc,
                voltage: volt,
                resistance: res,
            };
            // Kirim struct `reading` ke thread utama melalui channel `tx`.
            // `map_err` mengubah error `SendError` menjadi `()` agar sesuai tipe return fungsi.
            return tx.send(reading).map_err(|_| ());
        }
    }
    // Jika format tidak sesuai atau parsing gagal, kembalikan error.
    Err(())
}