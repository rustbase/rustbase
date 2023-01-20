use scram::{AuthenticationProvider, AuthenticationStatus, PasswordInfo, ScramServer};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use super::server;

use server::read_socket;
use server::{Response, Status};

#[derive(Clone)]
pub struct DefaultAuthenticationProvider {
    pub dustdata: Arc<RwLock<dustdata::DustData>>,
}

impl AuthenticationProvider for DefaultAuthenticationProvider {
    fn get_password_for(&self, username: &str) -> Option<PasswordInfo> {
        let dustdata = self.dustdata.read().unwrap();

        let user = dustdata.get(username).unwrap();

        if let Some(user) = user {
            let user = user.as_document().unwrap();

            let hashed_password = user.get_binary_generic("password").unwrap();
            let salt = user.get_binary_generic("salt").unwrap();

            Some(PasswordInfo::new(
                hashed_password.to_vec(),
                4096,
                salt.to_vec(),
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest {
    challenge: String,
}

fn process_authentication_request(buffer: &[u8]) -> Result<AuthRequest, Response> {
    let challenge: String = match String::from_utf8(buffer.to_vec()) {
        Ok(request) => request,
        Err(e) => {
            let response = Response {
                message: Some(e.to_string()),
                status: Status::Error,
                body: None,
            };

            return Err(response);
        }
    };

    Ok(AuthRequest { challenge })
}

#[allow(clippy::unused_io_amount)]
pub async fn authentication_challenge<IO>(
    scram_server: ScramServer<DefaultAuthenticationProvider>,
    stream: &mut IO,
) -> AuthenticationStatus
where
    IO: AsyncWrite + AsyncRead + Unpin,
{
    let mut buffer = vec![0; 1028];

    let client_first =
        match process_authentication_request(&read_socket(stream, &mut buffer).await.unwrap()) {
            Ok(request) => request.challenge,
            Err(response) => {
                stream
                    .write_all(&bson::to_vec(&response).unwrap())
                    .await
                    .unwrap();
                return AuthenticationStatus::NotAuthenticated;
            }
        };

    let scram_first = scram_server.handle_client_first(&client_first).unwrap();

    let (scram_server, server_first) = scram_first.server_first();

    stream.write_all(server_first.as_bytes()).await.unwrap();

    let client_final =
        match process_authentication_request(&read_socket(stream, &mut buffer).await.unwrap()) {
            Ok(request) => request.challenge,
            Err(response) => {
                stream
                    .write_all(&bson::to_vec(&response).unwrap())
                    .await
                    .unwrap();
                return AuthenticationStatus::NotAuthenticated;
            }
        };

    let scram_server = scram_server.handle_client_final(&client_final).unwrap();

    let (status, server_final) = scram_server.server_final();

    stream.write_all(server_final.as_bytes()).await.unwrap();

    status
}
