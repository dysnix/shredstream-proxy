use std::net::UdpSocket;
use solana_ledger::shred::merkle::Shred;

fn main() -> std::io::Result<()> {
    // Bind to UDP port (using IPv6 to match forwarder's send socket)
    // IPv6 sockets can receive from both IPv6 and IPv4-mapped addresses
    let socket = UdpSocket::bind("[::]:21000")?;
    println!("Socket bound to: {}", socket.local_addr()?);

    // Buffer to receive packets (Solana shreds are typically 1203-1228 bytes)
    let mut buf = [0u8; 1500];
    let mut packet_count = 0u64;

    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                packet_count += 1;
                let packet_data = buf[..size].to_vec();
                
                // Try to parse as a shred using from_payload
                match Shred::from_payload(packet_data) {
                    Ok(shred) => {
                        let slot = shred.common_header().slot;
                        let fec_set_index = shred.fec_set_index();
                        let shred_index = shred.index();
                        let shred_type = match &shred {
                            Shred::ShredData(_) => "data",
                            Shred::ShredCode(_) => "code",
                        };
                        
                        println!(
                            "[Packet #{}] Received shred from {}: slot={}, fec_set_index={}, shred_index={}, size={} bytes, type={}",
                            packet_count,
                            src,
                            slot,
                            fec_set_index,
                            shred_index,
                            size,
                            shred_type
                        );
                    }
                    Err(e) => {
                        // Not a valid shred, but still log the packet
                        println!(
                            "[Packet #{}] Received raw packet from {}: size={} bytes (not a valid shred: {})",
                            packet_count,
                            src,
                            size,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                return Err(e);
            }
        }
    }
}