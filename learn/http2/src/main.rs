use bytes::Buf as _;
use bytes::BufMut as _;
use tokio::io::AsyncReadExt as _;
use tokio::io::AsyncWriteExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "google.com".to_owned());
    let tcp_stream = tokio::net::TcpStream::connect((host.clone(), 443)).await?;
    let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        tokio_rustls::rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let mut tls_config = tokio_rustls::rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    tls_config.alpn_protocols.push(b"h2".to_vec());
    tls_config.key_log = std::sync::Arc::new(tokio_rustls::rustls::KeyLogFile::new());
    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(tls_config));
    let tls_stream = connector
        .connect(
            tokio_rustls::rustls::ServerName::try_from(host.as_str())?,
            tcp_stream,
        )
        .await?;
    let (_, session) = tls_stream.get_ref();
    println!("Protocol version: {:?}", session.protocol_version());
    println!(
        "ALPN protocol: {:?}",
        session.alpn_protocol().map(String::from_utf8_lossy)
    );
    let (tls_reader, mut tls_writer) = tokio::io::split(tls_stream);
    const BUF_READER_CAP: u32 = 64 * 1024;
    let mut tls_reader = tokio::io::BufReader::with_capacity(BUF_READER_CAP as usize, tls_reader);

    // https://httpwg.org/specs/rfc7540.html#rfc.section.3.5
    let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    tls_writer.write_all(preface).await?;

    let mut window_size = BUF_READER_CAP;
    const HEADER_TABLE_SIZE: u16 = 4096;
    // Send SETTINGS frame
    {
        // https://httpwg.org/specs/rfc7540.html#rfc.section.6.5
        let mut payload = bytes::BytesMut::new();
        // Set SETTINGS_HEADER_TABLE_SIZE to HEADER_TABLE_SIZE
        payload.put_u16(0x1);
        payload.put_u32(HEADER_TABLE_SIZE as u32);
        // Set SETTINGS_ENABLE_PUSH to 0
        payload.put_u16(0x2);
        payload.put_u32(0);
        // Set SETTINGS_INITIAL_WINDOW_SIZE to window_size
        payload.put_u16(0x4);
        payload.put_u32(window_size);

        // https://httpwg.org/specs/rfc7540.html#rfc.section.4.1
        let mut frame = bytes::BytesMut::new();
        frame.put_uint(payload.len() as u64, 3); // Length
        frame.put_u8(0x4); // Type = SETTINGS (0x4)
        frame.put_u8(0x00); // Flags = 0 (ACK = 0)
        frame.put_u32(0x00000000); // Stream Identifier = 0
        frame.extend_from_slice(&payload);
        tls_writer.write_all(&frame).await?;
    }
    // Send HEADERS frame
    {
        // https://httpwg.org/specs/rfc7540.html#rfc.section.6.2
        // No Pad Length (No PADDED flag)
        // No Stream Dependency (No PRIORITY flag)
        // No Weight (No PRIORITY flag)
        let mut encoder = hpack_codec::Encoder::new(HEADER_TABLE_SIZE);
        let mut header_encoder = encoder.enter_header_block(Vec::new())?;
        header_encoder.encode_field(hpack_codec::table::StaticEntry::SchemeHttps)?;
        header_encoder.encode_field(hpack_codec::field::LiteralHeaderField::with_indexed_name(
            hpack_codec::table::StaticEntry::Authority,
            host.as_bytes(),
        ))?;
        header_encoder.encode_field(hpack_codec::table::StaticEntry::MethodGet)?;
        header_encoder.encode_field(hpack_codec::table::StaticEntry::PathRoot)?;
        let header_block_flagment = header_encoder.finish();
        // No Padding (No PADDED flag)

        let mut frame = bytes::BytesMut::new();
        frame.put_uint(header_block_flagment.len() as u64, 3); // Length
        frame.put_u8(0x1); // Type = HEADERS (0x1)
        frame.put_u8(0x1 | 0x4); // Flags = END_STREAM | END_HEADERS
        frame.put_u32(0x00000001); // Stream Identifier = 1
        frame.extend_from_slice(&header_block_flagment);
        tls_writer.write_all(&frame).await?;
    }

    // Read responses
    let mut http_body = bytes::BytesMut::new();
    let mut table_size = 4096;
    'read_frame: loop {
        if window_size < BUF_READER_CAP / 2 {
            const WINDOW_SIZE_INCREMENT: u32 = 16 * BUF_READER_CAP;
            for stream_identifier in 0..=1 {
                let mut frame = bytes::BytesMut::new();
                frame.put_uint(4, 3);
                frame.put_u8(0x8); // Type = WINDOW_UPDATE (0x8)
                frame.put_u8(0); // Flags = 0
                frame.put_u32(stream_identifier); // Stream Identifier = stream_identifier
                frame.put_u32(WINDOW_SIZE_INCREMENT);
                tls_writer.write_all(&frame).await?;
                println!(
                    "Send WINDOW_UPDATE to stream_identifier={}",
                    stream_identifier
                );
            }
            window_size += WINDOW_SIZE_INCREMENT;
        }

        println!(
            "Reading frame (buffer.len()={}, window_size={})",
            tls_reader.buffer().len(),
            window_size
        );
        // https://httpwg.org/specs/rfc7540.html#rfc.section.4.1
        let mut header = bytes::BytesMut::new();
        while header.len() < 9 {
            let mut buf = bytes::BytesMut::new();
            buf.resize(9 - header.len(), 0);
            let read_bytes = tls_reader.read(&mut buf).await?;
            if read_bytes == 0 {
                eprintln!("  Got EOF while reading frame header");
                break 'read_frame;
            } else {
                header.put(&buf[0..read_bytes]);
                eprintln!("  Read {}/9 bytes in total for header", header.len());
            }
        }
        let length = header.get_uint(3) as usize;
        let type_ = header.get_u8();
        let flags = header.get_u8();
        let stream_identifier = header.get_u32();
        println!("  frame.length = {}", length);
        println!("  frame.type = 0x{:x}", type_);
        println!("  frame.flags = 0x{:x}", flags);
        println!("  frame.stream_identifier = {}", stream_identifier);
        window_size -= length as u32;

        let mut payload = bytes::BytesMut::new();
        if length > 0 {
            while payload.len() < length {
                let mut buf = bytes::BytesMut::new();
                buf.resize(length - payload.len(), 0);
                println!("  Read payload for {} bytes", buf.len());
                let read_bytes = tls_reader.read(&mut buf).await?;
                if read_bytes == 0 {
                    eprintln!("  Got EOF while reading frame payload");
                    break 'read_frame;
                } else {
                    payload.put(&buf[0..read_bytes]);
                    eprintln!(
                        "  Read {}/{} bytes in total for payload",
                        payload.len(),
                        length
                    );
                }
            }
        }

        match type_ {
            0x0 => {
                // DATA frame
                // https://httpwg.org/specs/rfc7540.html#rfc.section.6.1
                if (flags & 0x8) == 0 {
                    http_body.put(payload);
                } else {
                    // PADDED
                    let pad_length = payload[0] as usize;
                    println!("Pad Length = {}", pad_length);
                    http_body.put(&payload[1..length - pad_length - 1]);
                }
                if (flags & 0x1) != 0 {
                    // https://httpwg.org/specs/rfc7540.html#rfc.section.6.4
                    println!("    Return RST_STREAM frame");
                    let mut frame = bytes::BytesMut::new();
                    frame.put_uint(4, 3); // Length
                    frame.put_u8(0x3); // Type = RST_STREAM
                    frame.put_u8(0x0); // Flags
                    frame.put_u32(stream_identifier);
                    frame.put_u32(0x5); // STREAM_CLOSED
                    tls_writer.write_all(&frame).await?;
                    break;
                }
            }
            0x1 => {
                // HEADERS frame
                // https://httpwg.org/specs/rfc7540.html#rfc.section.6.2
                let mut decoder = hpack_codec::Decoder::new(table_size);
                let mut header_decoder = decoder.enter_header_block(&payload)?;
                while let Some(field) = header_decoder.decode_field()? {
                    println!(
                        "    {}: {}",
                        String::from_utf8_lossy(field.name()),
                        String::from_utf8_lossy(field.value())
                    );
                }
            }
            0x3 => {
                // RST_STREAM frame
                // https://httpwg.org/specs/rfc7540.html#rfc.section.6.4
                let error_code = payload.get_u32();
                println!("    error_code: 0x{:x}", error_code);
            }
            0x4 => {
                // SETTINGS frame
                // https://httpwg.org/specs/rfc7540.html#SETTINGS
                if (flags & 0x1) == 0 {
                    let mut all_processed = true;
                    for _ in 0..(length / 6) {
                        let identifier = payload.get_u16();
                        let value = payload.get_u32();
                        let identifier_str = match identifier {
                            0x1 => {
                                table_size = value as u16;
                                "SETTINGS_HEADER_TABLE_SIZE"
                            }
                            0x2 => "SETTINGS_ENABLE_PUSH",
                            0x3 => "SETTINGS_MAX_CONCURRENT_STREAMS",
                            0x4 => "SETTINGS_INITIAL_WINDOW_SIZE",
                            0x5 => "SETTINGS_MAX_FRAME_SIZE",
                            0x6 => "SETTINGS_MAX_HEADER_LIST_SIZE",
                            _ => {
                                all_processed = false;
                                "UNKNOWN"
                            }
                        };
                        println!("    {} (0x{:x}) = {}", identifier_str, identifier, value);
                    }

                    if all_processed {
                        println!("    Return ACK response");
                        let mut frame = bytes::BytesMut::new();
                        frame.put_uint(0, 3); // Length
                        frame.put_u8(0x4); // Type = SETTINGS (0x4)
                        frame.put_u8(0x1); // Flags = 1 (ACK = 1)
                        frame.put_u32(0x00000000); // Stream Identifier = 0
                        tls_writer.write_all(&frame).await?;
                    }
                }
            }
            0x7 => {
                // GOAWAY frame
                // https://httpwg.org/specs/rfc7540.html#rfc.section.6.8
                let last_stream_id = payload.get_u32();
                let error_code = payload.get_u32();
                println!(
                    "    (last_stream_id={}) error_code: 0x{:x}",
                    last_stream_id, error_code
                );
                break;
            }
            0x8 => {
                // WINDOW_UPDATE frame
                // https://httpwg.org/specs/rfc7540.html#rfc.section.6.9
                if length != 4 {
                    // Should return FRAME_SIZE_ERROR
                    println!("    Invalid frame length for WINDOW_UPDATE: {}", length);
                }
                let window_size_increment = payload.get_u32();
                println!("    Window Size Increment: {}", window_size_increment);
            }
            _ => {
                println!("    Skip unknown frame type: 0x{:x}", type_);
            }
        }
    }
    if !http_body.is_empty() {
        if !http_body.ends_with(b"\n") {
            http_body.put_u8(b'\n');
        }
        tokio::io::stdout().write_all(&http_body).await?;
    }
    Ok(())
}
