use scram::{AuthenticationProvider, AuthenticationStatus, PasswordInfo, ScramServer};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct DefaultAuthenticationProvider {
    pub dustdata: Arc<Mutex<dustdata::DustData>>,
}

impl AuthenticationProvider for DefaultAuthenticationProvider {
    fn get_password_for(&self, username: &str) -> Option<PasswordInfo> {
        let dustdata = self.dustdata.lock().unwrap();

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

#[allow(clippy::unused_io_amount)]
pub async fn authentication_challenge(
    scram_server: ScramServer<DefaultAuthenticationProvider>,
    stream: &mut TcpStream,
) -> AuthenticationStatus {
    let mut buffer = vec![0; 1028];

    let n = stream.read(&mut buffer).await.unwrap();
    let client_first = String::from_utf8(buffer[..n].to_vec()).unwrap();

    let scram_first = scram_server.handle_client_first(&client_first).unwrap();

    let (scram_server, server_first) = scram_first.server_first();

    stream.write_all(server_first.as_bytes()).await.unwrap();

    let n = stream.read(&mut buffer).await.unwrap();
    let client_final = String::from_utf8(buffer[..n].to_vec()).unwrap();

    let scram_server = scram_server.handle_client_final(&client_final).unwrap();

    let (status, server_final) = scram_server.server_final();

    stream.write_all(server_final.as_bytes()).await.unwrap();

    status
}
