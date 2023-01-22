use bson::Bson;
use dustdata::DustData;
use dustdata::Error as DustDataError;
use rand::Rng;
use rustbase_scram::hash_password;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::config;
use crate::query;
use crate::server;

use config::schema;
use server::cache;
use server::route;
use server::wirewave;

use cache::Cache;
use query::parser::{ASTNode, Keywords, Verbs};
use wirewave::authorization::UserPermission;
use wirewave::server::{Response, Status};

pub struct Core {
    cache: Arc<RwLock<Cache>>,
    routers: Arc<RwLock<HashMap<String, DustData>>>,
    config: Arc<schema::RustbaseConfig>,
    current_database: String,
    system_db: Arc<RwLock<DustData>>,
    current_user: Option<String>,
}

enum TransactionError {
    InternalError(DustDataError),
    ExternalError(Status, String),
}

impl Core {
    pub fn new(
        cache: Arc<RwLock<Cache>>,
        routers: Arc<RwLock<HashMap<String, DustData>>>,
        config: Arc<schema::RustbaseConfig>,
        system_db: Arc<RwLock<DustData>>,
        current_database: String,
        current_user: Option<String>,
    ) -> Self {
        Self {
            cache,
            routers,
            config,
            current_database,
            system_db,
            current_user,
        }
    }

    pub fn run(&mut self, ast: ASTNode) -> Result<Response, Status> {
        match ast {
            ASTNode::IntoExpression {
                keyword,
                json,
                ident,
            } => self.expr_into(keyword, *json, *ident),

            ASTNode::MonadicExpression {
                keyword,
                verb,
                expr,
            } => self.monadic_expr(keyword, verb, expr),

            ASTNode::SingleExpression { keyword, ident } => self.sgl_expr(keyword, ident),
            _ => Err(Status::SyntaxError),
        }
    }

    fn expr_into(
        &mut self,
        keyword: Keywords,
        value: ASTNode,
        expr: ASTNode,
    ) -> Result<Response, Status> {
        match keyword {
            Keywords::Insert => {
                let key = match expr {
                    ASTNode::Identifier(ident) => ident,
                    _ => {
                        unreachable!()
                    }
                };

                let value = match value {
                    ASTNode::Bson(json) => json,
                    _ => {
                        unreachable!()
                    }
                };

                match self.insert_into_dustdata(key, value) {
                    Ok(_) => Ok(Response {
                        message: None,
                        status: Status::Ok,
                        body: None,
                    }),

                    Err(e) => self.dd_error(e),
                }
            }

            Keywords::Update => {
                let key = match expr {
                    ASTNode::Identifier(ident) => ident,
                    _ => {
                        unreachable!()
                    }
                };

                let value = match value {
                    ASTNode::Bson(json) => json,
                    _ => {
                        unreachable!()
                    }
                };

                match self.update_dustdata(key, value) {
                    Ok(_) => Ok(Response {
                        message: None,
                        status: Status::Ok,
                        body: None,
                    }),

                    Err(e) => self.dd_error(e),
                }
            }

            _ => Err(Status::SyntaxError),
        }
    }

    fn monadic_expr(
        &mut self,
        keyword: Keywords,
        verb: Verbs,
        expr: Option<Vec<ASTNode>>,
    ) -> Result<Response, Status> {
        match keyword {
            Keywords::Insert => match verb {
                Verbs::User => {
                    if expr.is_none() {
                        return Err(Status::SyntaxError);
                    }

                    let expr = expr.unwrap();

                    let mut username = String::new();
                    let mut permission = String::new();
                    let mut password = String::new();

                    // idk if this is the best way to do this
                    for node in expr {
                        match node {
                            // this will find the password and permission
                            ASTNode::AssignmentExpression { ident, value } => {
                                match ident.as_str() {
                                    "password" => {
                                        password = match *value {
                                            ASTNode::Bson(s) => {
                                                let s = s.as_str();

                                                // if the password is not a string, return an error
                                                if let Some(s) = s {
                                                    s.to_string()
                                                } else {
                                                    return syntax_error(
                                                        "password must be a string",
                                                    );
                                                }
                                            }

                                            _ => {
                                                return syntax_error("password must be a string");
                                            }
                                        }
                                    }

                                    "permission" => {
                                        permission = match *value {
                                            ASTNode::Bson(s) => {
                                                let s = s.as_str();

                                                // if the permission is not a string, return an error
                                                if let Some(s) = s {
                                                    s.to_string()
                                                } else {
                                                    return syntax_error(
                                                        "permission must be a string",
                                                    );
                                                }
                                            }
                                            _ => {
                                                return syntax_error("permission must be a string");
                                            }
                                        }
                                    }

                                    _ => {}
                                }
                            }

                            ASTNode::Identifier(ref ident) => username = ident.clone(),

                            _ => {}
                        }
                    }

                    if username.is_empty() || password.is_empty() || permission.is_empty() {
                        return syntax_error("username, password, and permission are required");
                    }

                    let permission = UserPermission::from_str(permission.as_str());

                    if permission.is_none() {
                        return syntax_error(
                            "permission must be 'read' or 'write', 'read_and_write', or 'admin'",
                        );
                    }

                    match self.create_user(username, password, permission.unwrap()) {
                        Ok(_) => Ok(Response {
                            message: None,
                            status: Status::Ok,
                            body: None,
                        }),

                        Err(e) => self.dd_error(e),
                    }
                }

                _ => Err(Status::SyntaxError),
            },

            Keywords::Delete => match verb {
                Verbs::Database => {
                    let database = if let Some(expr) = expr {
                        match expr[0] {
                            ASTNode::Identifier(ref ident) => ident.clone(),
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        self.current_database.clone()
                    };

                    match self.delete_database(database) {
                        Ok(_) => Ok(Response {
                            message: None,
                            status: Status::Ok,
                            body: None,
                        }),

                        Err(e) => self.dd_error(e),
                    }
                }

                Verbs::User => {
                    let user = if let Some(expr) = expr {
                        match expr[0] {
                            ASTNode::Identifier(ref ident) => ident.clone(),
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        return Err(Status::SyntaxError);
                    };

                    match self.delete_user(user) {
                        Ok(_) => Ok(Response {
                            message: None,
                            status: Status::Ok,
                            body: None,
                        }),

                        Err(e) => self.dd_error(e),
                    }
                }
            },

            Keywords::Update => match verb {
                Verbs::User => {
                    if expr.is_none() {
                        return Err(Status::SyntaxError);
                    }

                    let mut password: Option<String> = None;
                    let mut permission: Option<String> = None;
                    let mut username = String::new();

                    for node in expr.unwrap() {
                        match node {
                            // this will find the password and permission
                            ASTNode::AssignmentExpression { ident, value } => {
                                match ident.as_str() {
                                    "password" => {
                                        password = match *value {
                                            ASTNode::Bson(s) => {
                                                let s = s.as_str();

                                                // if the password is not a string, return an error
                                                if let Some(s) = s {
                                                    Some(s.to_string())
                                                } else {
                                                    return syntax_error(
                                                        "password must be a string",
                                                    );
                                                }
                                            }
                                            _ => None,
                                        }
                                    }

                                    "permission" => {
                                        permission = match *value {
                                            ASTNode::Bson(s) => {
                                                let s = s.as_str();

                                                // if the password is not a string, return an error
                                                if let Some(s) = s {
                                                    Some(s.to_string())
                                                } else {
                                                    return syntax_error(
                                                        "permission must be a string",
                                                    );
                                                }
                                            }
                                            _ => None,
                                        }
                                    }

                                    _ => {}
                                }
                            }

                            ASTNode::Identifier(ref ident) => username = ident.clone(),

                            _ => {}
                        }
                    }

                    let permission: Option<UserPermission> = if let Some(permission) = permission {
                        UserPermission::from_str(permission.as_str())
                    } else {
                        None
                    };

                    match self.update_user(username, password, permission) {
                        Ok(_) => Ok(Response {
                            message: None,
                            status: Status::Ok,
                            body: None,
                        }),

                        Err(e) => self.dd_error(e),
                    }
                }

                _ => Err(Status::SyntaxError),
            },

            _ => Err(Status::SyntaxError),
        }
    }

    fn sgl_expr(
        &mut self,
        keyword: Keywords,
        ident: Option<Box<ASTNode>>,
    ) -> Result<Response, Status> {
        match keyword {
            Keywords::Get => {
                if ident.is_none() {
                    return Err(Status::SyntaxError);
                }

                let key = match *ident.unwrap() {
                    ASTNode::Identifier(ident) => ident,
                    _ => {
                        unreachable!()
                    }
                };

                match self.get_from_dustdata(key) {
                    Ok(value) => Ok(Response {
                        message: None,
                        status: Status::Ok,
                        body: Some(value),
                    }),

                    Err(e) => self.dd_error(e),
                }
            }

            Keywords::Delete => {
                let key = match *ident.unwrap() {
                    ASTNode::Identifier(ident) => ident,
                    _ => {
                        unreachable!()
                    }
                };

                match self.delete_from_dustdata(key) {
                    Ok(_) => Ok(Response {
                        message: None,
                        status: Status::Ok,
                        body: None,
                    }),

                    Err(e) => self.dd_error(e),
                }
            }

            Keywords::List => match self.list_from_dustdata() {
                Ok(keys) => Ok(Response {
                    message: None,
                    status: Status::Ok,
                    body: Some(Bson::Array(keys.into_iter().map(Bson::String).collect())),
                }),

                Err(e) => self.dd_error(e),
            },

            _ => Err(Status::SyntaxError),
        }
    }

    // user dd interface

    fn insert_into_dustdata(&mut self, key: String, value: Bson) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "database.reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Write)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut routers = self.routers.write().unwrap();

        if !routers.contains_key(&self.current_database) {
            let dd = route::create_dustdata(
                &Path::new(&self.config.database.path).join(self.current_database.clone()),
            );

            routers.insert(self.current_database.clone(), dd);
            println!("[Engine] created database {}", self.current_database);
        }

        let dd = routers.get_mut(&self.current_database).unwrap();

        dd.insert(&key, value)
            .map_err(TransactionError::InternalError)
    }

    fn update_dustdata(&mut self, key: String, value: Bson) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "database.reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Write)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut cache = self.cache.write().unwrap();
        let cache_key = format!("{}:{}", self.current_database, key);
        cache.remove(&cache_key).ok();

        let mut routers = self.routers.write().unwrap();
        let dd = routers.get_mut(&self.current_database);

        if let Some(dd) = dd {
            dd.update(&key, value)
                .map_err(TransactionError::InternalError)
        } else {
            Err(TransactionError::ExternalError(
                Status::DatabaseNotFound,
                "database.notFound".to_string(),
            ))
        }
    }

    fn delete_from_dustdata(&mut self, key: String) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "database.reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Write)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut cache = self.cache.write().unwrap();
        let cache_key = format!("{}:{}", self.current_database, key);
        cache.remove(&cache_key).ok();

        let mut routers = self.routers.write().unwrap();
        let dd = routers.get_mut(&self.current_database);

        if let Some(dd) = dd {
            dd.delete(&key).map_err(TransactionError::InternalError)
        } else {
            Err(TransactionError::ExternalError(
                Status::DatabaseNotFound,
                "database.notFound".to_string(),
            ))
        }
    }

    fn get_from_dustdata(&mut self, key: String) -> Result<Bson, TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "database.reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Read)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut cache = self.cache.write().unwrap();

        let cache_key = format!("{}:{}", self.current_database, key);

        if let Some(bson) = cache.get(&cache_key) {
            return Ok(bson.clone());
        }

        let routers = self.routers.read().unwrap();
        let dd = routers.get(&self.current_database);

        if let Some(dd) = dd {
            let value = dd.get(&key).map_err(TransactionError::InternalError)?;

            if let Some(bson) = value {
                cache.insert(cache_key, bson.clone()).unwrap();

                Ok(bson)
            } else {
                Err(TransactionError::ExternalError(
                    Status::KeyNotExists,
                    "key.notFound".to_string(),
                ))
            }
        } else {
            Err(TransactionError::ExternalError(
                Status::DatabaseNotFound,
                "database.notFound".to_string(),
            ))
        }
    }

    fn list_from_dustdata(&mut self) -> Result<Vec<String>, TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "database.reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Read)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let routers = self.routers.read().unwrap();
        let dd = routers.get(&self.current_database).unwrap();

        dd.list_keys().map_err(TransactionError::InternalError)
    }

    fn delete_database(&mut self, database: String) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "database.reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut routers = self.routers.write().unwrap();

        if let Some(mut dd) = routers.remove(&database) {
            dd.lsm.drop();
            drop(dd);

            let database = database.clone();

            // using thread to delete database because it's a blocking operation
            let c_db = database.clone();
            let c_path = self.config.database.path.clone();
            std::thread::spawn(move || {
                route::remove_dustdata(&c_path, c_db);
            });

            println!("[Engine] database {} deleted", database);

            Ok(())
        } else {
            Err(TransactionError::ExternalError(
                Status::DatabaseNotFound,
                "database.notFound".to_string(),
            ))
        }
    }

    // auth interface
    fn create_user(
        &mut self,
        username: String,
        password: String,
        user_permission: UserPermission,
    ) -> Result<(), TransactionError> {
        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut dd = self.system_db.write().unwrap();

        let salt = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let hash_password =
            hash_password(&password, std::num::NonZeroU32::new(4096).unwrap(), &salt).to_vec();

        let hash_password = bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: hash_password,
        };

        let salt = bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: salt,
        };

        let doc = bson::doc! {
            "password": hash_password,
            "salt": salt,
            "permission": user_permission as i32,
        };

        dd.insert(&username, Bson::Document(doc))
            .map_err(TransactionError::InternalError)
    }

    fn delete_user(&mut self, username: String) -> Result<(), TransactionError> {
        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut dd = self.system_db.write().unwrap();

        dd.delete(&username)
            .map_err(TransactionError::InternalError)
    }

    fn update_user(
        &mut self,
        username: String,
        password: Option<String>,
        user_permission: Option<UserPermission>,
    ) -> Result<(), TransactionError> {
        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::Error,
                    "permission.denied".to_string(),
                ));
            }
        }

        let mut dd = self.system_db.write().unwrap();

        let user = dd.get(&username).map_err(TransactionError::InternalError)?;

        if user.is_none() {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "user.notFound".to_string(),
            ));
        }

        let mut user = user.unwrap();
        let user = user.as_document_mut().unwrap();

        if let Some(password) = password {
            let salt = rand::thread_rng().gen::<[u8; 32]>().to_vec();
            let hash_password =
                hash_password(&password, std::num::NonZeroU32::new(4096).unwrap(), &salt).to_vec();

            let hash_password = bson::Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: hash_password,
            };

            let salt = bson::Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: salt,
            };

            let doc = bson::doc! {
                "password": hash_password,
                "salt": salt,
            };

            user.extend(doc);
        }

        if let Some(user_permission) = user_permission {
            user.extend(bson::doc! {
                "permission": user_permission as i32,
            });
        }

        println!("{:?}", user);

        dd.update(&username, bson::to_bson(user).unwrap())
            .map_err(TransactionError::InternalError)
    }

    fn user_has_perm(
        &self,
        username: String,
        perm: UserPermission,
    ) -> Result<bool, TransactionError> {
        let dd = self.system_db.read().unwrap();

        let user = dd.get(&username).map_err(TransactionError::InternalError)?;

        if user.is_none() {
            return Err(TransactionError::ExternalError(
                Status::Error,
                "user.notFound".to_string(),
            ));
        }

        let user = user.unwrap();
        let user = user.as_document().unwrap();

        let user_permission = user.get("permission").unwrap().as_i32().unwrap();
        let user_permission = UserPermission::from_i32(user_permission).unwrap();

        Ok(user_permission.cmp(&perm))
    }

    // error
    fn dd_error(&self, error: TransactionError) -> Result<Response, Status> {
        match error {
            TransactionError::InternalError(e) => {
                let code = parse_dd_error_code(e.code);

                Ok(Response {
                    message: Some(code.1),
                    status: code.0,
                    body: None,
                })
            }
            TransactionError::ExternalError(e, message) => Ok(Response {
                message: Some(message),
                status: e,
                body: None,
            }),
        }
    }
}

fn parse_dd_error_code(code: dustdata::ErrorCode) -> (Status, String) {
    match code {
        dustdata::ErrorCode::KeyExists => {
            (Status::KeyAlreadyExists, "key.alreadyExists".to_string())
        }
        dustdata::ErrorCode::KeyNotExists => (Status::KeyNotExists, "key.notExists".to_string()),
        dustdata::ErrorCode::NotFound => (Status::Error, "notFound".to_string()),
    }
}

fn syntax_error(msg: &str) -> Result<Response, Status> {
    Ok(Response {
        message: Some(msg.to_string()),
        status: Status::SyntaxError,
        body: None,
    })
}
