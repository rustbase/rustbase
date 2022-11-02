// use tonic::{Request, Status};

// pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
//     let auth = crate::config::get_auth_config();

//     // basic auth
//     if let Some(auth) = auth {
//         let auth_header = req.metadata().get("authorization");

//         if let Some(auth_header) = auth_header {
//             let auth_header = auth_header.to_str().unwrap();

//             if auth_header
//                 == format!(
//                     "Basic {}",
//                     base64::encode(format!("{}:{}", auth.username, auth.password))
//                 )
//             {
//                 Ok(req)
//             } else {
//                 Err(Status::unauthenticated("Invalid credentials"))
//             }
//         } else {
//             Err(Status::unauthenticated("No credentials provided"))
//         }
//     } else {
//         Ok(req)
//     }
// }
