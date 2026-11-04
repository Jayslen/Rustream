use std::fs::{File, metadata};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::thread::{self, Thread};

// use axum::extract::path;
use axum::http::{Request, header};
use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing,
};

#[derive(Debug)]
enum RequestRanges {
    Range(u64, u64),
    From(u64),
    Last(u64),
    None,
}

const FILE_PATH: &str = "/home/jayslen/Videos/Naruto.mp4";
#[tokio::main]
async fn main() {
    let app = Router::new().route("/video/{id}", routing::get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Path(id): Path<String>, header: HeaderMap) -> impl IntoResponse {
    let file_metadata = metadata(FILE_PATH).unwrap();
    let mut f = File::open(FILE_PATH).unwrap();

    let mut response_header = HeaderMap::new();
    response_header.insert("Content-Type", HeaderValue::from_static("video/mp4"));
    response_header.insert("Accept-Ranges", HeaderValue::from_static("bytes"));

    let range = parse_range(header.get("Range"));
    let buff_size: u64;
    let mut buffer: Vec<u8> = Vec::new();

    match range {
        RequestRanges::Range(from, to) => {
            buff_size = to - from;
            buffer = vec![0u8; buff_size as usize];
            f.seek(SeekFrom::Start(from)).unwrap();
            f.read_exact(&mut buffer).unwrap();
            response_header.insert(
                "Content-Range",
                HeaderValue::from_str(&format!("bytes {}-{}/{}", from, to, file_metadata.size()))
                    .unwrap(),
            );

            response_header.insert(
                "Content-Length",
                HeaderValue::from_str(&format!("{}", buff_size)).unwrap(),
            );

            return (StatusCode::PARTIAL_CONTENT, response_header, buffer);
        }
        RequestRanges::From(from) => {
            buff_size = file_metadata.size() - from;
            buffer = vec![0u8; buff_size as usize];
            f.seek(SeekFrom::Start(from)).unwrap();
            f.read_exact(&mut buffer).unwrap();
            response_header.insert(
                "Content-Range",
                HeaderValue::from_str(&format!("bytes {}-/{}", from, file_metadata.size()))
                    .unwrap(),
            );
            response_header.insert(
                "Content-Length",
                HeaderValue::from_str(&format!("{}", buff_size)).unwrap(),
            );

            return (StatusCode::PARTIAL_CONTENT, response_header, buffer);
        }
        RequestRanges::Last(to) => {
            buff_size = to;

            buffer = vec![0u8; buff_size as usize];
            f.seek(SeekFrom::End(-(to as i64))).unwrap();

            response_header.insert(
                "Content-Range",
                HeaderValue::from_str(&format!("bytes -{}/{}", to, file_metadata.size())).unwrap(),
            );
            println!("Range last {}", to);

            response_header.insert(
                "Content-Length",
                HeaderValue::from_str(&format!("{}", buff_size)).unwrap(),
            );

            f.read_exact(&mut buffer).unwrap();

            return (StatusCode::PARTIAL_CONTENT, response_header, buffer);
        }
        RequestRanges::None => {
            f.read_to_end(&mut buffer).unwrap();

            response_header.insert(
                "Content-Length",
                HeaderValue::from_str(&format!("{}", file_metadata.size())).unwrap(),
            );
            return (StatusCode::OK, response_header, buffer);
        }
    }
}

fn parse_range(raw_range: Option<&HeaderValue>) -> RequestRanges {
    match raw_range {
        Some(value) => {
            let range: Vec<&str> = (value.to_str().unwrap().split("=").collect::<Vec<&str>>()[0])
                .split("bytes")
                .collect();

            if range[0].len() > 0 && range[1].len() > 0 {
                let from = range[0].parse::<u64>().unwrap();
                let to = range[1].parse::<u64>().unwrap();
                return RequestRanges::Range(from, to);
            } else if range[0].len() > 0 {
                let from = range[0].parse::<u64>().unwrap();
                return RequestRanges::From(from);
            } else if range[1].len() > 0 {
                let to = range[1].parse::<u64>().unwrap();
                return RequestRanges::Last(to);
            } else {
                return RequestRanges::None;
            }
        }
        None => RequestRanges::None,
    }
}
