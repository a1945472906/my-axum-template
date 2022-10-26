pub mod model {
    use serde::{Deserialize, Serialize};
    use utils::libs::jwt::Exp;
    #[derive(Deserialize, Serialize)]
    pub struct LoginKey {
        pub username: String,
        pub exp: u64,
    }
    impl LoginKey {
        pub fn new(username: String) -> Self {
            LoginKey { username, exp: 0 }
        }
    }
    impl Exp for LoginKey {
        fn get_exp(&self) -> u64 {
            self.exp
        }
        fn set_exp(&mut self, exp: u64) {
            self.exp = exp;
        }
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct UserInfo {
        pub user_id: i64,
        pub username: String,
        pub realname: String,
        pub user_desc: String,
        pub avatar: String,
        pub roles: Vec<RoleInfo>,
        pub sex: Option<bool>,
    }
    impl UserInfo {
        pub fn is_admin(&self) -> bool {
            for RoleInfo { value, .. } in &self.roles {
                if value == "admin" {
                    return true;
                }
            }
            false
        }
        pub fn is_customer(&self) -> bool {
            for RoleInfo { value, .. } in &self.roles {
                if value == "customer" {
                    return true;
                }
            }
            false
        }
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct RoleInfo {
        pub role_name: String,
        pub value: String,
    }
    #[derive(Deserialize, Serialize)]
    pub struct Users {
        pub users: Vec<UserInfo>,
    }
}

pub mod req {
    use super::model::RoleInfo;
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize)]
    pub struct Login {
        pub username: String,
        pub password: String,
    }
    #[derive(Serialize, Deserialize)]
    pub struct RefreshToken {
        pub refresh_token: String,
    }
    #[derive(Serialize, Deserialize)]
    pub struct AddRole {
        pub username: String,
        pub role_info: RoleInfo,
    }

    #[derive(Serialize, Deserialize)]
    pub struct UpdateUserInfo {
        pub realname: Option<String>,
        pub sex: Option<bool>,
        pub user_desc: Option<String>,
    }
    #[derive(Serialize, Deserialize)]
    pub struct KeyLogin {
        pub key: String,
    }
}
pub mod resp {
    use super::model;
    use serde::{Deserialize, Serialize};
    #[derive(Deserialize, Serialize, Clone)]
    pub struct LoginInfo {
        pub info: model::UserInfo,
        pub token: String,
        pub refresh_token: String,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct GetUsers {
        // pub contractors: Option<Vec<model::UserInfo>>,
        pub customers: Option<Vec<model::UserInfo>>,
        pub others: Option<Vec<model::UserInfo>>,
    }
}
