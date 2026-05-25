# Metode Akuisisi Data Sensor Asinkron Berbasis Embedded Rust pada ESP32-S3 dengan Zero-Copy Buffer, Fault Isolation, dan Watchdog Task

## Project Information

**Course:** Pemrograman Kontroller  
**Assignment:** Evaluasi Tengah Semester  
**Lecturer:** Ahmad Radhy, S.Si., M.Si  
**Class:** 4C  

**Group Members:**

| Name | Student ID |
|---|---|
| Khusnul Khuluq Ahmada | 2042241001 |
| Wildan Maulana Zulkha | 2042241059 |

**Study Program:** D4 Teknologi Rekayasa Instrumentasi  
**Department:** Teknik Instrumentasi  
**Faculty:** Fakultas Vokasi  
**Institution:** Institut Teknologi Sepuluh Nopember  
**Year:** 2026  

---

## Project Description

This project implements a sensor monitoring simulation using ESP32-S3 and Embedded Rust. The proposed method is designed to improve sensor data acquisition by combining asynchronous task logic, zero-copy static buffer, fault isolation, digital filtering, batch communication, and watchdog task monitoring.

The system monitors three parameters: temperature, humidity, and air quality. In the simulation, the DHT22 sensor is represented by two potentiometers. One potentiometer is used to simulate temperature, and another potentiometer is used to simulate humidity. A third potentiometer is used to simulate an analog air quality sensor.

The output system uses three LEDs as indicators. The green LED represents normal condition, the yellow LED represents warning condition, and the red LED represents danger or fault condition. The sensor values, filtered data, buffer condition, and system status are displayed through the serial monitor.

This project was developed as a simulation-based implementation of a new method proposed from the integration of future work from 15 journal references related to embedded systems, microcontroller architecture, Embedded Rust, sensor acquisition, memory safety, and fault-tolerant systems.

---

## Proposed Method Background

The proposed method is titled:

**Metode Akuisisi Data Sensor Asinkron Berbasis Embedded Rust pada ESP32-S3 dengan Zero-Copy Buffer, Fault Isolation, dan Watchdog Task**

The method was developed by combining future work from 15 journal references. The main idea is to build a safer, more reliable, and more efficient embedded sensor monitoring system using Rust on ESP32-S3.

The method integrates the following concepts:

- Asynchronous sensor acquisition logic
- Zero-copy static buffer
- Moving average digital filter
- Fault detection and isolation
- Decision logic for system status
- Batch serial communication
- Watchdog task monitoring concept

In this simulation, the zero-copy buffer is represented by a static array buffer. Sensor data is stored in this buffer before being processed by the moving average filter. Fault isolation is represented by checking whether the sensor data is valid or not. The watchdog task concept is represented as a logical monitoring mechanism in the program flow.

---

## System Components

The simulation uses the following components:

- ESP32-S3
- 3 potentiometers
  - Temperature input
  - Humidity input
  - Air quality input
- 3 LEDs
  - Green LED for normal condition
  - Yellow LED for warning condition
  - Red LED for danger or fault condition
- Resistors
- Serial monitor
- Wokwi Simulator
- Visual Studio Code
- Embedded Rust environment
- GNUPlot for data visualization

---

## Pin Configuration

| Component | ESP32-S3 Pin |
|---|---|
| Temperature Potentiometer | GPIO12 |
| Humidity Potentiometer | GPIO11 |
| Air Quality Potentiometer | GPIO13 |
| Red LED | GPIO37 |
| Yellow LED | GPIO36 |
| Green LED | GPIO35 |

---

## Block Diagram

The block diagram shows the overall architecture of the proposed method. Sensor data from the input components is processed by the ESP32-S3 through several logical stages, including asynchronous sensor task, zero-copy buffer, digital filter, fault detection and isolation, decision logic, and watchdog task.

![Block Diagram](docs/block_diagram.png)

---

## Flowchart

The flowchart describes the algorithm of the proposed method. The system starts by initializing the ESP32-S3, sensor inputs, LED outputs, serial monitor, buffer, and Rust task logic. After initialization, the system reads sensor data, validates the data, stores it into the buffer, applies moving average filtering, evaluates the system status, updates the LED indicators, sends data to the serial monitor, and repeats the process.

![Flowchart](docs/flowchart.png)

---

## Simulation Circuit

The simulation circuit was created using Wokwi Simulator. Three potentiometers are used as sensor inputs, while three LEDs are used as system indicators.

![Simulation Circuit](docs/wokwi_circuit.png)

---

## How to Run the Simulation

### 1. Clone the Repository

```bash
git clone https://github.com/ahmada2135/kelompok8_esp32-s3_rust.git
cd kelompok8_esp32-s3_rust