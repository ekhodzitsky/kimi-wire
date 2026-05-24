use kimi_wire::WireError;

#[test]
fn test_wire_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: WireError = io_err.into();
    assert!(matches!(err, WireError::Io(msg) if msg.contains("file not found")));
}

#[test]
fn test_wire_error_from_serde_json_parse_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let err: WireError = json_err.into();
    assert!(matches!(err, WireError::JsonParse(_)));
}

#[test]
fn test_wire_error_from_serde_json_io_error() {
    // A serde_json::Error that originates from I/O is mapped to WireError::Io.
    use std::io::Read;
    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("read failed"))
        }
    }
    let json_err = serde_json::from_reader::<_, serde_json::Value>(FailingReader).unwrap_err();
    let err: WireError = json_err.into();
    assert!(matches!(err, WireError::Io(_)));
}
