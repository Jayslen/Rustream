use std::fs::{File, metadata};
use std::io::{Read, Seek, SeekFrom};

use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing,
};
use std::os::unix::fs::MetadataExt;
use std::str::FromStr;

#[derive(Debug)]
enum RequestRange {
    Range(u64, u64, u64),
    Invalid,
}

const FILE_PATH: &str = "/home/jayslen/Videos/video-faststart.mp4";
#[tokio::main]
async fn main() {
    let app = Router::new().route("/video/{id}", routing::get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Path(id): Path<String>, req_header: HeaderMap) -> impl IntoResponse {
    let file_metadata = metadata(FILE_PATH).unwrap();
    let mut f = File::open(FILE_PATH).unwrap();

    let mut res_header = HeaderMap::new();
    res_header.insert("Content-Type", HeaderValue::from_static("video/mp4"));
    res_header.insert("Accept-Ranges", HeaderValue::from_static("bytes"));

    let req_range = parse_range(req_header.get("Range"), file_metadata.size());

    println!("Request Range: {:?}", req_range);

    let mut buffer: Vec<u8> = Vec::new();

    match req_range {
        RequestRange::Range(from, to, length) => {
            println!("From: {}, To: {}, Length: {}", from, to, length);
            buffer = vec![0u8; length as usize];

            f.seek(SeekFrom::Start(from)).unwrap();
            f.read_exact(&mut buffer).unwrap();

            res_header.insert(
                "Content-Range",
                HeaderValue::from_str(&format!("bytes {}-{}/{}", from, to, file_metadata.size()))
                    .unwrap(),
            );

            res_header.insert(
                "Content-Length",
                HeaderValue::from_str(&format!("{}", length)).unwrap(),
            );

            return (StatusCode::PARTIAL_CONTENT, res_header, buffer);
        }
        RequestRange::Invalid => {
            return (StatusCode::RANGE_NOT_SATISFIABLE, res_header, buffer);
        }
    }
}

fn parse_range(raw_range: Option<&HeaderValue>, req_file_size: u64) -> RequestRange {
    match raw_range {
        Some(value) => {
            let mut range_parse: Vec<i64> =
                value.to_str().unwrap().split("=").collect::<Vec<&str>>()[1]
                    .split("-")
                    .map(|f| {
                        return match f64::from_str(f) {
                            Ok(v) => v as i64,
                            Err(_) => {
                                // -1 is for empty string, -2 is for invalid range
                                if f.trim().is_empty() { -1 } else { -2 }
                            }
                        };
                    })
                    .collect();

            if range_parse.contains(&-2) {
                return RequestRange::Invalid;
            }

            if range_parse[0] < 0 {
                range_parse[0] = req_file_size as i64 - range_parse[1];
                range_parse[1] = req_file_size as i64 - 1;
            }

            if range_parse[1] < 0 {
                range_parse[1] = req_file_size as i64 - 1;
            }

            let content_length = (range_parse[1] - range_parse[0]) + 1;

            RequestRange::Range(
                range_parse[0] as u64,
                range_parse[1] as u64,
                content_length as u64,
            )
        }
        _ => RequestRange::Invalid,
    }
}
