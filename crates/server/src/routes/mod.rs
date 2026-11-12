use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use std::fs::metadata;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
};
use tokio_util::io::ReaderStream;

use std::os::unix::fs::MetadataExt;
use std::str::FromStr;

#[derive(Debug)]
enum RequestRange {
    Range(u64, u64, u64),
    Invalid,
}

pub async fn stream_route(Path(id): Path<String>, req_header: HeaderMap) -> impl IntoResponse {
    const FILE_PATH: &str = "/home/jayslen/Videos/A.Separation.mp4";
    let file_metadata = metadata(FILE_PATH).unwrap();

    let mut f = File::open(FILE_PATH).await.unwrap();

    let mut res_header = HeaderMap::new();
    res_header.insert("Content-Type", HeaderValue::from_static("video/mp4"));
    res_header.insert("Accept-Ranges", HeaderValue::from_static("bytes"));

    let req_range = parse_range(req_header.get("Range"), file_metadata.size());

    match req_range {
        RequestRange::Range(from, to, length) => {
            let _ = &mut f.seek(SeekFrom::Start(from)).await;
            let limit_reader = f.take(length);
            let stream = ReaderStream::new(limit_reader);

            res_header.insert(
                "Content-Range",
                HeaderValue::from_str(&format!("bytes {}-{}/{}", from, to, file_metadata.size()))
                    .unwrap(),
            );

            res_header.insert(
                "Content-Length",
                HeaderValue::from_str(&format!("{}", length)).unwrap(),
            );

            return (
                StatusCode::PARTIAL_CONTENT,
                res_header,
                Body::from_stream(stream),
            );
        }
        RequestRange::Invalid => {
            return (StatusCode::RANGE_NOT_SATISFIABLE, res_header, Body::empty());
        }
    }
}

fn parse_range(raw_range: Option<&HeaderValue>, req_file_size: u64) -> RequestRange {
    match raw_range {
        Some(value) => {
            if value.len() == 0 {
                return RequestRange::Invalid;
            }

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    const FILE_SIZE: u64 = 1000;

    #[test]
    fn empty_range() {
        let range = HeaderValue::from_static("");

        let response = parse_range(Some(&range), FILE_SIZE);

        assert!(matches!(response, RequestRange::Invalid));
    }

    #[test]
    fn explicit_range() {
        let range = HeaderValue::from_static("bytes=200-300");

        let response = parse_range(Some(&range), FILE_SIZE);

        assert!(matches!(response, RequestRange::Range(200, 300, 101)));
    }

    #[test]
    fn open_ended_range() {
        let range = HeaderValue::from_static("bytes=200-");

        let response = parse_range(Some(&range), FILE_SIZE);

        assert!(matches!(response, RequestRange::Range(200, 999, 800)));
    }

    #[test]
    fn suffix_range() {
        let range = HeaderValue::from_static("bytes=-500");

        let response = parse_range(Some(&range), FILE_SIZE);

        assert!(matches!(response, RequestRange::Range(500, 999, 500)));
    }

    #[test]
    fn first_byte_range() {
        let range = HeaderValue::from_static("bytes=0-0");

        let response = parse_range(Some(&range), FILE_SIZE);

        assert!(matches!(response, RequestRange::Range(0, 0, 1)));
    }

    #[test]
    fn last_byte_range() {
        let range = HeaderValue::from_static("bytes=999-999");

        let response = parse_range(Some(&range), FILE_SIZE);

        assert!(matches!(response, RequestRange::Range(999, 999, 1)));
    }

    #[test]
    fn range_without_header() {
        let response = parse_range(None, FILE_SIZE);

        assert!(matches!(response, RequestRange::Invalid));
    }
}
