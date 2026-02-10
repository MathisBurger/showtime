import random
import time

import messages_pb2 as proto
import paho.mqtt.client as mqtt

BROKER = "localhost"
TOPIC = "showtime/status"

client = mqtt.Client()
client.connect(BROKER, 1883, 60)

print(f"Sending test data to {TOPIC}... STRG+C to quit.")

try:
    while True:
        msg = proto.EspStatusMessage()
        msg.r = random.randint(0, 255)
        msg.g = random.randint(0, 255)
        msg.b = random.randint(0, 255)
        msg.last_sacn_pkt = random.randint(1000, 9999)
        msg.device_name = "Stage_Left_01"
        msg.signal_strength = random.randint(60, 100)
        binary_data = msg.SerializeToString()

        client.publish(TOPIC, binary_data)

        print(f"Sent: RGB({msg.r}, {msg.g}, {msg.b}) von {msg.device_name}")
        time.sleep(1)
except KeyboardInterrupt:
    print("Shutdown.")
