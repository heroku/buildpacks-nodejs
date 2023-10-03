use std::{
    fs::{create_dir_all, File},
    path::Path,
};

use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    trace::{Config, TracerProvider},
    Resource,
};

pub fn init_tracing(buildpack_name: impl Into<String>) -> TracerProvider {
    let bp_name = buildpack_name.into();
    let telem_file_path = Path::new("/tmp")
        .join("cnb-telemetry")
        .join(format!("{}.jsonl", bp_name.replace('/', "_")));

    // Ensure the telemetry dir exists
    if let Some(parent_dir) = telem_file_path.parent() {
        let _ = create_dir_all(parent_dir);
    }

    // Create a telemetry exporter that writes to the telemetry file
    // (or /dev/null in case of file issues)
    let exporter = match File::options()
        .create(true)
        .append(true)
        .open(&telem_file_path)
    {
        Ok(f) => opentelemetry_stdout::SpanExporter::builder()
            .with_writer(std::io::BufWriter::new(NoisyFileWriter(f)))
            .build(),
        Err(err) => {
            println!("Error writing to file {telem_file_path:?}: {err}");
            opentelemetry_stdout::SpanExporter::builder()
                .with_writer(std::io::sink())
                .build()
        }
    };

    // Create a global tracer provider with the exporter.
    let provider = TracerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                bp_name.clone(),
            )])),
        )
        .with_simple_exporter(exporter)
        .build();
    global::set_tracer_provider(provider.clone());
    provider
}

#[cfg(test)]
mod tests {
    use std::fs::{self};

    use super::init_tracing;
    use opentelemetry::{global, trace::TraceContextExt, trace::Tracer};

    #[test]
    fn test_tracer_writes_span_file() -> Result<(), String> {
        let buildpack_name = "heroku_test_buildpack";
        let test_span_name = "test_span_1";
        let test_event_name = "test_event_1";
        let telemetry_file_path = format!("/tmp/cnb-telemetry/{buildpack_name}.jsonl");

        let _ = fs::remove_file(&telemetry_file_path);

        init_tracing(buildpack_name.to_string());
        let tracer = global::tracer("");
        tracer.in_span(test_span_name, |cx| {
            cx.span().add_event(test_event_name, Vec::new());
        });
        global::shutdown_tracer_provider();

        let contents = fs::read_to_string(telemetry_file_path)
            .expect("expected to read existing telemetry file");
        println!("Contents: {contents}");

        if !contents.contains(buildpack_name) {
            Err("File export did not include buildpack name")?;
        }

        if !contents.contains(test_span_name) {
            Err("File export did not include test span")?;
        }

        if !contents.contains(test_event_name) {
            Err("File export did not include test event")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct NoisyFileWriter(File);
impl std::io::Write for NoisyFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        println!("Writing to {self:?}! {}", String::from_utf8_lossy(buf));
        self.0.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        println!("Flushing {self:?}!");
        self.0.flush()
    }
}
