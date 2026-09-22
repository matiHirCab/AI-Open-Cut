//! Opt-in render-only transport. All operation semantics remain in shared dispatch.
use super::*;
use std::io::BufRead;

#[cfg(windows)]
mod process_job;

const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;
const PROTOCOL_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    request_id: String,
    request: Request,
}

fn invalid(message: &str) -> CoreError {
    CoreError::new(opencut_editor_core::ErrorCode::InvalidArgument, message)
}

fn is_render(request: &Request) -> bool {
    matches!(
        request,
        Request::RenderPreview { .. }
            | Request::RenderPreviewRange { .. }
            | Request::RenderDraftPreview { .. }
            | Request::ExportVideo { .. }
    )
}

fn read_line(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>, CoreError> {
    let mut line = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|_| invalid("worker input unavailable"))?;
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Err(invalid("unterminated worker line"))
            };
        }
        let newline = available.iter().position(|b| *b == b'\n');
        let take = newline.unwrap_or(available.len());
        if take > MAX_LINE_BYTES - line.len() {
            return Err(invalid("worker line exceeds limit"));
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take + usize::from(newline.is_some()));
        if newline.is_some() {
            return Ok(Some(line));
        }
    }
}

struct BoundedLine(Vec<u8>);
impl Write for BoundedLine {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_LINE_BYTES - self.0.len() {
            return Err(io::Error::other("worker output exceeds limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn write_event(writer: &mut impl Write, event: impl Serialize) -> Result<(), CoreError> {
    let mut line = BoundedLine(Vec::new());
    serde_json::to_writer(&mut line, &event)?;
    writer
        .write_all(&line.0)
        .and_then(|()| writer.write_all(b"\n"))
        .and_then(|()| writer.flush())
        .map_err(|_| {
            CoreError::new(
                opencut_editor_core::ErrorCode::InternalError,
                "worker output unavailable",
            )
        })
}

pub(super) fn run(core: &EditorCore, renderer: &Renderer) -> Result<(), CoreError> {
    #[cfg(windows)]
    process_job::install()?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve(core, renderer, &mut stdin.lock(), &mut stdout.lock())
}

fn serve(
    core: &EditorCore,
    renderer: &Renderer,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<(), CoreError> {
    write_event(
        writer,
        serde_json::json!({"type":"ready","protocolVersion":PROTOCOL_VERSION}),
    )?;
    while let Some(line) = read_line(reader)? {
        let envelope: Envelope =
            serde_json::from_slice(&line).map_err(|_| invalid("invalid worker envelope"))?;
        let mut output_failed = false;
        let mut output = |event: serde_json::Value| match write_event(
            writer,
            serde_json::json!({"requestId":envelope.request_id,"event":event}),
        ) {
            Ok(()) => Ok(()),
            Err(error) => {
                output_failed = true;
                Err(error)
            }
        };
        let result = if !is_render(&envelope.request) {
            Err(invalid("render worker accepts only render operations"))
        } else {
            renderer
                .clone()
                .with_request_id(&envelope.request_id)
                .and_then(|scoped| {
                    dispatch(core, &scoped, envelope.request, &mut EventSink(&mut output))
                })
        };
        if output_failed {
            return Err(CoreError::new(
                opencut_editor_core::ErrorCode::InternalError,
                "worker output failed",
            ));
        }
        if let Err(error) = result {
            write_event(
                writer,
                serde_json::json!({"requestId":envelope.request_id,"event":{
                    "type":"error","error":ErrorBody {
                        code:error.code,message:error.message,retryable:error.retryable,
                        failed_stage:error.failed_stage,ffmpeg_exit_code:error.ffmpeg_exit_code,
                        ffmpeg_stderr_excerpt:error.ffmpeg_stderr_excerpt
                    }
                }}),
            )?;
        }
        #[cfg(feature = "raster-cache-test-hooks")]
        {
            let (hits, misses) = renderer.raster_cache_test_counts();
            eprintln!(
                "{}",
                serde_json::json!({"rasterCacheTest":{"requestId":envelope.request_id,"hits":hits,"misses":misses}})
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worker_contract_and_inclusive_framing() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/render-worker-v1.json")).unwrap();
        assert_eq!(fixture["maxLineBytes"], MAX_LINE_BYTES);
        assert_eq!(fixture["ready"]["protocolVersion"], PROTOCOL_VERSION);
        let mut names = Vec::new();
        for request in fixture["requests"].as_array().unwrap() {
            let envelope: Envelope = serde_json::from_value(request.clone()).unwrap();
            assert!(is_render(&envelope.request));
            names.push(request["request"]["operation"].clone());
        }
        assert_eq!(serde_json::json!(names), fixture["operations"]);
        for input in fixture["invalidEnvelopes"].as_array().unwrap() {
            assert!(serde_json::from_value::<Envelope>(input.clone()).is_err());
        }
        let rejected: Envelope =
            serde_json::from_value(fixture["rejectedOperation"].clone()).unwrap();
        assert!(!is_render(&rejected.request));
        for size in [MAX_LINE_BYTES, MAX_LINE_BYTES + 1] {
            let mut bytes = vec![b' '; size];
            bytes.push(b'\n');
            let result = read_line(&mut io::Cursor::new(bytes));
            assert_eq!(result.is_ok(), size == MAX_LINE_BYTES);
            let mut out = BoundedLine(Vec::new());
            assert_eq!(
                out.write_all(&vec![b' '; size]).is_ok(),
                size == MAX_LINE_BYTES
            );
        }
        assert!(read_line(&mut io::Cursor::new(b"truncated")).is_err());
        assert!(read_line(&mut io::Cursor::new(b"")).unwrap().is_none());
    }
}
