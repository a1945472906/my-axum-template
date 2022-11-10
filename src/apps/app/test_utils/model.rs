pub mod req {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct PutLru2CacheReq {
        pub key: String,
        pub value: u8,
    }
    #[derive(Deserialize, Serialize)]
    pub struct GetLru2CacheReq {
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
}
