import random
import threading
import time
import binascii

import messages_pb2 as proto
import paho.mqtt.client as mqtt

BROKER = "localhost"
TOPIC_STATUS = "showtime/status"
TOPIC_CONFIG = "showtime/config"
TOPIC_SET_DMX = "showtime/set_dmx/"

NUM_DEVICES = 5 # Etwas weniger für schönere Logs
MAC_PREFIX = "001B63844"

def simulate_device(idx, name, ip):
    client = mqtt.Client()
    
    # Sinnvolle Standardwerte: Jedes Gerät hat einen eigenen Start-Bereich
    # Wir nehmen 20 Kanäle Abstand pro Gerät
    start_addr = idx * 20 + 1
    
    device_data = {
        "name": name,
        "configs": [{"universe": 1, "start_addr": start_addr, "led_count": 5, "esp_pin": 2, "mode": 0}]
    }

    def on_message(client, userdata, msg):
        try:
            if msg.topic == TOPIC_CONFIG:
                config_update = proto.UpdateConfig()
                config_update.ParseFromString(msg.payload)
                if config_update.mac_addr == ip:
                    print(f"[{name}] ⚙️ Received Config Update: name={config_update.device_name}, outputs={len(config_update.dmx_config)}")
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
            
            elif msg.topic.startswith(TOPIC_SET_DMX):
                target_mac = msg.topic.split("/")[-1]
                if target_mac == ip:
                    dmx_cmd = proto.SetDmx()
                    dmx_cmd.ParseFromString(msg.payload)
                    for output in dmx_cmd.outputs:
                        hex_vals = binascii.hexlify(output.dmx_values).decode()
                        print(f"[{name}] 💡 DMX Update (Pin {output.esp_pin}): {hex_vals}")
                        
                        # Wir simulieren die Farbanpassung basierend auf dem ersten Pixel
                        if len(output.dmx_values) >= 3:
                            device_data["r"] = output.dmx_values[0]
                            device_data["g"] = output.dmx_values[1]
                            device_data["b"] = output.dmx_values[2]
                            
        except Exception as e:
            print(f"Error processing message on {name}: {e}")

    client.on_message = on_message
    client.connect(BROKER, 1883, 60)
    client.subscribe(TOPIC_CONFIG)
    client.subscribe(f"{TOPIC_SET_DMX}{ip}")
    client.loop_start()
    
    print(f"Started simulation for {name} ({ip}) at DMX Start {start_addr}")

    device_data["r"] = 0
    device_data["g"] = 0
    device_data["b"] = 0

    while True:
        msg = proto.EspStatusMessage()
        msg.r = device_data["r"]
        msg.g = device_data["g"]
        msg.b = device_data["b"]
        
        msg.last_sacn_pkt = random.randint(10, 50)
        msg.device_name = device_data["name"]
        msg.mac_addr = ip
        
        for cfg in device_data["configs"]:
            dmx = msg.dmx_config.add()
            dmx.universe = cfg["universe"]
            dmx.start_addr = cfg["start_addr"]
            dmx.led_count = cfg["led_count"]
            dmx.esp_pin = cfg["esp_pin"]
            dmx.mode = cfg["mode"]

        msg.signal_strength = random.randint(90, 100)
        binary_data = msg.SerializeToString()
        client.publish(TOPIC_STATUS, binary_data)
        
        # Status-Update alle 0.5 Sekunden für stabiles "Online" im Client
        time.sleep(0.5)

threads = []
try:
    for i in range(NUM_DEVICES):
        name = f"Spot_{i + 1:02d}"
        ip = f"{MAC_PREFIX}{100 + i}"
        t = threading.Thread(
            target=simulate_device, args=(i, name, ip), daemon=True
        )
        t.start()
        threads.append(t)

    while True:
        time.sleep(1)

except KeyboardInterrupt:
    print("\nSimulation stopped.")
