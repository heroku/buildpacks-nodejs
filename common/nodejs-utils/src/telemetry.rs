use std::{
    fs::{create_dir_all, File},
    path::Path,
};

use opentelemetry::{
    global::{self, BoxedTracer},
    KeyValue,
};
use opentelemetry_sdk::{
    trace::{Config, TracerProvider},
    Resource,
};

pub fn init_tracer(buildpack_name: impl Into<String>) -> BoxedTracer {
    let bp_name = buildpack_name.into();
    let telem_file_path = Path::new("/tmp")
        .join("cnb-telemetry")
        .join(format!("{}.jsonl", bp_name.replace('/', "_")));
    if let Some(parent_dir) = telem_file_path.parent() {
        let _ = create_dir_all(parent_dir);
    }
    let exporter = match File::create(telem_file_path) {
        Ok(f) => opentelemetry_stdout::SpanExporter::builder()
            .with_writer(f)
            .build(),
        Err(_) => opentelemetry_stdout::SpanExporter::default(),
    };
    let provider = TracerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                bp_name.clone(),
            )])),
        )
        .with_simple_exporter(exporter)
        .build();
    global::set_tracer_provider(provider);
    global::tracer(bp_name)
}

#[cfg(test)]
mod tests {
    use std::fs::{self};

    use super::init_tracer;
    use opentelemetry::{global, trace::TraceContextExt, trace::Tracer};

    #[test]
    fn test_tracer_writes_span_file() -> Result<(), String> {
        let buildpack_name = "heroku_test_buildpack";
        let test_span_name = "test_span_1";
        let test_event_name = "test_event_1";
        let telemetry_file_path = format!("/tmp/cnb-telemetry/{buildpack_name}.jsonl");

        let _ = fs::remove_file(&telemetry_file_path);

        init_tracer(buildpack_name.to_string());
        let tracer = global::tracer("");
        tracer.in_span(test_span_name, |cx| {
            cx.span().add_event(test_event_name, Vec::new());
        });
        global::shutdown_tracer_provider();
        let contents = fs::read_to_string(telemetry_file_path)
            .expect("expected to read existing telemetry file");
        println!("{contents}");

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
