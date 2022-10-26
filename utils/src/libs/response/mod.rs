use axum::response::IntoResponse;
use axum::Json;
use bytes::{BufMut, BytesMut};
use http::{
    header::{self, HeaderValue},
    StatusCode,
};
use serde::Serialize;

#[allow(unused)]
pub enum ErrCode {
    Ok,
    BadRequest,
    UnAuthorized,
    NotFound,
    Forbiden,
    MethodNotAllowed,
    InternalServerError,
}
impl ErrCode {
    pub fn to_u16(&self) -> u16 {
        match self {
            ErrCode::Ok => 200,
            ErrCode::BadRequest => 400,
            ErrCode::UnAuthorized => 401,
            ErrCode::NotFound => 404,
            ErrCode::Forbiden => 403,
            ErrCode::MethodNotAllowed => 405,
            ErrCode::InternalServerError => 500,
        }
    }
}
impl Serialize for ErrCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ErrCode::Ok => serializer.serialize_u32(200),
            ErrCode::BadRequest => serializer.serialize_u32(400),
            ErrCode::UnAuthorized => serializer.serialize_u32(401),
            ErrCode::NotFound => serializer.serialize_u32(404),
            ErrCode::Forbiden => serializer.serialize_u32(403),
            ErrCode::MethodNotAllowed => serializer.serialize_u32(405),
            ErrCode::InternalServerError => serializer.serialize_u32(500),
        }
    }
}
#[derive(Serialize)]
pub struct Meta {
    pub err_code: ErrCode,
    pub err_message: String,
}
impl Meta {
    pub fn from(code: u16, err_message: &str) -> Meta {
        let err_code = match code {
            200 => ErrCode::Ok,
            400 => ErrCode::BadRequest,
            401 => ErrCode::UnAuthorized,
            403 => ErrCode::Forbiden,
            404 => ErrCode::NotFound,
            405 => ErrCode::MethodNotAllowed,
            500 => ErrCode::InternalServerError,
            _ => ErrCode::InternalServerError,
        };
        Meta {
            err_code,
            err_message: String::from(err_message),
        }
    }
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            err_code: ErrCode::Ok,
            err_message: String::new(),
        }
    }
}

#[derive(Serialize)]
pub struct Response<B> {
    pub meta: Meta,
    pub body: Option<B>,
}

impl<B> Response<B> {
    pub fn from(result: Result<B, Meta>) -> Response<B> {
        match result {
            Ok(body) => Response {
                meta: Meta::default(),
                body: Some(body),
            },
            Err(meta) => Response { meta, body: None },
        }
    }
}

impl<'a, B> IntoResponse for Response<B>
where
    B: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let code = self.meta.err_code.to_u16();
        let mut res = Json(self).into_response();
        *res.status_mut() = StatusCode::from_u16(code).unwrap();
        res
    }
}
