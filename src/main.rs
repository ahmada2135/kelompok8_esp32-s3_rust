use anyhow::Result;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};
use esp_idf_hal::peripherals::Peripherals;

use log::{info, warn};

use esp_idf_sys as _;

const READ_INTERVAL_MS: u32 = 1000;
const BUFFER_SIZE: usize = 5;

const FORCE_FAULT: bool = false;

const TEMP_WASPADA: f32 = 25.0;
const TEMP_BAHAYA: f32 = 35.0;

const HUM_WASPADA: f32 = 30.0;
const HUM_BAHAYA: f32 = 45.0;

const AQ_WASPADA: f32 = 30.0;
const AQ_BAHAYA: f32 = 45.0;

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

fn status_code(status: SystemStatus) -> u8 {
    match status {
        SystemStatus::Normal => 0,
        SystemStatus::Waspada => 1,
        SystemStatus::Bahaya => 2,
        SystemStatus::Fault => 3,
    }
}

fn map_adc_to_temp(raw: u16) -> f32 {
    (raw as f32 / 4095.0) * 50.0
}

fn map_adc_to_percent(raw: u16) -> f32 {
    (raw as f32 / 4095.0) * 100.0
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
    let temp_valid = (0.0..=50.0).contains(&data.temp);
    let hum_valid = (0.0..=100.0).contains(&data.hum);
    let aq_valid = (0.0..=100.0).contains(&data.aq);

    temp_valid && hum_valid && aq_valid
}

fn evaluate_status(data: SensorData) -> SystemStatus {
    if data.temp >= TEMP_BAHAYA || data.hum >= HUM_BAHAYA || data.aq >= AQ_BAHAYA {
        SystemStatus::Bahaya
    } else if data.temp >= TEMP_WASPADA || data.hum >= HUM_WASPADA || data.aq >= AQ_WASPADA {
        SystemStatus::Waspada
    } else {
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
    if r {
        merah.set_high()?;
    } else {
        merah.set_low()?;
    }

    if y {
        kuning.set_high()?;
    } else {
        kuning.set_low()?;
    }

    if g {
        hijau.set_high()?;
    } else {
        hijau.set_low()?;
    }

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
        SystemStatus::Normal => {
            set_lampu(merah, kuning, hijau, false, false, true)?;
        }
        SystemStatus::Waspada => {
            set_lampu(merah, kuning, hijau, false, true, false)?;
        }
        SystemStatus::Bahaya => {
            set_lampu(merah, kuning, hijau, true, false, false)?;
        }
        SystemStatus::Fault => {
            set_lampu(merah, kuning, hijau, true, false, false)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("SISTEM MONITORING SENSOR ESP32-S3 BERBASIS RUST");
    info!("Metode: Zero-Copy Buffer, Moving Average, Fault Isolation");
    info!("Input : 3 Potensiometer sebagai TEMP, HUM, AQ");
    info!("Output: 3 LED indikator Normal/Waspada/Bahaya");
    info!("CSV,cycle,temp_map,hum_map,aq_map,temp_filter,hum_filter,aq_filter,status");
    info!("Status: 0=Normal, 1=Waspada, 2=Bahaya, 3=Fault");

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let mut led_merah = PinDriver::output(pins.gpio37)?;
    let mut led_kuning = PinDriver::output(pins.gpio36)?;
    let mut led_hijau = PinDriver::output(pins.gpio35)?;

    set_lampu(
        &mut led_merah,
        &mut led_kuning,
        &mut led_hijau,
        false,
        false,
        false,
    )?;

    let adc = AdcDriver::new(peripherals.adc2)?;

    let adc_config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };

    let mut temp_pin = AdcChannelDriver::new(&adc, pins.gpio12, &adc_config)?;
    let mut hum_pin = AdcChannelDriver::new(&adc, pins.gpio11, &adc_config)?;
    let mut aq_pin = AdcChannelDriver::new(&adc, pins.gpio13, &adc_config)?;

    let mut static_buffer: [SensorData; BUFFER_SIZE] = [SensorData::empty(); BUFFER_SIZE];
    let mut buffer_index: usize = 0;
    let mut buffer_count: usize = 0;

    let mut cycle_counter: u32 = 0;

    loop {
        cycle_counter += 1;

        let raw_temp = adc.read(&mut temp_pin)?;
        let raw_hum = adc.read(&mut hum_pin)?;
        let raw_aq = adc.read(&mut aq_pin)?;

        let data_baru = SensorData {
            temp: map_adc_to_temp(raw_temp),
            hum: map_adc_to_percent(raw_hum),
            aq: map_adc_to_percent(raw_aq),
        };

        if FORCE_FAULT || !data_valid(data_baru) {
            let status = SystemStatus::Fault;

            apply_status_to_led(
                status,
                &mut led_merah,
                &mut led_kuning,
                &mut led_hijau,
            )?;

            println!(
                "CSV,{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{}",
                cycle_counter,
                data_baru.temp,
                data_baru.hum,
                data_baru.aq,
                data_baru.temp,
                data_baru.hum,
                data_baru.aq,
                status_code(status)
            );

            warn!("FAULT pada siklus ke-{}", cycle_counter);
            warn!("Sensor tidak valid atau FORCE_FAULT aktif.");
            warn!("Sistem mencoba membaca ulang sensor.");

            FreeRtos::delay_ms(READ_INTERVAL_MS);
            continue;
        }

        push_to_buffer(
            &mut static_buffer,
            &mut buffer_index,
            &mut buffer_count,
            data_baru,
        );

        let filtered_data = moving_average(&static_buffer, buffer_count);

        let status = evaluate_status(filtered_data);

        apply_status_to_led(
            status,
            &mut led_merah,
            &mut led_kuning,
            &mut led_hijau,
        )?;

        println!(
            "CSV,{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{}",
            cycle_counter,
            data_baru.temp,
            data_baru.hum,
            data_baru.aq,
            filtered_data.temp,
            filtered_data.hum,
            filtered_data.aq,
            status_code(status)
        );

        info!(
            "Siklus {} | TEMP {:.1} C | HUM {:.1} % | AQ {:.1} % | STATUS {} | BUFFER {}/{}",
            cycle_counter,
            filtered_data.temp,
            filtered_data.hum,
            filtered_data.aq,
            status_text(status),
            buffer_count,
            BUFFER_SIZE
        );

        FreeRtos::delay_ms(READ_INTERVAL_MS);
    }
}