#include <cstdint>
#include <pb_encode.h>
#include <pb_decode.h>
#include "messages.pb.h"
#include <WiFi.h>
#include <PubSubClient.h>

typedef struct {
    uint32_t r;
    uint32_t g;
    uint32_t b;
} rgb_values_t;

// --- Configuration ---
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";
const char* mqtt_server = "YOUR_MQTT_BROKER_IP";
const int mqtt_buffer_size = 16384;

// --- Learned configuration --
char deviceName[64] = "Unknown";
pb_size_t dmx_config_count = 0;
esp_status_DmxConfig dmx_config[8];


// -- System global vars --
WiFiClient espClient;
PubSubClient client(espClient);
String macAddr;

// -- Business logic vars --
uint8_t buffer[mqtt_buffer_size];
uint32_t lastStatusSend = 0;
uint32_t lastSacnRcv = 0;
esp_status_SetDmx dmx_data = esp_status_SetDmx_init_default;

void setup() {
    Serial.begin(115200);
    setup_wifi();
    client.setBufferSize(mqtt_buffer_size);
    client.setServer(mqtt_server, 1883);
    client.setCallback(callback);
}

void loop() {
    if (!client.connected()) {
        reconnect_mqtt();
    }
    client.loop();

    uint32_t now = millis();
    if (now - lastStatusSend > 50) {
        lastStatusSend = now;
        send_status();
    }
}

void resetAllPins() {
  for (int i = 0; i < dmx_config_count; i++) {
    pinMode(dmx_config[i].esp_pin, OUTPUT);
    digitalWrite(dmx_config[i].esp_pin, LOW);
  }
}

void updateFromDmxValues() {
  for (int i = 0; i < dmx_data.outputs_count; i++) {
    uint32_t start_addr = dmx_data.outputs[i].dmx_start_addr;
    
    // Find matching config
    for (int j = 0; j < dmx_config_count; j++) {
      if (dmx_config[j].start_addr == start_addr) {
        uint32_t pin = dmx_config[j].esp_pin;
        uint8_t* raw_data = dmx_data.outputs[i].dmx_values.bytes;
        size_t data_len = dmx_data.outputs[i].dmx_values.size;

        // Implementation for LED control would go here
        // For now, we just log it
        // Serial.printf("Pin %d: Received %d bytes\n", pin, data_len);
      }
    }
  }
}

rgb_values_t getRgbValues() {
  rgb_values_t rgb_values = {0, 0, 0};
  
  if (dmx_data.outputs_count > 0 && dmx_data.outputs[0].dmx_values.size >= 3) {
    rgb_values.r = dmx_data.outputs[0].dmx_values.bytes[0];
    rgb_values.g = dmx_data.outputs[0].dmx_values.bytes[1];
    rgb_values.b = dmx_data.outputs[0].dmx_values.bytes[2];
  }
  
  return rgb_values;
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
    if (String(config.mac_addr) == macAddr && config.is_ack == false) {
      Serial.printf("Received new Config: Name=%s, Outputs=%d\n", config.device_name, config.dmx_config_count);

      strncpy(deviceName, config.device_name, sizeof(deviceName));
      dmx_config_count = config.dmx_config_count;
      memcpy(dmx_config, config.dmx_config, sizeof(dmx_config));

      resetAllPins();

      // Send back ACK
      config.is_ack = true;
      pb_ostream_t ostream = pb_ostream_from_buffer(buffer, sizeof(buffer));
      if (pb_encode(&ostream, esp_status_UpdateConfig_fields, &config)) {
        client.publish("showtime/config", buffer, ostream.bytes_written);
        Serial.println("Sent config ACK to worker");
      } else {
        Serial.println("Failed to encode config ACK");
      }
    }
  } else {
    Serial.println("Failed to decode UpdateConfig");
  }
}

void handleDmxUpdate(byte* payload, unsigned int length) {
  pb_istream_t stream = pb_istream_from_buffer(payload, length);

  if (pb_decode(&stream, esp_status_SetDmx_fields, &dmx_data)) {
    lastSacnRcv = millis();
    updateFromDmxValues();
  } else {
    Serial.println("Failed to decode SetDmx");
  }
}

void send_status() {
  if (!client.connected()) return;

  esp_status_EspStatusMessage msg = esp_status_EspStatusMessage_init_default;
  strncpy(msg.device_name, deviceName, sizeof(msg.device_name));
  strncpy(msg.mac_addr, macAddr.c_str(), sizeof(msg.mac_addr));
  msg.signal_strength = WiFi.RSSI();
  msg.dmx_config_count = dmx_config_count;
  memcpy(msg.dmx_config, dmx_config, sizeof(msg.dmx_config));
  rgb_values_t rgb_values = getRgbValues();
  msg.r = rgb_values.r;
  msg.g = rgb_values.g;
  msg.b = rgb_values.b;

  pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));
  if (pb_encode(&stream, esp_status_EspStatusMessage_fields, &msg)) {
    client.publish("showtime/status", buffer, stream.bytes_written);
  } else {
    Serial.println("Failed to encode status message");
  }
}

void reconnect_mqtt() {
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    if (client.connect(macAddr.c_str())) {
      Serial.println("connected to MQTT server");

      client.subscribe("showtime/config");
      String dmxTopic = "showtime/set_dmx/" + macAddr;
      client.subscribe(dmxTopic.c_str());
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

  strncpy(msg.mac_addr, macAddr.c_str(), sizeof(msg.mac_addr));
  msg.is_ack = true;

  pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));
  if (pb_encode(&stream, esp_status_UpdateConfig_fields, &msg)) {
    client.publish("showtime/config", buffer, stream.bytes_written);
    Serial.println("Sent handshake (config request) to worker");
  } else {
    Serial.println("Failed to encode handshake");
  }
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

  macAddr = WiFi.macAddress();
  Serial.println("\nWiFi connected");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
  Serial.print("MAC address: ");
  Serial.println(macAddr);
}
