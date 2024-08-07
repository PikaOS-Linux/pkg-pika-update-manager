use chrono;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::task;

pub async fn send_successful_to_socket(socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("Could not connect to server");

    let message = "FN_OVERRIDE_SUCCESSFUL";

    // Send the message to the server
    stream
        .write_all(message.as_bytes())
        .await
        .expect("Failed to write to stream");
}

pub async fn send_failed_to_socket(socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("Could not connect to server");

    let message = "FN_OVERRIDE_FAILED";

    // Send the message to the server
    stream
        .write_all(message.as_bytes())
        .await
        .expect("Failed to write to stream");
}

// Function to handle a single client connection
pub async fn handle_client(
    mut stream: UnixStream,
    buffer_sender: async_channel::Sender<String>,
    log_file_path: String,
) {
    // Buffer to store incoming data
    let mut buffer = [0; 1024];

    if !Path::new(&log_file_path).exists() {
        match fs::File::create(&log_file_path) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Warning: {} file couldn't be created", log_file_path);
            }
        };
    }

    // Read data from the stream
    match stream.read(&mut buffer).await {
        Ok(size) => {
            let message = String::from_utf8_lossy(&buffer[..size]).to_string();
            // Send to async buffer sender
            buffer_sender
                .send_blocking(message.clone())
                .expect("Buffer channel closed");
            // Write to log file
            let mut log_file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&log_file_path)
                .unwrap();

            if let Err(e) = writeln!(
                log_file,
                "[{}] {}",
                chrono::offset::Local::now().format("%Y/%m/%d_%H:%M"),
                message
            ) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
        Err(e) => {
            // Print error message if reading fails
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}

pub async fn start_socket_server(
    buffer_sender: async_channel::Sender<String>,
    socket_path: &str,
    log_file_path: &str,
) {
    // Remove the socket file if it already exists
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Could not remove existing socket file");
    }

    // Bind the Unix listener to the socket path
    let listener = UnixListener::bind(socket_path).expect("Could not bind");

    println!("Server listening on {}", socket_path);

    // Loop to accept incoming connections
    loop {
        // Accept an incoming connection
        match listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection in a separate task
                task::spawn(handle_client(
                    stream,
                    buffer_sender.clone(),
                    log_file_path.to_owned(),
                ));
            }
            Err(e) => {
                // Print error message if a connection fails
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

pub async fn handle_client_no_log(
    mut stream: UnixStream,
    buffer_sender: async_channel::Sender<String>,
) {
    // Buffer to store incoming data
    let mut buffer = [0; 1024];

    // Read data from the stream
    match stream.read(&mut buffer).await {
        Ok(size) => {
            let message = String::from_utf8_lossy(&buffer[..size]).to_string();
            // Write to log file

            // Send to async buffer sender
            buffer_sender
                .send_blocking(message)
                .expect("Buffer channel closed")
        }
        Err(e) => {
            // Print error message if reading fails
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}

pub async fn start_socket_server_no_log(
    buffer_sender: async_channel::Sender<String>,
    socket_path: &str,
) {
    // Remove the socket file if it already exists
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).expect("Could not remove existing socket file");
    }

    // Bind the Unix listener to the socket path
    let listener = UnixListener::bind(socket_path).expect("Could not bind");

    println!("Server listening on {}", socket_path);

    // Loop to accept incoming connections
    loop {
        // Accept an incoming connection
        match listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection in a separate task
                task::spawn(handle_client_no_log(stream, buffer_sender.clone()));
            }
            Err(e) => {
                // Print error message if a connection fails
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
