# ESP32 Showtime Client

This folder contains a reference implementation for an ESP32 client using Arduino and Nanopb.

## Prerequisites

1.  **Libraries:**
    *   `PubSubClient` (for MQTT)
    *   `Nanopb` (for Protobuf)
2.  **Generate Protobuf Headers:**
    Use `nanopb_generator.py` on the `messages.proto` file. **Important:** keep the `messages.options` file in the same directory to ensure fixed-size buffers:
    ```bash
    python nanopb_generator.py messages.proto
    ```
    This will generate `messages.pb.c` and `messages.pb.h`. Copy these into your Arduino project.

## Workflow

1.  **Boot & Connect:** Connects to WiFi and MQTT.
2.  **Initial Handshake:** Sends an `UpdateConfig` with `is_ack = true` and only its MAC address.
3.  **Config Sync:** The Server-Worker responds with the full configuration on `showtime/config`.
4.  **Live Control:** Receives `SetDmx` packets on `showtime/set_dmx/<MAC>` for real-time LED control.
