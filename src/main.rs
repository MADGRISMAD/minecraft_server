use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::{BytesMut, BufMut};

// Enum para representar el estado del cliente
enum ClientState {
    Handshake,
    Status,
    Login,
    Play,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:25565").await?;
    println!("Minecraft server running on 127.0.0.1:25565");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection: {}", addr);

        tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(1024);
            let mut state = ClientState::Handshake;

            loop {
                if let Err(e) = socket.read_buf(&mut buffer).await {
                    eprintln!("Failed to read data: {}", e);
                    return;
                }
            
                if buffer.is_empty() {
                    eprintln!("Received an empty buffer. Skipping...");
                    continue; // Evita procesar buffers vacíos
                }
            
                match state {
                    ClientState::Handshake => {
                        if buffer.len() > 0 && buffer[0] == 0x00 {
                            println!("Handshake received.");
                            state = handle_handshake(&mut buffer);
                        }
                    }
                    ClientState::Status => {
                        if buffer.len() > 0 && buffer[0] == 0x00 {
                            println!("Status request received.");
                            handle_status(&mut socket).await;
                        } else if buffer.len() > 0 && buffer[0] == 0x01 {
                            println!("Ping request received.");
                            handle_ping(&mut socket, &buffer).await;
                        }
                    }
                    ClientState::Login => {
                        if buffer.len() > 0 && buffer[0] == 0x00 {
                            println!("Login Start received.");
                            handle_login_start(&mut socket).await;
                            state = ClientState::Play;
                        }
                    }
                    ClientState::Play => {
                        println!("Handling Play state (not implemented yet).");
                        break; // Detener el bucle por ahora
                    }
                }
            
                buffer.clear(); // Limpia el buffer para la próxima lectura
            }
            
        });
    }
}

// Maneja el Handshake y cambia al estado correspondiente
fn handle_handshake(buffer: &mut BytesMut) -> ClientState {
    let next_state = buffer[5]; // El cliente especifica el próximo estado en el Handshake
    match next_state {
        1 => ClientState::Status, // Ping
        2 => ClientState::Login,  // Login
        _ => ClientState::Handshake,
    }
}

// Responde al Status Request con información del servidor
async fn handle_status(socket: &mut tokio::net::TcpStream) {
    let motd = r#"{
        "version": {
            "name": "1.20.1",
            "protocol": 763
        },
        "players": {
            "max": 10,
            "online": 0,
            "sample": []
        },
        "description": {
            "text": "Welcome to RustCraft!"
        }
    }"#;

    let mut response = vec![0x00]; // Packet ID para Status Response
    response.extend_from_slice(motd.as_bytes());
    let length = response.len() as u8;

    let mut packet = vec![length]; // Longitud del paquete
    packet.extend_from_slice(&response);

    if let Err(e) = socket.write_all(&packet).await {
        eprintln!("Failed to write Status response: {}", e);
    }
}

// Responde al Ping Request devolviendo el mismo payload
async fn handle_ping(socket: &mut tokio::net::TcpStream, buffer: &BytesMut) {
    if let Err(e) = socket.write_all(buffer).await {
        eprintln!("Failed to write Ping response: {}", e);
    }
}

// Maneja el inicio de sesión y cambia al estado Play
async fn handle_login_start(socket: &mut tokio::net::TcpStream) {
    let uuid = "00000000-0000-0000-0000-000000000000";
    let username = "TestPlayer";

    let mut packet = vec![0x02]; // Packet ID para Login Success
    packet.extend_from_slice(uuid.as_bytes());
    packet.extend_from_slice(username.as_bytes());

    if let Err(e) = socket.write_all(&packet).await {
        eprintln!("Failed to write Login Success response: {}", e);
    }

    println!("Login Success sent to player {}.", username);
}
