use anyhow::Context as _;
use clap::Parser as _;
use std::io::BufRead as _;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Path to Parquet schema file
    #[clap(short, long)]
    schema: std::path::PathBuf,
    /// Path to input JSON Lines file
    #[clap(short, long)]
    input: std::path::PathBuf,
    /// Path to output Parquet file
    #[clap(short, long)]
    output: std::path::PathBuf,
    /// Write batch size
    #[clap(long, default_value_t = 1024)]
    write_batch_size: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let schema_str = std::fs::read_to_string(&args.schema).with_context(|| {
        format!(
            "failed to read Parquet schema file {}",
            args.schema.display()
        )
    })?;
    let schema = parquet::schema::parser::parse_message_type(&schema_str)
        .context("failed to parse Parquet schema")?;
    let schema = std::sync::Arc::new(schema);

    let mut rows = Vec::new();
    let file = std::fs::File::open(&args.input).with_context(|| {
        format!(
            "failed to open input JSON Lines file {}",
            args.input.display()
        )
    })?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.context("failed to read line from JSON Lines file")?;
        let row: serde_json::Value = serde_json::from_str(&line)
            .with_context(|| format!("failed to deserialize JSON {}", line))?;
        rows.push(row);
    }

    let file = std::fs::File::create(&args.output).with_context(|| {
        format!(
            "failed to create output Parquet file {}",
            args.output.display()
        )
    })?;
    let props = parquet::file::properties::WriterProperties::builder()
        .set_compression(parquet::basic::Compression::SNAPPY)
        .set_write_batch_size(args.write_batch_size)
        .build();
    let props = std::sync::Arc::new(props);
    let mut writer =
        parquet::file::writer::SerializedFileWriter::new(file, schema.clone(), props.clone())
            .context("failed to initialize SerializedFileWriter")?;
    json_to_parquet(&schema, &props, rows, &mut writer)?;
    writer
        .close()
        .context("failed to close SerializedFileWriter")?;

    Ok(())
}

fn json_to_parquet<W>(
    schema: &parquet::schema::types::Type,
    props: &parquet::file::properties::WriterProperties,
    rows: Vec<serde_json::Value>,
    writer: &mut parquet::file::writer::SerializedFileWriter<W>,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let mut row_group_writer = writer
        .next_row_group()
        .context("failed to initialize next SerializedRowGroupWriter")?;

    for field_type in schema.get_fields() {
        match field_type.as_ref() {
            parquet::schema::types::Type::PrimitiveType { physical_type, .. } => {
                match physical_type {
                    parquet::basic::Type::BOOLEAN => convert_column(
                        field_type,
                        props,
                        &rows,
                        &mut row_group_writer,
                        boolean_writer,
                    ),
                    parquet::basic::Type::INT32 => convert_column(
                        field_type,
                        props,
                        &rows,
                        &mut row_group_writer,
                        int32_writer,
                    ),
                    parquet::basic::Type::INT64 => convert_column(
                        field_type,
                        props,
                        &rows,
                        &mut row_group_writer,
                        int64_writer,
                    ),
                    parquet::basic::Type::INT96 => anyhow::bail!("INT96 is not supported"),
                    parquet::basic::Type::FLOAT => convert_column(
                        field_type,
                        props,
                        &rows,
                        &mut row_group_writer,
                        float_writer,
                    ),
                    parquet::basic::Type::DOUBLE => convert_column(
                        field_type,
                        props,
                        &rows,
                        &mut row_group_writer,
                        double_writer,
                    ),
                    parquet::basic::Type::BYTE_ARRAY => convert_column(
                        field_type,
                        props,
                        &rows,
                        &mut row_group_writer,
                        byte_array_writer,
                    ),
                    parquet::basic::Type::FIXED_LEN_BYTE_ARRAY => {
                        anyhow::bail!("FIXED_LEN_BYTE_ARRAY is not supported")
                    }
                }?
            }
            _ => unimplemented!("non-primitive type is not supported"),
        }
    }

    row_group_writer
        .close()
        .context("failed to close SerializedRowGroupWriter")?;

    Ok(())
}

fn convert_column<W, F>(
    field_type: &parquet::schema::types::Type,
    props: &parquet::file::properties::WriterProperties,
    rows: &[serde_json::Value],
    row_group_writer: &mut parquet::file::writer::SerializedRowGroupWriter<W>,
    writer: F,
) -> anyhow::Result<()>
where
    W: std::io::Write,
    F: Fn(
        &parquet::schema::types::Type,
        &mut parquet::file::writer::SerializedColumnWriter,
        Vec<Option<&serde_json::Value>>,
    ) -> anyhow::Result<()>,
{
    let mut col_writer = row_group_writer
        .next_column()
        .context("failed to initialize next SerializedColumnWriter")?
        .unwrap();
    for chunk in rows.chunks(props.write_batch_size()) {
        let values = chunk.iter().map(|row| row.get(field_type.name())).collect();
        writer(field_type, &mut col_writer, values)?;
    }
    col_writer
        .close()
        .context("failed to close SerializedColumnWriter")
}

fn generic_writer<'a, T, F>(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::column::writer::ColumnWriterImpl<'a, T>,
    json_values: Vec<Option<&serde_json::Value>>,
    coerce: F,
) -> anyhow::Result<()>
where
    T: parquet::data_type::DataType,
    F: Fn(&serde_json::Value) -> anyhow::Result<T::T>,
{
    let mut values = Vec::with_capacity(json_values.len());
    let mut def_levels = Vec::with_capacity(json_values.len());
    for json_value in json_values {
        if field_type.is_optional() && (json_value.is_none() || json_value.unwrap().is_null()) {
            def_levels.push(0);
        } else {
            let json_value = json_value.with_context(|| {
                format!("{} column is required but not present", field_type.name())
            })?;
            let v = coerce(json_value)?;
            values.push(v);
            def_levels.push(1);
        }
    }
    col_writer
        .write_batch(&values, Some(&def_levels), None)
        .with_context(|| {
            format!(
                "failed to write {} column values of type {}",
                field_type.name(),
                T::get_physical_type(),
            )
        })?;
    Ok(())
}

fn boolean_writer(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::file::writer::SerializedColumnWriter,
    json_values: Vec<Option<&serde_json::Value>>,
) -> anyhow::Result<()> {
    generic_writer(
        field_type,
        col_writer.typed::<parquet::data_type::BoolType>(),
        json_values,
        |json_value| {
            json_value.as_bool().with_context(|| {
                format!(
                    "{} column expects BOOLEAN value but got {}",
                    field_type.name(),
                    json_value
                )
            })
        },
    )
}

fn int32_writer(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::file::writer::SerializedColumnWriter,
    json_values: Vec<Option<&serde_json::Value>>,
) -> anyhow::Result<()> {
    generic_writer(
        field_type,
        col_writer.typed::<parquet::data_type::Int32Type>(),
        json_values,
        |json_value| match field_type.get_basic_info().converted_type() {
            parquet::basic::ConvertedType::NONE => json_value
                .as_i64()
                .with_context(|| {
                    format!(
                        "{} column expects INT32 value but got {}",
                        field_type.name(),
                        json_value
                    )
                })?
                .try_into()
                .with_context(|| {
                    format!(
                        "failed to convert {} column value to INT32 type: {}",
                        field_type.name(),
                        json_value
                    )
                }),
            parquet::basic::ConvertedType::DATE => {
                let s = json_value.as_str().with_context(|| {
                    format!(
                        "{} column expects DATE value but got {}",
                        field_type.name(),
                        json_value
                    )
                })?;
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .with_context(|| {
                        format!(
                            "failed to parse {} column DATE value: {}",
                            field_type.name(),
                            s
                        )
                    })
                    .map(|d| {
                        let unix_epoch_date = chrono::NaiveDate::from_ymd(1970, 1, 1);
                        (d - unix_epoch_date).num_days() as i32
                    })
            }
            ct => anyhow::bail!("unsupported converted type for INT32: {}", ct),
        },
    )
}

fn int64_writer(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::file::writer::SerializedColumnWriter,
    json_values: Vec<Option<&serde_json::Value>>,
) -> anyhow::Result<()> {
    generic_writer(
        field_type,
        col_writer.typed::<parquet::data_type::Int64Type>(),
        json_values,
        |json_value| match field_type.get_basic_info().converted_type() {
            parquet::basic::ConvertedType::NONE => json_value.as_i64().with_context(|| {
                format!(
                    "{} column expects INT64 but got {}",
                    field_type.name(),
                    json_value
                )
            }),
            parquet::basic::ConvertedType::TIMESTAMP_MILLIS => {
                let s = json_value.as_str().with_context(|| {
                    format!(
                        "{} column expects TIMESTAMP_MILLIS but got value: {}",
                        field_type.name(),
                        json_value
                    )
                })?;
                chrono::DateTime::parse_from_rfc3339(s)
                    .with_context(|| {
                        format!(
                            "failed to parse {} column TIMESTAMP_MILLIS value: {}",
                            field_type.name(),
                            s
                        )
                    })
                    .map(|t| t.timestamp() * 1000 + t.timestamp_subsec_millis() as i64)
            }
            parquet::basic::ConvertedType::TIMESTAMP_MICROS => {
                let s = json_value.as_str().with_context(|| {
                    format!(
                        "{} column expects TIMESTAMP_MICROS but got value: {}",
                        field_type.name(),
                        json_value
                    )
                })?;
                chrono::DateTime::parse_from_rfc3339(s)
                    .with_context(|| {
                        format!(
                            "failed to parse {} column TIMESTAMP_MICROS value: {}",
                            field_type.name(),
                            s
                        )
                    })
                    .map(|t| t.timestamp() * 1000000 + t.timestamp_subsec_micros() as i64)
            }
            ct => anyhow::bail!("unsupported converted type for INT64: {}", ct),
        },
    )
}

fn float_writer(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::file::writer::SerializedColumnWriter,
    json_values: Vec<Option<&serde_json::Value>>,
) -> anyhow::Result<()> {
    generic_writer(
        field_type,
        col_writer.typed::<parquet::data_type::FloatType>(),
        json_values,
        |json_value| {
            json_value
                .as_f64()
                .with_context(|| {
                    format!(
                        "{} column expects FLOAT value bot got {}",
                        field_type.name(),
                        json_value
                    )
                })
                .map(|n| n as f32)
        },
    )
}

fn double_writer(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::file::writer::SerializedColumnWriter,
    json_values: Vec<Option<&serde_json::Value>>,
) -> anyhow::Result<()> {
    generic_writer(
        field_type,
        col_writer.typed::<parquet::data_type::DoubleType>(),
        json_values,
        |json_value| {
            json_value.as_f64().with_context(|| {
                format!(
                    "{} column expects DOUBLE value bot got {}",
                    field_type.name(),
                    json_value
                )
            })
        },
    )
}

fn byte_array_writer(
    field_type: &parquet::schema::types::Type,
    col_writer: &mut parquet::file::writer::SerializedColumnWriter,
    json_values: Vec<Option<&serde_json::Value>>,
) -> anyhow::Result<()> {
    generic_writer(
        field_type,
        col_writer.typed::<parquet::data_type::ByteArrayType>(),
        json_values,
        |json_value| {
            json_value
                .as_str()
                .with_context(|| {
                    format!(
                        "{} column expects BYTE_ARRAY value bot got {}",
                        field_type.name(),
                        json_value
                    )
                })
                .map(Into::into)
        },
    )
}

#[cfg(test)]
mod tests {
    use bytes::BufMut as _;
    use parquet::file::reader::FileReader as _;

    #[test]
    fn it_works() {
        let schema = parquet::schema::parser::parse_message_type(
            r#"
            message test {
                required int64 t (timestamp(millis, true));
                optional int32 n;
                optional binary s (string);
                optional int64 t2 (timestamp(millis, true));
                optional double d;
                optional float f;
                optional boolean b;
                optional int32 dt (date);
                optional int64 tm (timestamp(micros, true));
            }
        "#,
        )
        .unwrap();
        let rows = vec![
            serde_json::json!({
                "t": "2022-09-03T10:14:42.831Z",
                "n": 1,
                "s": "2",
                "t2": "2022-09-05T11:15:43.942Z",
                "d": 3.4,
                "f": 2.0,
                "b": true,
                "dt": "2022-09-05",
                "tm": "2022-09-05T11:15:43.042878Z",
                "rest": "ignored",
            }),
            serde_json::json!({
                "t": "2022-09-04T06:13:22.033Z",
            }),
            serde_json::json!({
                "t": "2022-09-04T14:06:55.142Z",
                "n": null,
            }),
        ];
        let schema = std::sync::Arc::new(schema);
        let props =
            std::sync::Arc::new(parquet::file::properties::WriterProperties::builder().build());
        let mut writer = parquet::file::writer::SerializedFileWriter::new(
            bytes::BytesMut::new().writer(),
            schema.clone(),
            props.clone(),
        )
        .unwrap();
        super::json_to_parquet(&schema, &props, rows, &mut writer).unwrap();
        let buf = writer.into_inner().unwrap().into_inner().freeze();

        let reader = parquet::file::reader::SerializedFileReader::new(buf).unwrap();
        assert_eq!(reader.metadata().num_row_groups(), 1);
        let row_group = reader.metadata().row_group(0);
        assert_eq!(row_group.num_columns(), 9);
        let columns = row_group.columns();
        assert_eq!(columns[0].column_type(), parquet::basic::Type::INT64);
        assert_eq!(columns[0].num_values(), 3);
        assert_eq!(columns[1].column_type(), parquet::basic::Type::INT32);
        assert_eq!(columns[1].num_values(), 3);
        assert_eq!(columns[2].column_type(), parquet::basic::Type::BYTE_ARRAY);
        assert_eq!(columns[2].num_values(), 3);
        assert_eq!(columns[3].column_type(), parquet::basic::Type::INT64);
        assert_eq!(columns[3].num_values(), 3);
        assert_eq!(columns[4].column_type(), parquet::basic::Type::DOUBLE);
        assert_eq!(columns[4].num_values(), 3);
        assert_eq!(columns[5].column_type(), parquet::basic::Type::FLOAT);
        assert_eq!(columns[5].num_values(), 3);
        assert_eq!(columns[6].column_type(), parquet::basic::Type::BOOLEAN);
        assert_eq!(columns[6].num_values(), 3);
        assert_eq!(columns[7].column_type(), parquet::basic::Type::INT32);
        assert_eq!(columns[7].num_values(), 3);
        assert_eq!(columns[8].column_type(), parquet::basic::Type::INT64);
        assert_eq!(columns[8].num_values(), 3);

        let rows: Vec<parquet::record::Row> = reader.get_row_iter(None).unwrap().collect();
        assert_eq!(
            rows[0].to_json_value(),
            serde_json::json!({
                "t": "2022-09-03 10:14:42 +00:00",
                "n": 1,
                "s": "2",
                "t2": "2022-09-05 11:15:43 +00:00",
                "d": 3.4,
                "f": 2.0,
                "b": true,
                "dt": "2022-09-05 +00:00",
                "tm": "2022-09-05 11:15:43 +00:00",
            })
        );
        assert_eq!(
            rows[1].to_json_value(),
            serde_json::json!({
                "t": "2022-09-04 06:13:22 +00:00",
                "n": null,
                "s": null,
                "t2": null,
                "d": null,
                "f": null,
                "b": null,
                "dt": null,
                "tm": null,
            })
        );
        assert_eq!(
            rows[2].to_json_value(),
            serde_json::json!({
                "t": "2022-09-04 14:06:55 +00:00",
                "n": null,
                "s": null,
                "t2": null,
                "d": null,
                "f": null,
                "b": null,
                "dt": null,
                "tm": null,
            })
        );
    }

    #[test]
    fn it_fails_when_type_mismatch() {
        let schema = parquet::schema::parser::parse_message_type(
            r#"
            message test {
                optional int32 n;
            }
        "#,
        )
        .unwrap();
        let rows = vec![serde_json::json!({
            "n": "1",
        })];
        let schema = std::sync::Arc::new(schema);
        let props =
            std::sync::Arc::new(parquet::file::properties::WriterProperties::builder().build());
        let mut writer = parquet::file::writer::SerializedFileWriter::new(
            bytes::BytesMut::new().writer(),
            schema.clone(),
            props.clone(),
        )
        .unwrap();
        assert!(super::json_to_parquet(&schema, &props, rows, &mut writer).is_err());
    }

    #[test]
    fn it_fails_when_required_field_is_missing() {
        let schema = parquet::schema::parser::parse_message_type(
            r#"
            message test {
                required int32 n;
            }
        "#,
        )
        .unwrap();
        let rows = vec![serde_json::json!({
            "m": 1,
        })];
        let schema = std::sync::Arc::new(schema);
        let props =
            std::sync::Arc::new(parquet::file::properties::WriterProperties::builder().build());
        let mut writer = parquet::file::writer::SerializedFileWriter::new(
            bytes::BytesMut::new().writer(),
            schema.clone(),
            props.clone(),
        )
        .unwrap();
        assert!(super::json_to_parquet(&schema, &props, rows, &mut writer).is_err());
    }
}
