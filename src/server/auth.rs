use super::wirewave::server::Request;

pub fn check_auth(request: Request) -> Option<bool> {
    let auth = crate::config::get_auth_config();

    if let Some(auth) = auth {
        let database_auth_basic = format!(
            "Basic {}",
            base64::encode(format!("{}:{}", auth.username, auth.password))
        );

        if let Some(auth) = request.auth {
            if auth == database_auth_basic {
                Some(true) // Authenticated
            } else {
                Some(false) // Wrong auth
            }
        } else {
            Some(false) // No auth provided
        }
    } else {
        None // No auth
    }
}
