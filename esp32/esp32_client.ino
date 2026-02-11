#include <WiFi.h>
#include <PubSubClient.h>
#include <pb_encode.h>
#include <pb_decode.h>
#include "messages.pb.h" // Generated via nanopb

// --- Configuration ---
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";
const char* mqtt_server = "YOUR_MQTT_BROKER_IP";

WiFiClient espClient;
PubSubClient client(espClient);
String macAddr;

// --- Protobuf Buffers ---
uint8_t buffer[1024];

void setup() {
  Serial.begin(115200);
  setup_wifi();
  client.setServer(mqtt_server, 1883);
  client.setCallback(callback);
  
  macAddr = WiFi.macAddress();
  macAddr.replace(":", ""); // Remove colons to match our system format
}

void setup_wifi() {
  delay(10);
  Serial.println();
  Serial.print("Connecting to ");
  Serial.println(ssid);

  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println("\nWiFi connected");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
}

void callback(char* topic, byte* payload, unsigned int length) {
  String topicStr = String(topic);
  
  if (topicStr == "showtime/config") {
    handleConfigUpdate(payload, length);
  } else if (topicStr.startsWith("showtime/set_dmx/")) {
    handleDmxUpdate(payload, length);
  }
}

void handleConfigUpdate(byte* payload, unsigned int length) {
  esp_status_UpdateConfig config = esp_status_UpdateConfig_init_default;
  pb_istream_t stream = pb_istream_from_buffer(payload, length);
  
  if (pb_decode(&stream, esp_status_UpdateConfig_fields, &config)) {
    // Check if this config is meant for US - config updates are sent to everyone
    if (String(config.mac_addr) == macAddr) {
      Serial.printf("Received new Config: Name=%s, Outputs=%d\n", config.device_name, config.dmx_config_count);
      // Here you would store the config and re-initialize your LED pins
    }
  } else {
    Serial.println("Failed to decode UpdateConfig");
  }
}

void handleDmxUpdate(byte* payload, unsigned int length) {
  esp_status_SetDmx dmx_data = esp_status_SetDmx_init_default;
  pb_istream_t stream = pb_istream_from_buffer(payload, length);

  if (pb_decode(&stream, esp_status_SetDmx_fields, &dmx_data)) {
    // Process DMX outputs
    // The outputs are in dmx_data.outputs (repeated message)
    for (int i = 0; i < dmx_data.outputs_count; i++) {
        uint32_t pin = dmx_data.outputs[i].esp_pin;
        // Access raw bytes: dmx_data.outputs[i].dmx_values.bytes
        // Length: dmx_data.outputs[i].dmx_values.size
        // Serial.printf("DMX Update for Pin %d: %d bytes\n", pin, dmx_data.outputs[i].dmx_values.size);
    }
  } else {
    Serial.println("Failed to decode SetDmx");
  }
}

void reconnect() {
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    if (client.connect(macAddr.c_str())) {
      Serial.println("connected");
      
      // 1. Subscribe to relevant topics
      client.subscribe("showtime/config");
      String dmxTopic = "showtime/set_dmx/" + macAddr;
      client.subscribe(dmxTopic.c_str());
      
      // 2. Send Initial Handshake (Config Ack with empty fields but MAC)
      send_handshake();
    } else {
      Serial.print("failed, rc=");
      Serial.print(client.state());
      Serial.println(" try again in 5 seconds");
      delay(5000);
    }
  }
}

void send_handshake() {
  esp_status_UpdateConfig msg = esp_status_UpdateConfig_init_default;
  
  // Set only MAC and is_ack = true
  strncpy(msg.mac_addr, macAddr.c_str(), sizeof(msg.mac_addr));
  msg.is_ack = true;
  
  // Note: All other fields remain 0/empty by default initialization
  
  pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));
  if (pb_encode(&stream, esp_status_UpdateConfig_fields, &msg)) {
    client.publish("showtime/config", buffer, stream.bytes_written);
    Serial.println("Sent handshake (config request) to worker");
  } else {
    Serial.println("Failed to encode handshake");
  }
}

void loop() {
  if (!client.connected()) {
    reconnect();
  }
  client.loop();
  
  // Optional: Send periodic Status Updates to showtime/status
  // (Using EspStatusMessage similar to the python simulator)
}
