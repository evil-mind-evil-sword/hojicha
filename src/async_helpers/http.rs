//! HTTP request helper commands

use crate::core::{Cmd, Message};
use crate::commands;
use super::{AsyncConfig, AsyncResult};
use std::collections::HashMap;

/// HTTP methods
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// DELETE request
    Delete,
    /// PATCH request
    Patch,
    /// HEAD request
    Head,
}

/// HTTP request error
#[derive(Debug, Clone)]
pub enum HttpError {
    /// Network error
    NetworkError(String),
    /// Timeout
    Timeout,
    /// Invalid URL
    InvalidUrl(String),
    /// Server error (status code)
    ServerError(u16, String),
    /// Parse error
    ParseError(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::NetworkError(e) => write!(f, "Network error: {}", e),
            HttpError::Timeout => write!(f, "Request timed out"),
            HttpError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            HttpError::ServerError(code, msg) => write!(f, "Server error {}: {}", code, msg),
            HttpError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for HttpError {}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as string
    pub body: String,
    /// Response body as bytes
    pub bytes: Vec<u8>,
}

/// Create a GET request command
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::http_get;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     DataLoaded(String),
/// #     Error(String),
/// # }
/// # impl hojicha_core::Message for Msg {}
/// 
/// http_get("https://api.example.com/data", |result| {
///     match result {
///         Ok(response) => Msg::DataLoaded(response.body),
///         Err(e) => Msg::Error(e.to_string()),
///     }
/// })
/// # ;
/// ```
pub fn http_get<M, F>(url: impl Into<String>, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<HttpResponse, HttpError>) -> M + Send + 'static,
{
    http_request(HttpMethod::Get, url, None::<String>, None, handler)
}

/// Create a POST request command with JSON body
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::http_post;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     Posted,
/// #     Error(String),
/// # }
/// # impl hojicha_core::Message for Msg {}
/// 
/// let json_body = r#"{"name": "test"}"#;
/// http_post("https://api.example.com/data", json_body, |result| {
///     match result {
///         Ok(_) => Msg::Posted,
///         Err(e) => Msg::Error(e.to_string()),
///     }
/// })
/// # ;
/// ```
pub fn http_post<M, F, B>(url: impl Into<String>, body: B, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<HttpResponse, HttpError>) -> M + Send + 'static,
    B: Into<String>,
{
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    http_request(HttpMethod::Post, url, Some(body), Some(headers), handler)
}

/// Create a custom HTTP request command
///
/// This is the most flexible HTTP helper, allowing you to specify method,
/// headers, and body.
pub fn http_request<M, F, B>(
    method: HttpMethod,
    url: impl Into<String>,
    body: Option<B>,
    headers: Option<HashMap<String, String>>,
    handler: F,
) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<HttpResponse, HttpError>) -> M + Send + 'static,
    B: Into<String>,
{
    let url = url.into();
    let body = body.map(|b| b.into());
    
    commands::spawn(async move {
        // In a real implementation, we would use reqwest or similar
        // For now, we'll create a mock implementation
        let result = perform_http_request(method, url, body, headers).await;
        Some(handler(result))
    })
}

/// Internal function to perform HTTP request
/// In a real implementation, this would use reqwest or similar
async fn perform_http_request(
    method: HttpMethod,
    url: String,
    body: Option<String>,
    headers: Option<HashMap<String, String>>,
) -> Result<HttpResponse, HttpError> {
    // This is a placeholder implementation
    // In production, you would use reqwest or another HTTP client
    
    // Simulate network delay
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // For demonstration, return a mock response
    Ok(HttpResponse {
        status: 200,
        headers: headers.unwrap_or_default(),
        body: body.unwrap_or_else(|| format!("Mock response for {} {}", 
            match method {
                HttpMethod::Get => "GET",
                HttpMethod::Post => "POST",
                HttpMethod::Put => "PUT",
                HttpMethod::Delete => "DELETE",
                HttpMethod::Patch => "PATCH",
                HttpMethod::Head => "HEAD",
            },
            url
        )),
        bytes: Vec::new(),
    })
}

/// Create an HTTP request with retry logic
pub fn http_with_retry<M, F>(
    method: HttpMethod,
    url: impl Into<String>,
    config: AsyncConfig,
    handler: F,
) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<HttpResponse, HttpError>) -> M + Send + 'static,
{
    let url = url.into();
    
    commands::spawn(async move {
        let mut attempts = 0;
        loop {
            let result = perform_http_request(method, url.clone(), None, None).await;
            
            if result.is_ok() || attempts >= config.retries {
                return Some(handler(result));
            }
            
            attempts += 1;
            
            // Apply backoff strategy
            match config.backoff {
                super::BackoffStrategy::None => {},
                super::BackoffStrategy::Linear(duration) => {
                    tokio::time::sleep(duration * attempts).await;
                },
                super::BackoffStrategy::Exponential(duration) => {
                    tokio::time::sleep(duration * 2u32.pow(attempts)).await;
                },
            }
        }
    })
}