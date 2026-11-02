use std::path::{Path, PathBuf};

use reqwest::{Method, StatusCode, header};

use crate::error::RequestError;
use crate::service::RequestClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtagPolicy {
    CommitNow,
    /// Hands the ETag back on [`Fetched::etag`]
    /// Use when work downstream of the body can still fail so an interrupted run
    /// retries instead of getting 304
    Defer,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn fetch_fixture(
        response: &'static [u8],
        stall: bool,
        existing: bool,
        policy: EtagPolicy,
    ) -> (Option<Fetched>, Option<Vec<u8>>, Option<Vec<u8>>) {
        let root =
            std::env::temp_dir().join(format!("oneclient-cache-test-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&root).await.unwrap();
        let path = root.join("metadata.json");
        if existing {
            tokio::fs::write(&path, b"old metadata").await.unwrap();
            tokio::fs::write(etag_path(&path), b"old-etag")
                .await
                .unwrap();
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (accepted, received) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 4096];
            let size = stream.read(&mut request).await.unwrap();
            let _ = accepted.send(String::from_utf8_lossy(&request[..size]).into_owned());
            stream.write_all(response).await.unwrap();
            if stall {
                std::future::pending::<()>().await;
            }
        });
        let client = RequestClient::new(crate::config::NetConfig {
            meta_url_base: url.clone(),
            metadata_api_url: url.clone(),
            ..Default::default()
        })
        .unwrap();
        let started = Instant::now();
        let result =
            fetch_cached_with_budget(&client, &url, &path, policy, Duration::from_millis(300))
                .await
                .unwrap();
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "stalled metadata must not use the long download timeout"
        );
        let request = received.await.unwrap();
        if existing {
            assert!(request.to_lowercase().contains("if-none-match: old-etag"));
        }
        server.abort();
        let cached = tokio::fs::read(&path).await.ok();
        let etag = tokio::fs::read(etag_path(&path)).await.ok();
        if cached.is_some() {
            tokio::fs::remove_file(&path).await.unwrap();
        }
        if etag.is_some() {
            tokio::fs::remove_file(etag_path(&path)).await.unwrap();
        }
        tokio::fs::remove_dir(root).await.unwrap();
        (result, cached, etag)
    }

    #[tokio::test]
    async fn stalled_headers_fall_back_to_existing_cache() {
        let (fetched, cached, etag) = fetch_fixture(b"", true, true, EtagPolicy::CommitNow).await;
        let fetched = fetched.unwrap();
        assert!(!fetched.changed);
        assert_eq!(fetched.bytes, b"old metadata");
        assert_eq!(cached.unwrap(), b"old metadata");
        assert_eq!(etag.unwrap(), b"old-etag");
    }

    #[tokio::test]
    async fn stalled_body_preserves_cache_and_etag() {
        let (fetched, cached, etag) = fetch_fixture(
            b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nETag: new-etag\r\n\r\npartial",
            true,
            true,
            EtagPolicy::CommitNow,
        )
        .await;
        assert_eq!(fetched.unwrap().bytes, b"old metadata");
        assert_eq!(cached.unwrap(), b"old metadata");
        assert_eq!(etag.unwrap(), b"old-etag");
    }

    #[tokio::test]
    async fn stalled_first_run_returns_unavailable_without_creating_cache() {
        let (fetched, cached, etag) = fetch_fixture(b"", true, false, EtagPolicy::CommitNow).await;
        assert!(fetched.is_none() && cached.is_none() && etag.is_none());
    }

    #[tokio::test]
    async fn broken_body_uses_previous_cache() {
        let (fetched, cached, _) = fetch_fixture(
            b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\npartial",
            false,
            true,
            EtagPolicy::CommitNow,
        )
        .await;
        assert_eq!(fetched.unwrap().bytes, b"old metadata");
        assert_eq!(cached.unwrap(), b"old metadata");
    }

    #[tokio::test]
    async fn successful_fetch_keeps_commit_and_deferred_etag_behavior() {
        for policy in [EtagPolicy::CommitNow, EtagPolicy::Defer] {
            let (fetched, cached, etag) = fetch_fixture(
                b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nETag: new-etag\r\n\r\nnew",
                false,
                true,
                policy,
            )
            .await;
            let fetched = fetched.unwrap();
            assert!(fetched.changed);
            assert_eq!(fetched.bytes, b"new");
            assert_eq!(cached.unwrap(), b"new");
            match policy {
                EtagPolicy::CommitNow => {
                    assert_eq!(etag.unwrap(), b"new-etag");
                    assert!(fetched.etag.is_none());
                }
                EtagPolicy::Defer => {
                    assert_eq!(etag.unwrap(), b"old-etag");
                    assert_eq!(fetched.etag.as_deref(), Some("new-etag"));
                }
            }
        }
    }

    #[tokio::test]
    async fn unchanged_and_failed_responses_keep_cached_data() {
        for response in [
            &b"HTTP/1.1 304 Not Modified\r\n\r\n"[..],
            &b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\n\r\n"[..],
        ] {
            let (fetched, cached, _) =
                fetch_fixture(response, false, true, EtagPolicy::CommitNow).await;
            assert!(!fetched.as_ref().unwrap().changed);
            assert_eq!(fetched.unwrap().bytes, b"old metadata");
            assert_eq!(cached.unwrap(), b"old metadata");
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fetched {
    pub bytes: Vec<u8>,
    /// `false` when the server answered 304 or the request failed and the
    /// cached copy was served instead
    pub changed: bool,
    /// Present only for a fresh body under [`EtagPolicy::Defer`]
    pub etag: Option<String>,
}

impl Fetched {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, RequestError> {
        serde_json::from_slice(&self.bytes).map_err(|source| RequestError::DeserializeError {
            source,
            type_name: std::any::type_name::<T>().to_string(),
            url: String::new(),
            status: 200,
            snippet: crate::error::body_snippet(&self.bytes),
        })
    }

    /// Lossy a bad byte becomes a replacement character rather than an error
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

fn etag_path(cache_path: &Path) -> PathBuf {
    let mut path = cache_path.as_os_str().to_os_string();
    path.push(".etag");
    PathBuf::from(path)
}

async fn read_sidecar_etag(cache_path: &Path) -> Option<String> {
    polyio::read(etag_path(cache_path))
        .await
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .filter(|etag| !etag.is_empty())
}

/// Callers using [`EtagPolicy::Defer`] call this once the work depending on the
/// body has succeeded
pub async fn commit_etag(cache_path: &Path, etag: &str) {
    if let Err(err) = polyio::write_atomic(etag_path(cache_path), etag.as_bytes()).await {
        tracing::warn!("failed to write etag sidecar: {err}");
    }
}

/// Returns `Ok(None)` only when the request failed *and* there is no cached
/// copy
#[tracing::instrument(level = "debug", skip(client), fields(%url))]
pub async fn fetch_cached(
    client: &RequestClient,
    url: &str,
    cache_path: &Path,
    policy: EtagPolicy,
) -> Result<Option<Fetched>, RequestError> {
    fetch_cached_with_budget(
        client,
        url,
        cache_path,
        policy,
        std::time::Duration::from_secs(12),
    )
    .await
}

// Optional metadata and artwork must fall back to disk within a predictable
// time, including when headers arrive but the response body stops arriving.
// Game/library downloads use a separate path and retain their longer budgets.
async fn fetch_cached_with_budget(
    client: &RequestClient,
    url: &str,
    cache_path: &Path,
    policy: EtagPolicy,
    budget: std::time::Duration,
) -> Result<Option<Fetched>, RequestError> {
    let stored_etag = read_sidecar_etag(cache_path).await;
    let mut request = reqwest::Request::new(Method::GET, url.parse()?);

    if let Some(etag) = &stored_etag
        && let Ok(value) = header::HeaderValue::from_str(etag)
    {
        request.headers_mut().insert(header::IF_NONE_MATCH, value);
    }

    let cached = || async {
        match polyio::read(cache_path).await {
            Ok(bytes) => Ok(Some(Fetched {
                bytes,
                changed: false,
                etag: None,
            })),
            Err(err) => {
                tracing::debug!("no usable cache at {}: {err}", cache_path.display());
                Ok(None)
            }
        }
    };

    let remote = async {
        match client.send(request).await {
            Ok(res) if res.status() == StatusCode::NOT_MODIFIED => {
                tracing::debug!("cache hit (304)");
                cached().await
            }
            Ok(res) if res.status().is_success() => {
                let server_etag = res
                    .headers()
                    .get(header::ETAG)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_string);

                let bytes = res.bytes().await?.to_vec();
                polyio::write_atomic(cache_path, &bytes).await?;

                let etag = match (&server_etag, policy) {
                    (Some(etag), EtagPolicy::CommitNow) => {
                        commit_etag(cache_path, etag).await;
                        None
                    }
                    (_, EtagPolicy::Defer) => server_etag,
                    (None, EtagPolicy::CommitNow) => None,
                };

                Ok(Some(Fetched {
                    bytes,
                    changed: true,
                    etag,
                }))
            }
            Ok(res) => {
                tracing::warn!(status = %res.status(), "unexpected response; using cache if present");
                cached().await
            }
            Err(err) => {
                tracing::debug!("fetch failed, using cache if present: {err}");
                cached().await
            }
        }
    };
    match tokio::time::timeout(budget, remote).await {
        Ok(Ok(fetched)) => Ok(fetched),
        Ok(Err(err)) => {
            tracing::debug!("metadata body/cache update failed, using cache if present: {err}");
            cached().await
        }
        Err(_) => {
            tracing::warn!("metadata fetch timed out, using cache if present");
            cached().await
        }
    }
}
