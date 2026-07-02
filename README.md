# Akuisisi Data Sensor DHT22 dan MQ-2 Berbasis Embedded Rust pada ESP32-S3

## Informasi Proyek

*   **Mata Kuliah:** Pemrograman Kontroller  
*   **Tugas:** Evaluasi Tengah Semester (ETS)  
*   **Dosen Pengampu:** Ahmad Radhy, S.Si., M.Si.  
*   **Kelas:** 4C  
*   **Kelompok 8:** 
    1. Khusnul Khuluq Ahmada (NRP: 2042241001)  
    2. Wildan Maulana Zulkha (NRP: 2042241059)  
*   **Institusi:** Institut Teknologi Sepuluh Nopember (ITS), 2026  

---

## Deskripsi Singkat

Sistem ini mengimplementasikan akuisisi data iklim mikro dan kualitas udara menggunakan **Embedded Rust** pada mikrokontroler **ESP32-S3**. Program membaca data dari sensor digital **DHT22** (Suhu dan Kelembapan) serta sensor analog **MQ-2** (Konsentrasi Gas Asap/LPG). 

Data yang diambil disaring secara *real-time* menggunakan *Static Circular Buffer* berkapasitas 5 sampel dan dihaluskan lewat filter *Simple Moving Average* (SMA) sebelum dievaluasi oleh sistem keputusan (*Decision Logic*) untuk mengontrol aktuasi 3 buah LED indikator status (Normal, Waspada, Bahaya, Fault) serta luaran data CSV melalui komunikasi serial.

---

## Persamaan Pemrosesan Data

### 1. Pembacaan Sensor Digital DHT22
Sensor DHT22 mentransmisikan data digital 40-bit biner melalui protokol *Single-Wire*. Nilai fisis suhu ($TEMP$) dan kelembapan ($HUM$) diekstrak dari gabungan data *high byte* dan *low byte* 16-bit yang kemudian dibagi dengan faktor skala 10:

* **Rumus Suhu:**
  $$TEMP = \frac{\text{Data 16-bit Suhu}}{10}$$ *(dalam satuan °C)*

* **Rumus Kelembapan:**
  $$HUM = \frac{\text{Data 16-bit Kelembapan}}{10}$$ *(dalam satuan %)*

### 2. Kalibrasi Sensor Analog MQ-2 (Pembagi Tegangan & Power-Law)
Modul MQ-2 bekerja memanfaatkan prinsip rangkaian pembagi tegangan (*voltage divider*) dengan resistor beban internal ($R_L = 10\text{ k}\Omega$). Tegangan keluaran ($V_{out}$) dibaca oleh ADC 12-bit ($0 - 4095$) dengan rumus:

$$V_{out} = \frac{ADC_{raw}}{4095} \times V_{cc}$$

Untuk mencari nilai resistansi fisik internal sensor ($R_s$) berdasarkan nilai $V_{out}$ tersebut, diterapkan hukum pembagi tegangan balik:

$$R_s = R_L \cdot \left( \frac{V_{cc}}{V_{out}} - 1 \right)$$

Konsentrasi gas aktual dalam satuan *Parts Per Million* (PPM) dihitung dengan membandingkan nilai $R_s$ terhadap resistansi udara bersih ($R_0 = 9.83\text{ k}\Omega$) menggunakan persamaan regresi kurva logaritmik (*Power-Law*):

$$PPM = a \cdot \left( \frac{R_s}{R_0} \right)^b$$

*(Konstanta karakteristik kurva: $a = 116.6020682$ dan $b = -2.769034857$)*

### 3. Filter Sinyal Digital (Simple Moving Average)
Untuk mereduksi fluktuasi derau (*noise*), data dari *circular buffer* dikondensasi menggunakan rata-rata kumulatif linier dengan ukuran jendela tetap ($n = 5$):

$$\overline{TEMP} = \frac{1}{n}\sum_{i=1}^{n} TEMP_i$$

$$\overline{HUM} = \frac{1}{n}\sum_{i=1}^{n} HUM_i$$

$$\overline{GAS} = \frac{1}{n}\sum_{i=1}^{n} GAS_i$$

---

## Logika Ambang Batas (Threshold)

<<<<<<< HEAD
| Parameter | Kondisi NORMAL | Batas WASPADA | Batas BAHAYA | Rentang Valid Fisis |
|---|---|---|---|---|
| **Suhu (TEMP)** | $< 25.0^\circ\text{C}$ | $\ge 25.0^\circ\text{C}$ | $\ge 35.0^\circ\text{C}$ | $-20.0^\circ\text{C} \ \dots \ 80.0^\circ\text{C}$ |
| **Kelembapan (HUM)** | $< 30.0\%$ | $\ge 30.0\%$ | $\ge 45.0\%$ | $0.0\% \ \dots \ 100.0\%$ |
| **Kualitas Gas (GAS)** | $< 300.0\text{ PPM}$ | $\ge 300.0\text{ PPM}$ | $\ge 600.0\text{ PPM}$ | $0.0 \ \dots \ 1000.0\text{ PPM}$ |
=======
| Parameter | Batas WASPADA | Batas BAHAYA |
|---|---|---|
| **Suhu (TEMP)** | $\ge 25.0^\circ\text{C}$ | $\ge 35.0^\circ\text{C}$ |
| **Kelembapan (HUM)** | $\ge 30.0\%$ | $\ge 45.0\%$ |
| **Kualitas Gas (GAS)** | $\ge 300.0\text{ PPM}$ | $\ge 600.0\text{ PPM}$ |
>>>>>>> da51a4ade4b1be7b7e38154666262732002b8e2c

*   **Status Fault:** Jika pembacaan `NaN`, sensor terputus, atau data melompat keluar dari rentang valid fisis, sistem langsung mengaktifkan status **Fault** (Isolasi data rusak, LED Merah menyala, data tidak masuk buffer).

---

## Langkah Menjalankan Proyek

1. **Kloning Repositori:**
   ```bash
   git clone [https://github.com/ahmada2135/kelompok8_esp32-s3_rust.git](https://github.com/ahmada2135/kelompok8_esp32-s3_rust.git)
   cd kelompok8_esp32-s3_rust
