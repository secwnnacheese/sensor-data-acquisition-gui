// Mengimpor tipe data yang akan disimpan.
use crate::types::SensorReading;
// Mengimpor komponen dari library `mongodb`. `doc!` untuk membuat dokumen BSON.
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
// Mengimpor `Receiver` untuk menerima data dari thread utama.
use std::sync::mpsc::Receiver;
// Mengimpor `Runtime` dari Tokio untuk menjalankan kode asinkron.
use tokio::runtime::Runtime;

// Mendefinisikan konstanta untuk nama database dan collection.
const DB_NAME: &str = "sensor_db";
const COLLECTION_NAME: &str = "readings";

// Fungsi utama yang dijalankan di dalam thread database.
pub fn run_db_writer(rx: Receiver<SensorReading>) {
    // Membuat runtime Tokio baru untuk mengeksekusi fungsi-fungsi `async`.
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            // Jika gagal membuat runtime, program tidak bisa lanjut.
            eprintln!("Failed to create tokio runtime: {}", e);
            return;
        }
    };

    // Memblokir thread saat ini untuk menjalankan `future` asinkron di dalam runtime Tokio.
    rt.block_on(async {
        // Memanggil fungsi `async` untuk menghubungkan ke database.
        let collection = match connect_to_db().await {
            Ok(coll) => coll,
            Err(_) => return, // Jika gagal konek, hentikan eksekusi.
        };
        println!("Successfully connected to MongoDB. Waiting for data to store...");

        // Loop tak terbatas yang akan menunggu data masuk dari channel `rx`.
        // `rx.iter()` akan memblokir thread sampai ada data yang bisa diterima.
        for received_data in rx.iter() {
            // Memanggil fungsi `async` untuk memasukkan data ke collection.
            if let Err(e) = insert_reading(&collection, received_data).await {
                eprintln!("Failed to store data in MongoDB: {}", e);
            } else {
                // Konfirmasi bahwa data berhasil disimpan.
                println!("[DB THREAD] Data successfully stored in MongoDB.");
            }
        }
    });
}

// Fungsi asinkron untuk menginisialisasi koneksi ke MongoDB.
async fn connect_to_db() -> Result<Collection<SensorReading>, mongodb::error::Error> {
    // URI koneksi standar untuk instance MongoDB lokal.
    let client_uri = "mongodb://localhost:27017";
    // Parsing URI menjadi opsi-opsi koneksi client. `?` menyebarkan error.
    let mut client_options = ClientOptions::parse(client_uri).await?;
    // Memberi nama aplikasi (baik untuk logging di sisi DB).
    client_options.app_name = Some("SensorApp".to_string());

    // Membuat client MongoDB baru dengan opsi yang telah dikonfigurasi.
    let client = Client::with_options(client_options)?;
    // Mendapatkan handle ke database `sensor_db`.
    let db = client.database(DB_NAME);
    // Mendapatkan handle ke collection `readings` yang tipenya adalah `SensorReading`.
    let collection: Collection<SensorReading> = db.collection(COLLECTION_NAME);

    // Menjalankan command `ping` ke server DB untuk memverifikasi koneksi.
    client
        .database("admin")
        .run_command(doc! { "ping": 1 }, None)
        .await?;
    
    // Jika semua berhasil, kembalikan handle ke collection.
    Ok(collection)
}

// Fungsi asinkron untuk menyisipkan satu dokumen `SensorReading` ke dalam collection.
async fn insert_reading(
    collection: &Collection<SensorReading>,
    reading: SensorReading,
) -> Result<(), mongodb::error::Error> {
    // Menyisipkan satu dokumen. Library Rust untuk MongoDB akan mengonversi
    // struct `reading` menjadi BSON secara otomatis karena ada trait `Serialize`.
    collection.insert_one(reading, None).await?;
    Ok(())
}