use crate::config::schema::Tls;
use crate::server::auth::check_auth;

use async_trait::async_trait;
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader};
use std::{future::Future, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

fn load_certs(path: &String) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(path: &String) -> io::Result<Vec<PrivateKey>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

const BUFFER_SIZE: usize = 8 * 1024;

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

    pub async fn serve<A: ToSocketAddrs>(self, addr: A) {
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, addr) = listener.accept().await.unwrap();

            let svc = self.svc.clone();

            tokio::spawn(async move {
                println!("[Wirewave] incoming connection: {}", addr);
                handle_connection(stream, move |request| {
                    let svc = svc.clone();
                    async move { svc.inner.0.request(request).await }
                })
                .await;
            });
        }
    }

    pub async fn serve_tls<A: ToSocketAddrs>(self, addr: A, tls_config: Tls) {
        let certs = load_certs(&tls_config.ca_file).unwrap();
        let keys = load_keys(&tls_config.pem_key_file).unwrap();

        let server_tls_config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, keys[0].clone())
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
            .unwrap();

        let acceptor = TlsAcceptor::from(Arc::new(server_tls_config));

        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (stream, addr) = listener.accept().await.unwrap();

            let svc = self.svc.clone();

            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let stream = acceptor.accept(stream).await.unwrap();

                println!("[Wirewave] incoming connection: {}", addr);
                handle_connection_tls(stream, move |request| {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub auth: Option<String>,
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
    InvalidBson,
    InvalidAuth,
}

// if is ok, return request else return response and send to client
pub fn process_request(buf: &[u8]) -> Result<Request, Response> {
    let request: Request = match bson::from_slice(buf) {
        Ok(request) => request,
        Err(e) => {
            let response = Response {
                message: Some(e.to_string()),
                status: Status::InvalidBson,
                body: None,
            };

            return Err(response);
        }
    };

    if let Some(is_auth) = check_auth(request.clone()) {
        if !is_auth {
            let response = Response {
                message: None,
                status: Status::InvalidAuth,
                body: None,
            };

            return Err(response);
        }
    }

    Ok(request)
}

pub async fn handle_connection<F, Fut>(mut socket: TcpStream, callback: F)
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response, Status>>,
{
    let mut buf = [0; BUFFER_SIZE];

    loop {
        match socket.read(&mut buf).await {
            Err(e) => {
                eprintln!("[Wirewave] failed to read from socket; {:?}", e);
            }

            Ok(0) => {
                println!("[Wirewave] connection closed");
                break;
            }

            Ok(n) => match process_request(&buf[..n]) {
                Ok(request) => {
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
                Err(response) => {
                    let response = bson::to_vec(&response).unwrap();

                    socket.write_all(&response).await.unwrap();
                }
            },
        };
    }
}

pub async fn handle_connection_tls<F, Fut>(mut socket: TlsStream<TcpStream>, callback: F)
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response, Status>>,
{
    let mut buf = [0; BUFFER_SIZE];

    loop {
        match socket.read(&mut buf).await {
            Err(e) => {
                eprintln!("[Wirewave] failed to read from socket; {:?}", e);
            }

            Ok(0) => {
                println!("[Wirewave] connection closed");
                break;
            }

            Ok(n) => match process_request(&buf[..n]) {
                Ok(request) => {
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
                Err(response) => {
                    let response = bson::to_vec(&response).unwrap();

                    socket.write_all(&response).await.unwrap();
                }
            },
        };
    }
}
