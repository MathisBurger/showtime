import random
import threading
import time

import messages_pb2 as proto
import paho.mqtt.client as mqtt

BROKER = "localhost"
TOPIC = "showtime/status"

NUM_DEVICES = 25
DEVICE_NAMES = [f"Spot_{i + 1:02d}" for i in range(NUM_DEVICES)]
MAC_PREFIX = "001B63844"


def simulate_device(name, ip):
    client = mqtt.Client()
    
    device_data = {
        "name": name,
        "configs": [{"universe": 1, "start_addr": 1, "led_count": 100, "esp_pin": 2, "mode": 0}]
    }

    def on_message(client, userdata, msg):
        if msg.topic == "showtime/config":
            try:
                config_update = proto.UpdateConfig()
                config_update.ParseFromString(msg.payload)
                if config_update.mac_addr == ip:
                    print(f"[{name}] Received Config Update: name={config_update.device_name}, outputs={len(config_update.dmx_config)}")
                    device_data["name"] = config_update.device_name
                    device_data["configs"] = [
                        {
                            "universe": c.universe,
                            "start_addr": c.start_addr,
                            "led_count": c.led_count,
                            "esp_pin": c.esp_pin,
                            "mode": c.mode
                        } for c in config_update.dmx_config
                    ]
            except Exception as e:
                print(f"Error parsing config: {e}")

    client.on_message = on_message
    client.connect(BROKER, 1883, 60)
    client.subscribe("showtime/config")
    client.loop_start()
    
    print(f"Started simulation for {name} ({ip})")

    while True:
        msg = proto.EspStatusMessage()
        msg.r, msg.g, msg.b = (
            random.randint(0, 255),
            random.randint(0, 255),
            random.randint(0, 255),
        )
        msg.last_sacn_pkt = random.randint(1000, 9999)
        msg.device_name = device_data["name"]
        msg.mac_addr = ip
        
        # DMX-Konfigurationen aus dem lokalen State hinzufügen
        for cfg in device_data["configs"]:
            dmx = msg.dmx_config.add()
            dmx.universe = cfg["universe"]
            dmx.start_addr = cfg["start_addr"]
            dmx.led_count = cfg["led_count"]
            dmx.esp_pin = cfg["esp_pin"]
            dmx.mode = cfg["mode"]

        msg.signal_strength = random.randint(
            80, 100
        )  # Höhere Signalstärke für stabilere Demo

        binary_data = msg.SerializeToString()
        client.publish(TOPIC, binary_data)

        dice_roll = random.random()

        # NEUE LOGIK FÜR SCHNELLE UPDATES:
        if dice_roll < 0.90:
            # 90% der Zeit: Extrem schnell (Echtes Streaming-Feeling)
            wait_time = random.uniform(0.05, 0.1)  # 50ms bis 100ms
        elif dice_roll < 0.98:
            # 8% der Zeit: Kleiner Jitter
            wait_time = random.uniform(0.1, 0.3)  # 100ms bis 300ms
        else:
            # 2% der Zeit: Ein seltener "Schluckauf" (simuliert Netzwerk-Lag)
            wait_time = random.uniform(0.5, 1.5)
            print(f"DEBUG: {name} Jitter/Lag: {wait_time:.2f}s")

        time.sleep(wait_time)


# ... (Rest des Skripts bleibt gleich)


threads = []
try:
    for i in range(NUM_DEVICES):
        device_name = DEVICE_NAMES[i]
        device_ip = f"{MAC_PREFIX}{100 + i}"
        t = threading.Thread(
            target=simulate_device, args=(device_name, device_ip), daemon=True
        )
        t.start()
        threads.append(t)

    while True:
        time.sleep(1)

except KeyboardInterrupt:
    print("\nSimulation stopped.")
