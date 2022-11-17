// use super::env::{get, get_obs, WatchEnv};
// use super::observer::{Observable, Observer};
use super::rc::CancerCell;
use super::response::{ErrCode, Meta, Response};
use crate::libs::global::get_global_env;
use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
// use std::sync::Mutex
// use multer::Multipart;
// use http::Method;
use once_cell::sync::Lazy;
// use std::collections::HashMap;
use std::ops::Deref;

static CONTENT_LENGTH_LIMIT: Lazy<CancerCell<u64>> = Lazy::new(|| {
    let target = get_global_env().get("CONTENT_LENGTH_LIMIT").unwrap();
    let content_length_limit: u64 = match target.as_str() {
        "large" => 2048 * 1024 * 1024,  //  2g
        "medium" => 1024 * 1024 * 1024, //  1g
        "small" => 512 * 1024 * 1024,   //  512m
        _ => 0,
    };
    CancerCell::new(content_length_limit)
});

#[derive(Debug, Clone)]
pub struct DynContentLengthLimit<T>(pub T);
#[async_trait]
impl<T, B> FromRequest<B> for DynContentLengthLimit<T>
where
    T: FromRequest<B>,
    T::Rejection: IntoResponse,
    B: Send,
{
    type Rejection = (StatusCode, Response<()>);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let content_length = req
            .headers()
            .get(http::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok()?.parse::<u64>().ok());

        match (content_length, req.method()) {
            (content_length, &(Method::GET | Method::HEAD | Method::OPTIONS)) => {
                if content_length.is_some() {
                    return Err((
                        StatusCode::METHOD_NOT_ALLOWED,
                        Response {
                            meta: Meta {
                                err_code: ErrCode::MethodNotAllowed,
                                err_message: "`GET`, `HEAD`, `OPTIONS` requests are not allowed to have a `Content-Length` header".to_string(),
                            },
                            body: None,
                        },
                    ));
                } else if req
                    .headers()
                    .get(http::header::TRANSFER_ENCODING)
                    .map_or(false, |value| value.as_bytes() == b"chunked")
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Response {
                            meta: Meta {
                                err_code: ErrCode::BadRequest,
                                err_message: "Content length header is required".to_string(),
                            },
                            body: None,
                        },
                    ));
                }
            }
            (Some(content_length), _) if content_length > *CONTENT_LENGTH_LIMIT.get() => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Response {
                        meta: Meta {
                            err_code: ErrCode::BadRequest,
                            err_message: "Request payload is too large".to_string(),
                        },
                        body: None,
                    },
                ));
            }
            (None, _) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Response {
                        meta: Meta {
                            err_code: ErrCode::BadRequest,
                            err_message: "Content length header is required".to_string(),
                        },
                        body: None,
                    },
                ));
            }
            _ => {
                // let n = *CONTENT_LENGTH_LIMIT.get();
                // println!("content_length_limit: {}", n);
            }
        }
        match T::from_request(req).await {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err((
                StatusCode::BAD_REQUEST,
                Response {
                    meta: Meta {
                        err_code: ErrCode::BadRequest,
                        err_message: "Content length header is required".to_string(),
                    },
                    body: None,
                },
            )),
        }
    }
}

impl<T> Deref for DynContentLengthLimit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn get() -> &'static CancerCell<u64> {
    &CONTENT_LENGTH_LIMIT
}

// pub struct MultipartForm {
//     inner: Multipart<'static>,
//     map: HashMap<String, String>,
// }
