use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

#[async_trait]
pub trait Wirewave: Send + Sync + 'static {
    async fn request(&self, request: Request) -> Result<Response, Status>;
}

pub struct WirewaveServer<T: Wirewave> {
    inner: _Inner<T>,
}

impl<T: Wirewave> WirewaveServer<T> {
    pub fn new(inner: T) -> Self {
        let inner = _Inner(Arc::new(inner));
        Self { inner }
    }
}

struct _Inner<T>(Arc<T>);

pub struct Server<T: Wirewave> {
    svc: WirewaveServer<T>,
}

impl<T: Wirewave> Server<T> {
    pub fn new(svc: WirewaveServer<T>) -> Self {
        Self { svc }
    }

    pub async fn serve<A: ToSocketAddrs>(self, addr: A) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;

            let svc = self.svc.clone();

            tokio::spawn(async move {
                handle_connection(stream, move |request| {
                    let svc = svc.clone();
                    async move { svc.inner.0.request(request).await }
                })
                .await;
            });
        }
    }
}

impl<T: Wirewave> Clone for WirewaveServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Wirewave> Clone for _Inner<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug)]
pub struct Request {
    pub body: bson::Bson,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub message: Option<String>,
    pub status: Status,
    pub body: Option<bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Error,
    DatabaseNotFound,
    KeyNotExists,
    KeyAlreadyExists,
    SyntaxError,
    InvalidQuery,
    InvalidBody,
}

pub async fn handle_connection<F, Fut>(mut socket: TcpStream, callback: F)
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response, Status>>,
{
    let mut buf = [0; 1024];

    loop {
        match socket.read(&mut buf).await {
            Ok(n) if n == 0 => return,
            Ok(n) => {
                let doc: bson::Document = bson::from_slice(&buf[0..n]).unwrap();

                let body = doc.get("body").unwrap().clone();

                let request = Request { body };

                let response = match callback(request).await {
                    Ok(response) => response,
                    Err(status) => Response {
                        message: None,
                        status,
                        body: None,
                    },
                };

                let response = bson::to_bson(&response).unwrap();

                let response = bson::to_vec(&response).unwrap();

                socket.write_all(&response).await.unwrap();
            }
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
            }
        };
    }
}
