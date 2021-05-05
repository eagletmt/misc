use bytes::Buf as _;
use bytes::BufMut as _;
use prost::Message as _;
use tokio::io::AsyncReadExt as _;
use tokio::io::AsyncWriteExt as _;

#[derive(Debug)]
struct Frame<B> {
    type_: u8,
    flags: u8,
    stream_identifier: u32,
    payload: B,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "localhost";
    let tcp_stream = tokio::net::TcpStream::connect((host, 50051)).await?;
    let (tcp_reader, mut tcp_writer) = tokio::io::split(tcp_stream);
    let mut tcp_reader = tokio::io::BufReader::new(tcp_reader);

    let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    tcp_writer.write_all(preface).await?;

    const INITIAL_WINDOW_SIZE: u32 = 16 * 1024;
    let mut window_size = INITIAL_WINDOW_SIZE;
    const HEADER_TABLE_SIZE: u16 = 4096;

    const FRAME_TYPE_SETTINGS: u8 = 0x4;
    const SETTINGS_HEADER_TABLE_SIZE: u16 = 0x1;
    const SETTINGS_ENABLE_PUSH: u16 = 0x2;
    const SETTINGS_INITIAL_WINDOW_SIZE: u16 = 0x4;
    {
        // Send SETTINGS frame
        let mut payload = bytes::BytesMut::new();
        payload.put_u16(SETTINGS_HEADER_TABLE_SIZE);
        payload.put_u32(HEADER_TABLE_SIZE as u32);
        payload.put_u16(SETTINGS_ENABLE_PUSH);
        payload.put_u32(0);
        payload.put_u16(SETTINGS_INITIAL_WINDOW_SIZE);
        payload.put_u32(window_size);
        write_frame(
            &mut tcp_writer,
            Frame {
                type_: FRAME_TYPE_SETTINGS,
                flags: 0,
                stream_identifier: 0,
                payload,
            },
        )
        .await?;
    }
    const FRAME_TYPE_HEADERS: u8 = 0x1;
    {
        // Send HEADERS frame
        let mut encoder = hpack_codec::Encoder::new(HEADER_TABLE_SIZE);
        let mut header_encoder = encoder.enter_header_block(Vec::new())?;
        header_encoder.encode_field(hpack_codec::table::StaticEntry::MethodPost)?;
        header_encoder.encode_field(hpack_codec::table::StaticEntry::SchemeHttp)?;
        header_encoder.encode_field(hpack_codec::field::LiteralHeaderField::with_indexed_name(
            hpack_codec::table::StaticEntry::Path,
            b"/routeguide.RouteGuide/RecordRoute",
        ))?;
        header_encoder.encode_field(hpack_codec::field::LiteralHeaderField::with_indexed_name(
            hpack_codec::table::StaticEntry::Authority,
            b"localhost:50051",
        ))?;
        header_encoder.encode_field(hpack_codec::field::LiteralHeaderField::with_indexed_name(
            hpack_codec::table::StaticEntry::ContentType,
            b"application/grpc+proto",
        ))?;
        header_encoder.encode_field(hpack_codec::field::LiteralHeaderField::new(
            b"te",
            b"trailers",
        ))?;
        let header_block_flagment = header_encoder.finish();

        write_frame(
            &mut tcp_writer,
            Frame {
                type_: FRAME_TYPE_HEADERS,
                flags: 0x4,
                stream_identifier: 1,
                payload: header_block_flagment,
            },
        )
        .await?;
    }
    const FRAME_TYPE_DATA: u8 = 0x0;
    {
        // Send multiple Length-Prefixed-Message via DATA frame
        async fn send_point<W>(
            writer: &mut W,
            point: grpc::protos::Point,
        ) -> Result<(), Box<dyn std::error::Error>>
        where
            W: tokio::io::AsyncWrite + Unpin,
        {
            let mut protobuf = bytes::BytesMut::with_capacity(point.encoded_len());
            point.encode(&mut protobuf)?;
            println!("request protobuf: {:?}", protobuf);
            let mut payload = bytes::BytesMut::with_capacity(protobuf.len() + 5);
            payload.put_u8(0); // Compressed-Flag = 0
            payload.put_u32(protobuf.len() as u32); // Message-Length
            payload.put(protobuf);
            write_frame(
                writer,
                Frame {
                    type_: FRAME_TYPE_DATA,
                    flags: 0x0,
                    stream_identifier: 1,
                    payload,
                },
            )
            .await?;
            Ok(())
        }

        send_point(
            &mut tcp_writer,
            grpc::protos::Point {
                latitude: 407838351,
                longitude: -746143763,
            },
        )
        .await?;
        send_point(
            &mut tcp_writer,
            grpc::protos::Point {
                latitude: 408122808,
                longitude: -743999179,
            },
        )
        .await?;
        send_point(
            &mut tcp_writer,
            grpc::protos::Point {
                latitude: 413628156,
                longitude: -749015468,
            },
        )
        .await?;
        write_frame(
            &mut tcp_writer,
            Frame {
                type_: FRAME_TYPE_DATA,
                flags: 0x1,
                stream_identifier: 1,
                payload: &[],
            },
        )
        .await?;
    }

    const FRAME_TYPE_RST_STREAM: u8 = 0x3;
    const FRAME_TYPE_PING: u8 = 0x6;
    const FRAME_TYPE_WINDOW_UPDATE: u8 = 0x8;
    let mut response = bytes::BytesMut::new();
    loop {
        if window_size < INITIAL_WINDOW_SIZE / 2 {
            const WINDOW_SIZE_INCREMENT: u32 = 16 * INITIAL_WINDOW_SIZE;
            for stream_identifier in 0..=1 {
                let mut payload = bytes::BytesMut::new();
                payload.put_u32(WINDOW_SIZE_INCREMENT);
                write_frame(
                    &mut tcp_writer,
                    Frame {
                        type_: FRAME_TYPE_WINDOW_UPDATE,
                        flags: 0,
                        stream_identifier,
                        payload,
                    },
                )
                .await?;
            }
            window_size += WINDOW_SIZE_INCREMENT;
        }

        let mut header = read_bytes(&mut tcp_reader, 9).await?;
        let length = header.get_uint(3) as usize;
        let type_ = header.get_u8();
        let flags = header.get_u8();
        let stream_identifier = header.get_u32();
        let payload = read_bytes(&mut tcp_reader, length).await?;
        let mut frame = Frame {
            type_,
            flags,
            stream_identifier,
            payload,
        };
        println!(
            "Read frame: type=0x{:x} flags=0x{:x} stream_identifier={}",
            frame.type_, frame.flags, frame.stream_identifier
        );

        match frame.type_ {
            FRAME_TYPE_DATA => {
                response.put(frame.payload);
            }
            FRAME_TYPE_HEADERS => {
                let mut decoder = hpack_codec::Decoder::new(4096);
                let mut header_decoder = decoder.enter_header_block(&frame.payload)?;
                while let Some(field) = header_decoder.decode_field()? {
                    println!(
                        "    {}: {}",
                        String::from_utf8_lossy(field.name()),
                        String::from_utf8_lossy(field.value())
                    );
                }
                if (frame.flags & 0x1) != 0 {
                    break;
                }
            }
            FRAME_TYPE_RST_STREAM => {
                let error_code = frame.payload.get_u32();
                println!("    error_code: 0x{:x}", error_code);
                break;
            }
            FRAME_TYPE_SETTINGS => {
                if (frame.flags & 0x1) == 0 {
                    write_frame(
                        &mut tcp_writer,
                        Frame {
                            type_: FRAME_TYPE_SETTINGS,
                            flags: 0x1,
                            stream_identifier: 0,
                            payload: &[],
                        },
                    )
                    .await?;
                }
            }
            FRAME_TYPE_PING => {
                if (frame.flags & 0x1) == 0 {
                    write_frame(
                        &mut tcp_writer,
                        Frame {
                            type_: FRAME_TYPE_PING,
                            flags: 0x1,
                            stream_identifier: 0,
                            payload: frame.payload,
                        },
                    )
                    .await?;
                }
            }
            FRAME_TYPE_WINDOW_UPDATE => {
                let window_size_increment = frame.payload.get_u32();
                println!("  window_size_increment={}", window_size_increment);
            }
            _ => {
                println!("  Skip unknown frame type");
            }
        }
    }

    let compressed_flag = response.get_u8();
    let message_length = response.get_u32();
    response.truncate(message_length as usize);
    let reply = grpc::protos::RouteSummary::decode(response)?;
    println!("compressed_flag={} {:?}", compressed_flag, reply);

    Ok(())
}

async fn write_frame<W, B>(
    writer: &mut W,
    frame: Frame<B>,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: tokio::io::AsyncWrite + Unpin,
    B: AsRef<[u8]>,
{
    let mut buf = bytes::BytesMut::new();
    buf.put_uint(frame.payload.as_ref().len() as u64, 3);
    buf.put_u8(frame.type_);
    buf.put_u8(frame.flags);
    buf.put_u32(frame.stream_identifier);
    buf.extend_from_slice(frame.payload.as_ref());
    writer.write_all(&buf).await?;
    Ok(())
}

async fn read_bytes<R>(
    reader: &mut R,
    length: usize,
) -> Result<bytes::BytesMut, Box<dyn std::error::Error>>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut buf = bytes::BytesMut::new();
    while buf.len() < length {
        let mut tmp = bytes::BytesMut::new();
        tmp.resize(length - buf.len(), 0);
        let n = reader.read(&mut tmp).await?;
        if n == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Got EOF while reading frame header",
            )));
        } else {
            buf.put(&tmp[0..n]);
        }
    }
    Ok(buf)
}
