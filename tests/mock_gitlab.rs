use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Mock(Child, u16);

impl Drop for Mock {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn python() -> Option<&'static str> {
    ["python3", "python"].into_iter().find(|p| {
        Command::new(p)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

fn start(extra: &[&str]) -> Option<Mock> {
    let py = python()?;
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let child = Command::new(py)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tools/demo/mock_gitlab.py"
        ))
        .arg(port.to_string())
        .args(extra)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mock = Mock(child, port);
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Some(mock);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

fn request(port: u16, method: &str, path: &str) -> (String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let (head, body) = out.split_once("\r\n\r\n").unwrap();
    (head.to_string(), body.to_string())
}

#[test]
fn mock_paginates_counts_and_delays() {
    let Some(mock) = start(&["--big", "--latency-ms", "150"]) else {
        eprintln!("python not available, skipping");
        return;
    };
    let port = mock.1;
    let (head, body) = request(
        port,
        "GET",
        "/api/v4/projects/1001/merge_requests?per_page=50&page=2",
    );
    let head = head.to_ascii_lowercase();
    assert!(head.contains("x-page: 2"), "{head}");
    assert!(head.contains("x-next-page: 3"), "{head}");
    assert!(head.contains("x-total-pages: 3"), "{head}");
    assert!(head.contains("x-total: 117"), "{head}");
    assert!(head.contains("rel=\"next\""), "{head}");
    let items: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(items.as_array().unwrap().len(), 50);

    let start = Instant::now();
    request(port, "GET", "/api/v4/projects?per_page=100");
    assert!(start.elapsed() >= Duration::from_millis(140));

    let (_, stats) = request(port, "GET", "/__stats");
    let stats: serde_json::Value = serde_json::from_str(&stats).unwrap();
    assert_eq!(stats["total"], 2);
    assert_eq!(
        stats["paths"]["GET /projects/:id/merge_requests"]["count"],
        1
    );
    request(port, "POST", "/__reset");
    let (_, stats) = request(port, "GET", "/__stats");
    let stats: serde_json::Value = serde_json::from_str(&stats).unwrap();
    assert_eq!(stats["total"], 0);
}

#[test]
fn mock_big_world_totals() {
    let Some(mock) = start(&["--big"]) else {
        return;
    };
    let (head, _) = request(
        mock.1,
        "GET",
        "/api/v4/projects?per_page=100&membership=true",
    );
    assert!(head.to_ascii_lowercase().contains("x-total: 40"), "{head}");
}

#[test]
fn mock_ignores_non_numeric_paging_params() {
    let Some(mock) = start(&["--big"]) else {
        return;
    };
    let (head, body) = request(mock.1, "GET", "/api/v4/projects?per_page=abc&page=-");
    assert!(head.starts_with("HTTP/1.0 200"), "{head}");
    assert!(head.to_ascii_lowercase().contains("x-page: 1"), "{head}");
    assert!(serde_json::from_str::<serde_json::Value>(&body).is_ok());
}
