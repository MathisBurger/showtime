# 🎬 Showtime: LED Status Monitor

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![MQTT](https://img.shields.io/badge/protocol-MQTT-brokenwhite.svg)](https://mqtt.org/)
[![egui](https://img.shields.io/badge/ui-egui-blue.svg)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Showtime** is a high-performance real-time dashboard for monitoring ESP32-based LED controllers. Built with **Rust** and **egui**, it provides fluid visualization of status data, signal strength, and live color feedback via MQTT—optimized for large-scale installations.

---

## Features

* **Real-Time Monitoring:** Handles status updates from dozens of devices simultaneously with minimal latency.
* **Dynamic Grid Layout:** An intelligent, responsive interface that adapts to any window size and keeps device cards perfectly aligned.
* **Visual Color Feedback:** Each card displays the live RGB color currently output by the controller, including jitter analysis for sACN packets.
* **Smart Health Tracking:** Automatic state detection:
    * **Online**: Active data stream.
    * **Overdue**: Delayed packets (jitter/network lag).
    * **Offline**: Connection lost.
* **History Log:** Minor expandable history per device to analyze signal strength and color transitions over time.
* **Native Performance:** Powered by a Rust backend and hardware-accelerated UI for extreme resource efficiency. **NO ELECTRON BLOAT🚀🚀🚀**

---

## Installation
1.  **Clone the repository:**
    ```bash
    git clone [https://github.com/MathisBurger/showtime.git](https://github.com/MathisBurger/showtime.git)
    cd showtime
    ```

2. **Run the broker:**
  ```
  docker compose up -d
  ```

3.  **Run the application:**
    ```bash
    cargo run
    ```

4.  **Connect:**
    Enter your broker's IP address and port (default: 1883) in the setup screen and hit **Connect to show**.

---

## Architecture

Showtime utilizes **Protocol Buffers (protobuf)** for highly efficient binary data transmission. ESP32 clients publish their status messages to the `showtime/status` MQTT topic. The messages are then fetched by one or multiple clients running the showtime desktop app.

### Data Structure (Protobuf)
We use a compact binary format to minimize airtime for ESP32 devices:
```protobuf
message EspStatusMessage {
  uint32 r = 1;
  uint32 g = 2;
  uint32 b = 3;
  uint32 last_sacn_pkt = 4;
  string device_name = 5;
  uint32 signal_strength = 6;
  string ip_addr = 7;
}
```



### Simulation for Testing
To test the dashboard without physical hardware, you can use the included Python simulator. it generates 25 virtual ESP32 devices with randomized colors and simulated network jitter.

```bash
# In a new terminal
cd test_client
pip install paho-mqtt protobuf
python sim.py
```
