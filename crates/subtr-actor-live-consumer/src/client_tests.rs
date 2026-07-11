use super::*;

#[test]
fn parses_bare_host_and_port() {
    let parts = parse_ws_url("ws://127.0.0.1:9000").unwrap();
    assert_eq!(parts.host, "127.0.0.1");
    assert_eq!(parts.port, 9000);
    assert_eq!(parts.path, "/");
}

#[test]
fn parses_path_and_query() {
    let parts = parse_ws_url("ws://localhost:8080/?format=json").unwrap();
    assert_eq!(parts.host, "localhost");
    assert_eq!(parts.port, 8080);
    assert_eq!(parts.path, "/?format=json");
}

#[test]
fn rejects_missing_scheme_port_or_host() {
    for url in [
        "http://127.0.0.1:9000",
        "ws://127.0.0.1",
        "ws://:9000",
        "ws://127.0.0.1:notaport",
        "wss://127.0.0.1:9000",
    ] {
        assert!(
            matches!(parse_ws_url(url), Err(ClientError::InvalidUrl(_))),
            "expected InvalidUrl for {url}"
        );
    }
}
