//! APIエラー定義

use serde::Serialize;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{Rejection, Reply};

/// APIエラー
#[derive(Debug)]
pub enum ApiError {
    /// 不正なリクエスト
    BadRequest(String),
    /// リソースが見つからない
    NotFound(String),
    /// 処理エラー
    ProcessingError(String),
    /// 内部エラー
    InternalError(String),
    /// 機能が利用不可
    NotImplemented(String),
}

impl Reject for ApiError {}

/// エラーレスポンス
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

/// Rejectionをレスポンスに変換
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, error, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "not_found", "Resource not found")
    } else if let Some(e) = err.find::<ApiError>() {
        match e {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.as_str()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.as_str()),
            ApiError::ProcessingError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "processing_error", msg.as_str())
            }
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg.as_str())
            }
            ApiError::NotImplemented(msg) => {
                (StatusCode::NOT_IMPLEMENTED, "not_implemented", msg.as_str())
            }
        }
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "File too large",
        )
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "Method not allowed",
        )
    } else {
        eprintln!("Unhandled rejection: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Internal server error",
        )
    };

    let json = warp::reply::json(&ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    });

    Ok(warp::reply::with_status(json, code))
}

