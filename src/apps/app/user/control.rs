use super::model::*;
use utils::libs::{
    db::{database::DB, model::db_mark::*},
    extension::Cache,
    jwt::{authorize, decode_from_value, Exp, Token},
    response::{ErrCode, Meta},
};
const EXPIRE: u64 = 3600 * 24 * 30;
const REFRESH_EXPIRE: u64 = 24 * 3600;
const LOGIN_KEY_EXP: u64 = 24 * 3600;
use crate::RefreshTokenCache;
use serde_json::{json, Value};
// pub type RefreshTokenCache = Cache<String, Token<UserInfo>>;

pub fn generate_login_key(username: String) -> Result<String, ()> {
    let mut login_key = model::LoginKey::new(username);
    match authorize(&mut login_key, LOGIN_KEY_EXP) {
        Ok(key) => Ok(key.0),
        Err(_) => Err(()),
    }
}
pub fn decode_login_key(key: String) -> String {
    let user_data = decode_from_value::<model::LoginKey>(key).unwrap();
    user_data.username
}

pub async fn login(
    req: req::Login,
    db: DB<User>,
    mut cache: RefreshTokenCache,
) -> Result<resp::LoginInfo, Meta> {
    let result: Result<Value, _> = sqlx::query_scalar(
        "select json_build_object(
    'user_id',user_id, 
    'username', username, 
    'password', password, 
    'realname', realname, 
    'user_desc',user_desc, 
    'avatar', avatar, 
    'roles', roles, 
    'sex',sex)
    from users_table 
    where username=$1 and password=$2",
    )
    .bind(&req.username)
    .bind(&req.password)
    .fetch_one(&db.pool)
    .await;
    match result {
        Ok(value) => match serde_json::from_value::<model::UserInfo>(value) {
            Ok(user_info) => {
                let mut token_info = Token::new(user_info);
                match authorize(&mut token_info, EXPIRE) {
                    Ok((token, refresh_token)) => {
                        token_info.set_exp(token_info.get_exp() + REFRESH_EXPIRE);
                        cache.insert(refresh_token.clone(), token_info.clone());
                        let resp = resp::LoginInfo {
                            info: token_info.info,
                            token,
                            refresh_token,
                        };
                        return Ok(resp);
                    }
                    Err(_) => {
                        return Err(Meta {
                            err_code: ErrCode::InternalServerError,
                            err_message: "服务器错误,请联系管理员!".to_string(),
                        });
                    }
                };
            }
            Err(e) => {
                return Err(Meta {
                    err_code: ErrCode::BadRequest,
                    err_message: e.to_string(),
                })
            }
        },
        Err(e) => {
            println!("{}", e.to_string());
            return Err(Meta {
                err_code: ErrCode::BadRequest,
                err_message: "用户名或密码错误!".to_string(),
            });
        }
    }
}

pub async fn refresh_token(
    req: req::RefreshToken,
    mut cache: RefreshTokenCache,
) -> Result<resp::LoginInfo, Meta> {
    match cache.remove(&req.refresh_token) {
        Some(mut token_info) => {
            match authorize(&mut token_info, EXPIRE) {
                Ok((token, refresh_token)) => {
                    token_info.set_exp(token_info.get_exp() + REFRESH_EXPIRE);
                    cache.insert(refresh_token.clone(), token_info.clone());
                    let resp = resp::LoginInfo {
                        info: token_info.info,
                        token,
                        refresh_token,
                    };
                    return Ok(resp);
                }
                Err(_) => {
                    return Err(Meta {
                        err_code: ErrCode::InternalServerError,
                        err_message: "服务器错误,请联系管理员!".to_string(),
                    });
                }
            };
        }
        None => {
            return Err(Meta {
                err_code: ErrCode::BadRequest,
                err_message: String::from("refresh token不存在或已过期!"),
            })
        }
    }
}

pub async fn add_role(
    user_info: Token<model::UserInfo>,
    req: req::AddRole,
    db: DB<User>,
) -> Result<(), Meta> {
    if !user_info.is_admin() {
        return Err(Meta {
            err_code: ErrCode::UnAuthorized,
            err_message: String::from("权限不足!"),
        });
    }
    match sqlx::query("update users_table set roles=roles||$1::json where username=$2")
        .bind(json!(req.role_info))
        .bind(req.username)
        .execute(&db.pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Meta {
            err_code: ErrCode::BadRequest,
            err_message: e.to_string(),
        }),
    }
}

pub async fn update_user_info(
    mut user_info: Token<model::UserInfo>,
    req: req::UpdateUserInfo,
    db: DB<User>,
    mut cache: RefreshTokenCache,
) -> Result<resp::LoginInfo, Meta> {
    if req.sex.is_some() || req.realname.is_some() {
        let realname = req.realname.unwrap_or(user_info.info.realname.clone());
        let sex = req.sex.unwrap();
        match sqlx::query("update users_table set realname=$1 and sex=$2 where username=$3")
            .bind(&realname)
            .bind(&sex)
            .bind(&user_info.info.username)
            .execute(&db.pool)
            .await
        {
            Ok(_) => {
                user_info.info.realname = realname;
                user_info.info.sex = Some(sex);
                match authorize(&mut user_info, EXPIRE) {
                    Ok((token, refresh_token)) => {
                        user_info.set_exp(user_info.get_exp() + REFRESH_EXPIRE);
                        cache.insert(refresh_token.clone(), user_info.clone());
                        let resp = resp::LoginInfo {
                            info: user_info.info,
                            token,
                            refresh_token,
                        };
                        return Ok(resp);
                    }
                    Err(_) => {
                        return Err(Meta {
                            err_code: ErrCode::InternalServerError,
                            err_message: "服务器错误,请联系管理员!".to_string(),
                        });
                    }
                };
            }
            Err(e) => {
                return Err(Meta {
                    err_code: ErrCode::BadRequest,
                    err_message: e.to_string(),
                })
            }
        }
    } else {
        return Err(Meta {
            err_code: ErrCode::BadRequest,
            err_message: String::from("昵称和性别为空!"),
        });
    }
}

pub async fn get_user(
    user_info: Token<model::UserInfo>,
    db: DB<User>,
) -> Result<resp::GetUsers, Meta> {
    if !user_info.is_admin() {
        return Err(Meta::from(401, "权限不足"));
    }
    let result: Result<serde_json::Value, _> = sqlx::query_scalar(
        "select json_build_object(
                'users',array_agg(json_build_object(
                    'user_id', user_id,
                    'username', username,
                    'realname', realname,
                    'user_desc', user_desc,
                    'avatar', avatar,
                    'sex',sex,
                    'roles', roles
                ))
                )from users_table",
    )
    .fetch_one(&db.pool)
    .await;
    match result {
        Ok(value) => {
            match serde_json::from_value::<model::Users>(value) {
                Ok(users) => {
                    let mut users = users.users;
                    let _ = users.retain(|user| user.is_admin());
                    let customers = users
                        .drain_filter(|user| user.is_customer())
                        .collect::<Vec<model::UserInfo>>();
                    let others = users;

                    match user_info.is_admin() {
                        true => Ok(resp::GetUsers {
                            customers: Some(customers),
                            others: Some(others),
                        }),

                        _ => Err(Meta::from(401, "权限不足")),
                    }
                    // return Ok(());
                }
                Err(e) => Err(Meta::from(500, &e.to_string())),
            }
        }
        Err(e) => Err(Meta::from(500, &e.to_string())),
    }
    // Ok(())
}

pub async fn key_login(
    req: req::KeyLogin,
    db: DB<User>,
    mut cache: RefreshTokenCache,
) -> Result<resp::LoginInfo, Meta> {
    let username = decode_login_key(req.key);
    let result: Result<Value, _> = sqlx::query_scalar(
        "select json_build_object(
    'user_id',user_id, 
    'username', username, 
    'password', password, 
    'realname', realname, 
    'user_desc',user_desc, 
    'avatar', avatar, 
    'roles', roles, 
    'sex',sex)
    from users_table 
    where username=$1",
    )
    .bind(username)
    .fetch_one(&db.pool)
    .await;
    match result {
        Ok(value) => match serde_json::from_value::<model::UserInfo>(value) {
            Ok(user_info) => {
                let mut token_info = Token::new(user_info);
                match authorize(&mut token_info, EXPIRE) {
                    Ok((token, refresh_token)) => {
                        token_info.set_exp(token_info.get_exp() + REFRESH_EXPIRE);
                        cache.insert(refresh_token.clone(), token_info.clone());
                        let resp = resp::LoginInfo {
                            info: token_info.info,
                            token,
                            refresh_token,
                        };
                        return Ok(resp);
                    }
                    Err(_) => {
                        return Err(Meta {
                            err_code: ErrCode::InternalServerError,
                            err_message: "服务器错误,请联系管理员!".to_string(),
                        });
                    }
                };
            }
            Err(e) => {
                return Err(Meta {
                    err_code: ErrCode::BadRequest,
                    err_message: e.to_string(),
                })
            }
        },
        Err(e) => {
            println!("{}", e.to_string());
            return Err(Meta {
                err_code: ErrCode::BadRequest,
                err_message: "用户名或密码错误!".to_string(),
            });
        }
    }
}
