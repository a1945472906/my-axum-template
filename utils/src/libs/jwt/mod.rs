use super::global::get_global_env;
use super::rc::CancerCell;
use super::response::{self, ErrCode, Meta};
use crate::libs::extension::CacheValue;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts, TypedHeader},
};

use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::time::SystemTime;
pub trait Exp {
    fn get_exp(&self) -> u64;
    fn set_exp(&mut self, exp: u64);
}

#[allow(unused)]
#[derive(Debug)]
pub enum AuthError {
    TokenCreation,
    InvalidToken,
}

pub struct Keys {
    // env_key: &'static str,
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(env_key: &'static str) -> Self {
        let secret = get_global_env().get(env_key);
        match secret {
            Some(s) => Self {
                // env_key,
                encoding: EncodingKey::from_secret(s.as_bytes()),
                decoding: DecodingKey::from_secret(s.as_bytes()),
            },
            None => Self {
                // env_key,
                encoding: EncodingKey::from_secret("default_secret".as_bytes()),
                decoding: DecodingKey::from_secret("default_secret".as_bytes()),
            },
        }
    }
}

static KEY: Lazy<CancerCell<Keys>> = Lazy::new(|| CancerCell::new(Keys::new("JWT_SECRET")));

static KEY_FRESH: Lazy<CancerCell<Keys>> =
    Lazy::new(|| CancerCell::new(Keys::new("JWT_FRESH_SECRET")));

pub fn authorize<T>(t: &mut T, expire: u64) -> Result<(String, String), AuthError>
where
    T: Serialize + Exp,
{
    // let expire_refresh = expire + 24 * 3600;
    let expire_time = (SystemTime::now())
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + expire;
    t.set_exp(expire_time);
    let token = encode(&Header::default(), &t, &KEY.get().encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    let refresh_token = encode(&Header::default(), &t, &KEY_FRESH.get().encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok((token, refresh_token))
}

// pub fn decode_token<T>(token: &str) -> Result<T, ()>
// where
//     for<'a> T: Deserialize<'a>,
// {
//     let token_data = decode::<T>(token, &KEY.get().decoding, &Validation::default())
//         .map_err(|_| AuthError::InvalidToken);
//     match token_data {
//         Ok(token) => Ok(token.claims),
//         Err(_) => Err(()),
//     }
// }

#[allow(unused)]
pub fn get_key() -> &'static mut Keys {
    KEY.get_mut()
}

#[allow(unused)]
pub fn get_refresh_key() -> &'static mut Keys {
    KEY_FRESH.get_mut()
}

// pub fn init<F>(f: F)
// where
//     F: FnOnce() -> (),
// {
//     f()
// }

#[derive(Serialize, Deserialize, Clone)]
pub struct Token<T> {
    pub info: T,
    pub exp: u64,
}
unsafe impl<T> Send for Token<T> {}
unsafe impl<T> Sync for Token<T> {}
impl<T> Token<T>
where
    for<'a> T: Deserialize<'a> + Serialize,
    // for<'a> Token<T>: Deserialize<'a> + Serialize,
{
    pub fn new(info: T) -> Token<T> {
        Token { info, exp: 0 }
    }
}
impl<T> Exp for Token<T>
where
    for<'a> T: Deserialize<'a> + Serialize,
{
    fn get_exp(&self) -> u64 {
        self.exp
    }
    fn set_exp(&mut self, exp: u64) {
        self.exp = exp
    }
}
impl<T> CacheValue for Token<T>
where
    for<'a> T: Deserialize<'a> + Serialize,
    // for<'a> Token<T>: Deserialize<'a> + Serialize,
{
    fn is_expire(&self) -> bool {
        (SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            > self.exp
    }
}

impl<T> Deref for Token<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
impl<T> DerefMut for Token<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

#[async_trait]
impl<B, T> FromRequest<B> for Token<T>
where
    for<'a> T: Deserialize<'a> + Serialize,
    B: Send,
{
    type Rejection = response::Response<()>;
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| response::Response {
                    meta: Meta {
                        err_code: ErrCode::UnAuthorized,
                        err_message: String::from("Invalid token"),
                    },
                    body: None,
                })?;
        let token_data =
            decode::<Token<T>>(bearer.token(), &KEY.get().decoding, &Validation::default())
                .map_err(|_| response::Response {
                    meta: Meta {
                        err_code: ErrCode::UnAuthorized,
                        err_message: String::from("Invalid token"),
                    },
                    body: None,
                })?;
        Ok(token_data.claims)
    }
}

pub fn decode_from_value<T>(value: String) -> Result<T, ()>
where
    for<'a> T: Deserialize<'a> + Serialize,
{
    match decode::<T>(&value, &KEY.get().decoding, &Validation::default()) {
        Ok(data) => Ok(data.claims),
        Err(e) => {
            // println!({}, e.to_string());
            Err(())
        }
    }
}
