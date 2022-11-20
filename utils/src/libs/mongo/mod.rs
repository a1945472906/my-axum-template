pub use mongodb::{self, options::ClientOptions, Client};
use std::marker::PhantomData;
pub struct MongoDB<T> {
    pub client: Client,
    _marker: PhantomData<T>,
}

impl<T> MongoDB<T> {
    pub async fn new(url: &'static str, app_name: &'static str) -> Self {
        let mut client_options = ClientOptions::parse(url)
            .await
            .expect("can not connect to mongodb");
        client_options.app_name = Some(String::from(app_name));
        let client = Client::with_options(client_options).unwrap_or_else(|e| {
            panic!(
                "
            Error building mongodb client: {}
        ",
                e
            )
        });
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

pub mod mongo_mark {
    pub struct Master;
}
