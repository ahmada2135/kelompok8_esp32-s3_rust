use anyhow::Result;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};

use esp_idf_hal::delay::{Ets, FreeRtos};
use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};
use esp_idf_hal::peripherals::Peripherals;

use dht_sensor::{dht22, DhtReading};
use log::{info, warn};

use esp_idf_sys as _;

const READ_INTERVAL_MS: u32 = 1000;
const BUFFER_SIZE: usize = 5;

// Parameter Batas Suhu & Kelembapan
const TEMP_WASPADA: f32 = 25.0;
const TEMP_BAHAYA: f32 = 35.0;

const HUM_WASPADA: f32 = 30.0;
const HUM_BAHAYA: f32 = 45.0;

// Parameter Batas untuk Kualitas Udara (MQ-2)
const AQ_WASPADA: f32 = 300.0;
const AQ_BAHAYA: f32 = 600.0;

// Parameter Fisik Sensor MQ-2 (Kalibrasi)
const V_CC: f32 = 3.3;  // Tegangan operasional ESP32
const R_L: f32 = 10.0;  // Resistor beban (Load Resistor) pada modul MQ-2 (biasanya 10 kOhm)
const R_0: f32 = 9.83;  // Resistansi sensor pada udara bersih (Nilai kalibrasi standar)

#[derive(Clone, Copy)]
struct SensorData {
    temp: f32,
    hum: f32,
    aq: f32, 
}

impl SensorData {
    fn empty() -> Self {
        Self {
            temp: 0.0,
            hum: 0.0,
            aq: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
enum SystemStatus {
    Normal,
    Waspada,
    Bahaya,
    Fault,
}

fn status_text(status: SystemStatus) -> &'static str {
    match status {
        SystemStatus::Normal => "NORMAL",
        SystemStatus::Waspada => "WASPADA",
        SystemStatus::Bahaya => "BAHAYA",
        SystemStatus::Fault => "FAULT",
    }
}

// Konversi nilai ADC mentah menjadi konsentrasi gas (PPM) menggunakan rumus fisik Rs
fn map_adc_to_ppm(raw: u16) -> f32 {
    // 1. Mencegah pembagian dengan nol jika sensor terputus
    if raw == 0 {
        return 0.0; 
    }

    // 2. Konversi ADC mentah (0 - 4095) ke Tegangan (V_out)
    let v_out = (raw as f32 / 4095.0) * V_CC;

    // 3. Rumus pembagi tegangan untuk mencari Rs
    // Vcc / Vout = (Rs / RL) + 1  =>  Rs = RL * ((Vcc / Vout) - 1)
    let r_s = R_L * ((V_CC / v_out) - 1.0);

    // 4. Menghitung Rasio Rs/R0
    let rasio = r_s / R_0;

    // 5. Konversi Rasio ke PPM menggunakan rumus regresi power-law (Karakteristik MQ-2)
    // Rumus: PPM = a * (Rs/R0)^b (Nilai a dan b diambil dari datasheet MQ-2 untuk gas Asap/LPG)
    let a = 116.6020682;
    let b = -2.769034857;
    let ppm = a * rasio.powf(b);

    // Batasi output agar tidak menghasilkan nilai tak hingga (maks 10.000 PPM)
    ppm.clamp(0.0, 10000.0)
}

fn push_to_buffer(
    buffer: &mut [SensorData; BUFFER_SIZE],
    index: &mut usize,
    count: &mut usize,
    data: SensorData,
) {
    buffer[*index] = data;
    *index = (*index + 1) % BUFFER_SIZE;

    if *count < BUFFER_SIZE {
        *count += 1;
    }
}

fn moving_average(buffer: &[SensorData; BUFFER_SIZE], count: usize) -> SensorData {
    if count == 0 {
        return SensorData::empty();
    }

    let mut sum_temp = 0.0;
    let mut sum_hum = 0.0;
    let mut sum_aq = 0.0;

    for data in buffer.iter().take(count) {
        sum_temp += data.temp;
        sum_hum += data.hum;
        sum_aq += data.aq;
    }

    let n = count as f32;
    SensorData {
        temp: sum_temp / n,
        hum: sum_hum / n,
        aq: sum_aq / n,
    }
}

fn data_valid(data: SensorData) -> bool {
    let temp_valid = (-20.0..=80.0).contains(&data.temp);
    let hum_valid = (0.0..=100.0).contains(&data.hum);
    let aq_valid = (0.0..=1000.0).contains(&data.aq); 

    temp_valid && hum_valid && aq_valid
}

fn evaluate_status(data: SensorData) -> SystemStatus {
    if data.temp >= TEMP_BAHAYA || data.hum >= HUM_BAHAYA || data.aq >= AQ_BAHAYA {
        SystemStatus::Bahaya
    } 
    else if data.temp >= TEMP_WASPADA || data.hum >= HUM_WASPADA || data.aq >= AQ_WASPADA {
        SystemStatus::Waspada
    } 
    else {
        SystemStatus::Normal
    }
}

fn set_lampu<R, Y, G>(
    merah: &mut PinDriver<'_, R, Output>,
    kuning: &mut PinDriver<'_, Y, Output>,
    hijau: &mut PinDriver<'_, G, Output>,
    r: bool,
    y: bool,
    g: bool,
) -> Result<()>
where
    R: OutputPin,
    Y: OutputPin,
    G: OutputPin,
{
    if r { merah.set_high()?; } else { merah.set_low()?; }
    if y { kuning.set_high()?; } else { kuning.set_low()?; }
    if g { hijau.set_high()?; } else { hijau.set_low()?; }
    Ok(())
}

fn apply_status_to_led<R, Y, G>(
    status: SystemStatus,
    merah: &mut PinDriver<'_, R, Output>,
    kuning: &mut PinDriver<'_, Y, Output>,
    hijau: &mut PinDriver<'_, G, Output>,
) -> Result<()>
where
    R: OutputPin,
    Y: OutputPin,
    G: OutputPin,
{
    match status {
        SystemStatus::Normal => set_lampu(merah, kuning, hijau, false, false, true)?,
        SystemStatus::Waspada => set_lampu(merah, kuning, hijau, false, true, false)?,
        SystemStatus::Bahaya | SystemStatus::Fault => {
            set_lampu(merah, kuning, hijau, true, false, false)?
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("SISTEM MONITORING KUALITAS UDARA BERBASIS IOT");

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let mut led_merah = PinDriver::output(pins.gpio37)?;
    let mut led_kuning = PinDriver::output(pins.gpio36)?;
    let mut led_hijau = PinDriver::output(pins.gpio35)?;
    set_lampu(&mut led_merah, &mut led_kuning, &mut led_hijau, false, false, false)?;

    let adc1 = AdcDriver::new(peripherals.adc1)?;
    let adc_config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut aq_pin = AdcChannelDriver::new(&adc1, pins.gpio4, &adc_config)?;

    let mut dht_pin = PinDriver::input_output_od(pins.gpio5)?;
    dht_pin.set_high()?;

    let mut static_buffer: [SensorData; BUFFER_SIZE] = [SensorData::empty(); BUFFER_SIZE];
    let mut buffer_index: usize = 0;
    let mut buffer_count: usize = 0;
    let mut cycle_counter: u32 = 0;

    println!("Menunggu sensor DHT22 stabil (2 detik)...");
    FreeRtos::delay_ms(2000);

    loop {
        cycle_counter += 1;

        let mut delay = Ets;
        let dht_result = dht22::Reading::read(&mut delay, &mut dht_pin);
        let raw_aq = adc1.read(&mut aq_pin).unwrap_or(0);

        let (temp_val, hum_val, is_dht_valid) = match dht_result {
            Ok(reading) => (reading.temperature, reading.relative_humidity, true),
            Err(_) => (0.0, 0.0, false),
        };

        let data_baru = SensorData {
            temp: temp_val,
            hum: hum_val,
            aq: map_adc_to_ppm(raw_aq), 
        };

        if !is_dht_valid || !data_valid(data_baru) || data_baru.temp.is_nan() || data_baru.hum.is_nan() {
            let status = SystemStatus::Fault;
            apply_status_to_led(status, &mut led_merah, &mut led_kuning, &mut led_hijau)?;
            
            warn!("Siklus {} | STATUS: FAULT (Sensor error/tidak valid)", cycle_counter);
            FreeRtos::delay_ms(READ_INTERVAL_MS);
            continue;
        }

        push_to_buffer(&mut static_buffer, &mut buffer_index, &mut buffer_count, data_baru);
        let filtered_data = moving_average(&static_buffer, buffer_count);
        
        let status = evaluate_status(filtered_data);
        apply_status_to_led(status, &mut led_merah, &mut led_kuning, &mut led_hijau)?;

        info!(
            "Siklus: {}, Temp: {:.1} C, Hum: {:.1} %, Gas: {:.0}, Kondisi: {}, Buffer: {}/{}",
            cycle_counter,
            filtered_data.temp, 
            filtered_data.hum, 
            filtered_data.aq,
            status_text(status),
            buffer_count, 
            BUFFER_SIZE
        );

        println!("{} {:.1} {:.1} {:.0}", cycle_counter, filtered_data.temp, filtered_data.hum, filtered_data.aq);
        
        FreeRtos::delay_ms(READ_INTERVAL_MS);
    }
}