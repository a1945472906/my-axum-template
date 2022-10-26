pub mod model {
    use serde::{Deserialize, Serialize};

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
    #[derive(Deserialize, Serialize, Clone)]
    pub struct RoleInfo {
        pub role_name: String,
        pub value: String,
    }
}

pub mod req {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Login {
        pub username: String,
        pub password: String,
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
}
