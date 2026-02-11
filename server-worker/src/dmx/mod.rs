use std::net::UdpSocket;
use std::collections::HashMap;

pub struct SacnReceiver {
    socket: UdpSocket,
    universe_cache: HashMap<u16, Vec<u8>>,
}

impl SacnReceiver {
    pub fn new(port: u16) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(false)?;
        
        Ok(Self {
            socket,
            universe_cache: HashMap::new(),
        })
    }

    pub fn listen(&mut self) {
        let mut buf = [0u8; 1144]; // sACN packets are typically around 638 bytes, but max Ethernet MTU is safer
        
        log::info!("sACN Receiver listening on port {}", self.socket.local_addr().unwrap().port());

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    self.process_packet(&buf[..size], addr);
                }
                Err(e) => {
                    log::error!("Error receiving UDP packet: {}", e);
                }
            }
        }
    }

    fn process_packet(&mut self, data: &[u8], addr: std::net::SocketAddr) {
        // Basic sACN validation (E1.31)
        if data.len() < 125 { return; } // Minimum size for a valid Data Packet
        
        // Preamble Size (0x0010) and Post-amble Size (0x0000)
        if data[0..2] != [0x00, 0x10] { return; }
        
        // ACN Packet Identifier "ASC-E1.17"
        if &data[4..16] != b"ASC-E1.17\0\0\0" { return; }

        // Universe is at bytes 113-114 (big endian)
        let universe = u16::from_be_bytes([data[113], data[114]]);
        
        // DMX data starts at byte 125 (Property Values)
        // Byte 125 is the Start Code (usually 0x00)
        // Channels 1-512 follow from byte 126
        let dmx_data = &data[126..126 + 512.min(data.len() - 126)];

        if let Some(old_data) = self.universe_cache.get(&universe) {
            let mut changes = Vec::new();
            for (i, (&new_val, &old_val)) in dmx_data.iter().zip(old_data.iter()).enumerate() {
                if new_val != old_val {
                    changes.push((i + 1, old_val, new_val)); // i+1 for 1-based DMX address
                }
            }

            if !changes.is_empty() {
                log::info!("Universe {} (Unicast) changed ({} updates) from {}", universe, changes.len(), addr);
                for (channel, old, new) in changes.iter().take(10) {
                    log::info!("  Channel {}: {} -> {}", channel, old, new);
                }
                if changes.len() > 10 {
                    log::info!("  ... and {} more changes", changes.len() - 10);
                }
            }
        } else {
            log::info!("First Unicast packet for Universe {} from {}", universe, addr);
        }

        // Update cache
        self.universe_cache.insert(universe, dmx_data.to_vec());
    }
}

pub fn run_dmx_loop() {
    let port: u16 = std::env::var("SACN_PORT")
        .unwrap_or_else(|_| "5568".to_string())
        .parse()
        .expect("Invalid SACN_PORT");

    let mut receiver = SacnReceiver::new(port).expect("Failed to bind sACN socket");
    
    receiver.listen();
}
