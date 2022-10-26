use crate::libs::global::get_global_env;
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::marker::PhantomData;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::Duration;

const MAX_RETRY: u8 = 5;
const RETRY_TICK: u64 = 60;
pub struct MailRequest {
    to: Vec<String>,
    content: String,
    subject: String,
}
impl MailRequest {
    pub fn new(to: Vec<String>, content: String, subject: String) -> Self {
        Self {
            to,
            content,
            subject,
        }
    }
}

pub struct EmailServer<T> {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
    pub email_user: String,
    pub email_pwd: String,
    sender: UnboundedSender<MailRequest>,
    recv: UnboundedReceiver<MailRequest>,
    _marker: PhantomData<T>,
}
impl<T> EmailServer<T> {
    pub fn new() -> Self {
        let env = get_global_env();
        let email_user = env.get("EMAILUSER").unwrap().to_string();
        let email_pwd = env.get("EMAILPWD").unwrap().to_string();
        let (sender, recv) = unbounded_channel();
        let creds = Credentials::new(email_user.clone(), email_pwd.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(env.get("STMP").unwrap())
            .unwrap()
            .credentials(creds)
            .build();
        EmailServer {
            mailer,
            sender,
            recv,
            email_pwd,
            email_user,
            _marker: PhantomData,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.recv.recv().await {
                Some(m) => {
                    // tokio::spawn(async {
                    send_many_eamil(&self.email_user, &self.mailer, m).await;
                    // });
                }
                _ => {}
            }
        }
    }
    pub fn get_sender(&self) -> UnboundedSender<MailRequest> {
        self.sender.clone()
    }
}

pub async fn send_many_eamil(
    email_user: &String,
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    m: MailRequest,
) {
    // let env = env::get();
    let mut builder = Message::builder().from(email_user.parse().unwrap());
    for x in m.to {
        builder = builder.to(x.parse().unwrap());
    }
    let message = builder
        .subject(m.subject)
        .multipart(
            MultiPart::alternative().singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(m.content),
            ),
        )
        .expect("failed to build email");
    for _ in 0..MAX_RETRY {
        match mailer.send(message.clone()).await {
            Ok(_) => break,
            Err(e) => {
                println!("Could not send email: {:?}", e);
                println!("retry after 60s");
                tokio::time::sleep(Duration::new(RETRY_TICK, 0)).await;
            }
        }
    }
}
