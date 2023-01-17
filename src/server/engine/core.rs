use bson::Bson;
use dustdata::DustData;
use dustdata::Error as DustDataError;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::query;
use crate::server;

use config::schema;
use server::cache;
use server::route;
use server::wirewave;

use cache::Cache;
use query::parser::{ASTNode, Keywords, Verbs};
use wirewave::server::{Response, Status};

pub struct Core {
    cache: Arc<Mutex<Cache>>,
    routers: Arc<Mutex<HashMap<String, DustData>>>,
    config: Arc<schema::RustbaseConfig>,
    current_database: String,
}

enum TransactionError {
    InternalError(DustDataError),
    ExternalError(Status, String),
}

impl Core {
    pub fn new(
        cache: Arc<Mutex<Cache>>,
        routers: Arc<Mutex<HashMap<String, DustData>>>,
        config: Arc<schema::RustbaseConfig>,
        current_database: String,
    ) -> Self {
        Self {
            cache,
            routers,
            config,
            current_database,
        }
    }

    pub fn run(&mut self, ast: ASTNode) -> Result<Response, Status> {
        match ast {
            ASTNode::IntoExpression {
                keyword,
                json,
                ident,
            } => self.into_expr(keyword, json, ident),

            ASTNode::MonadicExpression {
                keyword,
                verb,
                expr,
            } => self.monadic_expr(keyword, verb, expr),

            ASTNode::SingleExpression { keyword, ident } => self.sgl_expr(keyword, ident),
            _ => {
                return Err(Status::SyntaxError);
            }
        }
    }

    fn into_expr(
        &mut self,
        keyword: Keywords,
        value: Box<ASTNode>,
        expr: Box<ASTNode>,
    ) -> Result<Response, Status> {
        match keyword {
            Keywords::Insert => {
                let key = match *expr {
                    ASTNode::Identifier(ident) => ident,
                    _ => {
                        unreachable!()
                    }
                };

                let value = match *value {
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
                let key = match *expr {
                    ASTNode::Identifier(ident) => ident,
                    _ => {
                        unreachable!()
                    }
                };

                let value = match *value {
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

            _ => return Err(Status::SyntaxError),
        }
    }

    fn monadic_expr(
        &mut self,
        keyword: Keywords,
        verb: Verbs,
        expr: Option<Vec<Box<ASTNode>>>,
    ) -> Result<Response, Status> {
        match keyword {
            Keywords::Delete => match verb {
                Verbs::Database => {
                    let database = if let Some(expr) = expr {
                        let database = match *expr[0] {
                            ASTNode::Identifier(ref ident) => ident.clone(),
                            _ => {
                                unreachable!()
                            }
                        };

                        database
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

                _ => {
                    return Err(Status::SyntaxError);
                }
            },

            _ => {
                return Err(Status::SyntaxError);
            }
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

            _ => {
                return Err(Status::SyntaxError);
            }
        }
    }

    // dd interface

    fn insert_into_dustdata(&mut self, key: String, value: Bson) -> Result<(), TransactionError> {
        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(&self.current_database) {
            let dd = route::create_dustdata(
                Path::new(&self.config.database.path)
                    .join(self.current_database.clone())
                    .to_str()
                    .unwrap()
                    .to_string(),
            );

            routers.insert(self.current_database.clone(), dd);
            println!("[Engine] created database {}", self.current_database);
        }

        let dd = routers.get_mut(&self.current_database).unwrap();

        dd.insert(&key, value)
            .map_err(|e| TransactionError::InternalError(e))
    }

    fn update_dustdata(&mut self, key: String, value: Bson) -> Result<(), TransactionError> {
        let mut routers = self.routers.lock().unwrap();
        let dd = routers.get_mut(&self.current_database);

        if let Some(dd) = dd {
            dd.update(&key, value)
                .map_err(|e| TransactionError::InternalError(e))
        } else {
            Err(TransactionError::ExternalError(
                Status::DatabaseNotFound,
                "database.notFound".to_string(),
            ))
        }
    }

    fn delete_from_dustdata(&mut self, key: String) -> Result<(), TransactionError> {
        let mut routers = self.routers.lock().unwrap();
        let dd = routers.get_mut(&self.current_database);

        if let Some(dd) = dd {
            dd.delete(&key)
                .map_err(|e| TransactionError::InternalError(e))
        } else {
            Err(TransactionError::ExternalError(
                Status::DatabaseNotFound,
                "database.notFound".to_string(),
            ))
        }
    }

    fn get_from_dustdata(&mut self, key: String) -> Result<Bson, TransactionError> {
        let mut cache = self.cache.lock().unwrap();

        let cache_key = format!("{}:{}", self.current_database, key);

        if let Some(bson) = cache.get(&cache_key) {
            return Ok(bson.clone());
        }

        let mut routers = self.routers.lock().unwrap();
        let dd = routers.get_mut(&self.current_database);

        if let Some(dd) = dd {
            let value = dd
                .get(&key)
                .map_err(|e| TransactionError::InternalError(e))?;

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
        let mut routers = self.routers.lock().unwrap();
        let dd = routers.get_mut(&self.current_database).unwrap();

        dd.list_keys()
            .map_err(|e| TransactionError::InternalError(e))
    }

    fn delete_database(&mut self, database: String) -> Result<(), TransactionError> {
        let mut routers = self.routers.lock().unwrap();

        if let Some(mut dd) = routers.remove(&database) {
            dd.lsm.drop();
            drop(dd);

            let database = database.clone();

            // using thread to delete database because it's a blocking operation
            let c_db = database.clone();
            let c_path = self.config.database.path.clone();
            std::thread::spawn(move || {
                route::remove_dustdata(c_path, c_db);
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

    // -- error
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
