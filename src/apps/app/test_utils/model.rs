pub mod req {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct PutLru2CacheReq {
        pub key: String,
        pub value: u8,
    }
    #[derive(Deserialize, Serialize)]
    pub struct GetLru2CacheReq {
        pub key: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct GetRedisReq {
        pub key: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct PutRedisReq {
        pub key: String,
        pub value: String,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct PutMongoDBReq {
        pub key: String,
        pub value: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct FindMongoDBReq {
        pub key: String,
    }
}

pub mod res {
    use serde::{Deserialize, Serialize};
    #[derive(Deserialize, Serialize)]
    pub struct GetLru2CacheRes {
        pub key: String,
        pub value: u8,
    }
    #[derive(Deserialize, Serialize)]
    pub struct GetRedisRes {
        pub key: String,
        pub value: String,
    }
    #[derive(Deserialize, Serialize)]
    pub struct FindMongoDBRes {
        pub key: String,
        pub value: String,
    }
}
