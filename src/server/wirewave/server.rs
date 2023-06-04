use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::future::Future;
use std::io::{self, BufReader};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, ToSocketAddrs};

use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;

use rustbase_scram::{AuthenticationStatus, ScramServer};

use super::super::main::current_users;
use super::authentication;
use crate::config;

use authentication::authentication_challenge;

use config::schema::Tls;

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
    async fn request(&self, request: Request, username: Option<String>) -> Result<Response, Error>;
    async fn new_connection(&self, username: Option<String>, addr: SocketAddr);
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
    system_db: Arc<RwLock<dustdata::DustData>>,
    auth_provider: authentication::DefaultAuthenticationProvider,
}

impl<T: Wirewave> Server<T> {
    pub fn new(svc: WirewaveServer<T>, system_db: Arc<RwLock<dustdata::DustData>>) -> Self {
        let auth_provider = authentication::DefaultAuthenticationProvider {
            dustdata: system_db.clone(),
        };

        Self {
            svc,
            auth_provider,
            system_db,
        }
    }

    pub async fn serve<A: ToSocketAddrs>(self, addr: A) {
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (mut stream, addr) = listener.accept().await.unwrap();

            let svc = self.svc.clone();

            let server = ScramServer::new(self.auth_provider.clone());

            let system_db = Arc::clone(&self.system_db);

            tokio::spawn(async move {
                let users = current_users(system_db);

                let require_authentication = users > 0;

                let username = if require_authentication {
                    let (status, username) = authentication_challenge(server, &mut stream).await;

                    if status != AuthenticationStatus::Authenticated {
                        println!("[Wirewave] authentication failed: {:?}", status);
                        stream.shutdown().await.unwrap();

                        return;
                    }

                    username
                } else {
                    None
                };

                svc.inner.0.new_connection(username.clone(), addr).await;

                handle_connection(stream, move |request| {
                    let svc = svc.clone();
                    let username = username.clone();
                    async move { svc.inner.0.request(request, username).await }
                })
                .await;
            });
        }
    }

    pub async fn serve_tls<A: ToSocketAddrs>(self, addr: A, tls_config: &Tls) {
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

            let server = ScramServer::new(self.auth_provider.clone());

            let acceptor = acceptor.clone();

            let system_db = Arc::clone(&self.system_db);

            tokio::spawn(async move {
                let mut stream = acceptor.accept(stream).await.unwrap();

                let users = current_users(system_db);

                let require_authentication = users > 0;

                let username = if require_authentication {
                    let (status, username) = authentication_challenge(server, &mut stream).await;

                    if status != AuthenticationStatus::Authenticated {
                        println!("[Wirewave] authentication failed: {:?}", status);
                        stream.shutdown().await.unwrap();

                        return;
                    }

                    username
                } else {
                    None
                };

                svc.inner.0.new_connection(username.clone(), addr).await;

                handle_connection(stream, move |request| {
                    let svc = svc.clone();
                    let username = username.clone();
                    async move { svc.inner.0.request(request, username).await }
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
    pub body: bson::Document,
    pub header: ReqHeader,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqHeader {
    #[serde(rename = "type")]
    pub type_: Type,
    pub auth: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Type {
    Query,
    Ping,
    PreRequest,
    Cluster,
}

// ----

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub header: ResHeader,
    pub body: Option<bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResHeader {
    pub status: Status,
    pub messages: Option<Vec<String>>,
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
    pub query_message: Option<String>,
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Inserted,
    Updated,

    // ----
    InvalidQuery,
    NotFound,
    AlreadyExists,
    BadBson,
    BadAuth,
    BadBody,
    NotAuthorized,
    Reserved,
    SyntaxError,

    // ----
    InternalError,
}

// if is ok, return request else return response and send to client
fn process_request(buf: &[u8]) -> Result<Request, Response> {
    let request: Request = match bson::from_slice(buf) {
        Ok(request) => request,
        Err(e) => {
            let response = Response {
                body: None,
                header: ResHeader {
                    status: Status::BadBson,
                    messages: Some(vec![e.to_string()]),
                    is_error: true,
                },
            };

            return Err(response);
        }
    };

    Ok(request)
}

pub async fn read_socket<IO>(socket: &mut IO, buffer: &mut [u8]) -> io::Result<Vec<u8>>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    let mut request_bytes = Vec::new();

    while let Ok(n) = socket.read(buffer).await {
        if n == 0 {
            socket.shutdown().await.ok();
            socket.flush().await.ok();
            break;
        }

        request_bytes.extend_from_slice(&buffer[..n]);

        if n < BUFFER_SIZE {
            break;
        }
    }

    Ok(request_bytes)
}

async fn handle_connection<F, Fut, IO>(mut socket: IO, callback: F)
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response, Error>>,
    IO: AsyncRead + AsyncWrite + Unpin,
{
    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        let request_bytes = read_socket(&mut socket, &mut buffer).await.unwrap();

        if request_bytes.is_empty() {
            break;
        }

        match process_request(&request_bytes[..]) {
            Ok(request) => {
                let request = match request.header.type_ {
                    Type::Ping => {
                        let response = Response {
                            body: Some(bson::Bson::Document(request.body)),
                            header: ResHeader {
                                status: Status::Ok,
                                messages: None,
                                is_error: false,
                            },
                        };

                        let response = bson::to_bson(&response).unwrap();
                        let response = bson::to_vec(&response).unwrap();

                        socket.write_all(&response).await.ok();

                        continue;
                    }

                    _ => request,
                };

                let response = match callback(request).await {
                    Ok(response) => response,
                    Err(error) => Response {
                        body: None,
                        header: ResHeader {
                            status: error.status,
                            messages: Some(vec![error.message]),
                            is_error: true,
                        },
                    },
                };

                let response = bson::to_bson(&response).unwrap();
                let response = bson::to_vec(&response).unwrap();

                socket.write_all(&response).await.ok();
            }
            Err(response) => {
                let response = bson::to_vec(&response).unwrap();

                socket.write_all(&response).await.ok();
            }
        }
    }
}
