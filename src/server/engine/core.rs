use bson::Bson;
use dustdata::DustData;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::config;
use crate::query;
use crate::server;
use crate::server::wirewave::server::ResHeader;

use config::schema;
use server::cache;
use server::wirewave;

use cache::Cache;
use query::parser::{ASTNode, Keywords, Verbs};
use wirewave::authorization::UserPermission;
use wirewave::server::{Error, Response, Status};

use interface::TransactionError;

use super::{interface, var_manager};

pub struct Core {
    interface: interface::DustDataInterface,
    variable_manager: var_manager::VariableManager,
}

struct ExpressionResponse(Option<Bson>);

impl Core {
    pub fn new(
        cache: Arc<RwLock<Cache>>,
        routers: Arc<RwLock<HashMap<String, DustData>>>,
        config: Arc<schema::RustbaseConfig>,
        system_db: Arc<RwLock<DustData>>,
        current_database: String,
        current_user: Option<String>,
    ) -> Self {
        let interface = interface::DustDataInterface::new(
            cache,
            routers,
            config,
            system_db,
            current_database,
            current_user,
        );

        let variable_manager = var_manager::VariableManager::new();

        Self {
            interface,
            variable_manager,
        }
    }

    pub fn run_ast(&mut self, ast: Vec<ASTNode>) -> Result<Response, Error> {
        let mut bodies = Vec::new();

        for node in ast {
            let result = match node {
                ASTNode::IntoExpression {
                    keyword,
                    value,
                    ident,
                } => self.expr_into(keyword, *value, *ident),

                ASTNode::MonadicExpression {
                    keyword,
                    verb,
                    expr,
                } => self.monadic_expr(keyword, verb, expr),

                ASTNode::SingleExpression { keyword, ident } => self.sgl_expr(keyword, ident),

                ASTNode::AssignmentExpression { ident, value } => self.assignment(ident, *value),

                _ => {
                    let error = Error {
                        message: "Invalid query".to_string(),
                        query_message: None,
                        status: Status::InvalidQuery,
                    };

                    return Err(error);
                }
            }?;

            if let Some(body) = result.0 {
                bodies.push(body);
            }
        }

        Ok(Response {
            header: ResHeader {
                status: Status::Ok,
                messages: None,
                is_error: false,
            },
            body: Some(Bson::Array(bodies)),
        })
    }

    fn assignment(&mut self, ident: String, value: ASTNode) -> Result<ExpressionResponse, Error> {
        let value = match value {
            ASTNode::Bson(bson) => bson,

            ASTNode::SingleExpression { keyword, ident } => self.sgl_expr(keyword, ident)?.0.into(),
            ASTNode::IntoExpression {
                keyword,
                value,
                ident,
            } => self.expr_into(keyword, *value, *ident)?.0.into(),
            ASTNode::MonadicExpression {
                keyword,
                verb,
                expr,
            } => self.monadic_expr(keyword, verb, expr)?.0.into(),

            _ => {
                return Err(query_error(
                    "value must be a json object, json array, string, float, integer or boolean",
                ));
            }
        };

        self.variable_manager.set(&ident, value);

        Ok(ExpressionResponse(None))
    }

    fn expr_into(
        &mut self,
        keyword: Keywords,
        value: ASTNode,
        expr: ASTNode,
    ) -> Result<ExpressionResponse, Error> {
        match keyword {
            Keywords::Insert => self.ast_into_insert(value, expr),

            Keywords::Update => self.ast_into_update(value, expr),

            _ => {
                let error = Error {
                    message: format!("{:?} is unexpected for into expression", keyword),
                    query_message: None,
                    status: Status::InvalidQuery,
                };

                Err(error)
            }
        }
    }

    fn monadic_expr(
        &mut self,
        keyword: Keywords,
        verb: Verbs,
        expr: Option<Vec<ASTNode>>,
    ) -> Result<ExpressionResponse, Error> {
        match keyword {
            Keywords::Insert => match verb {
                Verbs::User => self.ast_user_insert(expr),

                _ => {
                    let error = Error {
                        message: format!("{:?} is unexpected for insert expression", verb),
                        query_message: None,
                        status: Status::InvalidQuery,
                    };

                    Err(error)
                }
            },

            Keywords::Delete => match verb {
                Verbs::Database => self.ast_database_delete(expr),

                Verbs::User => self.ast_user_delete(expr),
            },

            Keywords::Update => match verb {
                Verbs::User => self.ast_user_update(expr),

                _ => {
                    let error = Error {
                        message: format!("{:?} is unexpected for update expression", verb),
                        query_message: None,
                        status: Status::InvalidQuery,
                    };

                    Err(error)
                }
            },

            _ => {
                let error = Error {
                    message: format!("{:?} is unexpected for monadic expression", keyword),
                    query_message: None,
                    status: Status::InvalidQuery,
                };

                Err(error)
            }
        }
    }

    fn sgl_expr(
        &mut self,
        keyword: Keywords,
        ident: Option<Box<ASTNode>>,
    ) -> Result<ExpressionResponse, Error> {
        match keyword {
            Keywords::Get => self.ast_sgl_get(ident),

            Keywords::Delete => self.ast_sgl_delete(ident),

            Keywords::List => self.ast_sgl_list(),

            _ => {
                let error = Error {
                    message: format!("{:?} is unexpected for single expression", keyword),
                    query_message: None,
                    status: Status::InvalidQuery,
                };

                Err(error)
            }
        }
    }

    fn ast_into_insert(
        &mut self,
        value: ASTNode,
        expr: ASTNode,
    ) -> Result<ExpressionResponse, Error> {
        let key = match expr {
            ASTNode::Identifier(ident) => ident,
            ASTNode::VariableIdentifier(ref key) => {
                let value = self.variable_manager.get(key);

                if value.is_none() {
                    return Err(query_error("variable not found"));
                }

                if let Bson::String(key) = value.unwrap() {
                    key.to_owned()
                } else {
                    return Err(query_error("variable must be a string"));
                }
            }
            _ => return Err(query_error("key must be an identifier")),
        };

        let value = match value {
            ASTNode::Bson(json) => json,
            ASTNode::VariableIdentifier(ref key) => {
                let value = self.variable_manager.get(key);

                if value.is_none() {
                    return Err(query_error("variable not found"));
                }

                value.unwrap().clone()
            }
            _ => return Err(query_error("value must be a json object")),
        };

        match self.interface.insert_into_dustdata(key, value) {
            Ok(result) => Ok(ExpressionResponse(Some(result))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    fn ast_into_update(
        &mut self,
        value: ASTNode,
        expr: ASTNode,
    ) -> Result<ExpressionResponse, Error> {
        let key = match expr {
            ASTNode::Identifier(ident) => ident,
            ASTNode::VariableIdentifier(ref key) => {
                let value = self.variable_manager.get(key);

                if value.is_none() {
                    return Err(query_error("variable not found"));
                }

                if let Bson::String(key) = value.unwrap() {
                    key.to_owned()
                } else {
                    return Err(query_error("variable must be a string"));
                }
            }
            _ => return Err(query_error("key must be an identifier")),
        };

        let value = match value {
            ASTNode::Bson(json) => json,
            ASTNode::VariableIdentifier(ref key) => {
                let value = self.variable_manager.get(key);

                if value.is_none() {
                    return Err(query_error("variable not found"));
                }

                value.unwrap().clone()
            }
            _ => return Err(query_error("value must be a json object")),
        };

        match self.interface.update_dustdata(key, value) {
            Ok(result) => Ok(ExpressionResponse(Some(result))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    fn ast_user_insert(&mut self, expr: Option<Vec<ASTNode>>) -> Result<ExpressionResponse, Error> {
        if expr.is_none() {
            return Err(query_error("user insert must have an expression"));
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
                                        return Err(query_error("password must be a string"));
                                    }
                                }

                                _ => {
                                    return Err(query_error("password must be a string"));
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
                                        return Err(query_error("permission must be a string"));
                                    }
                                }
                                _ => {
                                    return Err(query_error("permission must be a string"));
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
            return Err(query_error(
                "username, password, and permission are required",
            ));
        }

        let permission = UserPermission::from_str(permission.as_str());

        if permission.is_err() {
            return Err(query_error(
                "permission must be 'read' or 'write', 'read_and_write', or 'admin'",
            ));
        }

        match self
            .interface
            .create_user(username, password, permission.unwrap())
        {
            Ok(result) => Ok(ExpressionResponse(Some(result))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    fn ast_user_delete(&mut self, expr: Option<Vec<ASTNode>>) -> Result<ExpressionResponse, Error> {
        let user = if let Some(expr) = expr {
            match expr[0] {
                ASTNode::Identifier(ref ident) => ident.clone(),
                ASTNode::VariableIdentifier(ref key) => {
                    let value = self.variable_manager.get(key);

                    if value.is_none() {
                        return Err(query_error("variable not found"));
                    }

                    if let Bson::String(key) = value.unwrap() {
                        key.to_owned()
                    } else {
                        return Err(query_error("variable must be a string"));
                    }
                }
                _ => {
                    return Err(query_error("user delete must have an expression"));
                }
            }
        } else {
            return Err(query_error("user delete must have an expression"));
        };

        match self.interface.delete_user(user) {
            Ok(result) => Ok(ExpressionResponse(Some(result))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    fn ast_user_update(&mut self, expr: Option<Vec<ASTNode>>) -> Result<ExpressionResponse, Error> {
        if expr.is_none() {
            return Err(query_error("user update must have an expression"));
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
                                        return Err(query_error("password must be a string"));
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
                                        return Err(query_error("permission must be a string"));
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

        let permission = if let Some(permission) = permission {
            let permission = UserPermission::from_str(permission.as_str());
            if permission.is_err() {
                return Err(query_error(
                    "permission must be 'read' or 'write', 'read_and_write', or 'admin'",
                ));
            };

            Some(permission.unwrap())
        } else {
            None
        };

        match self.interface.update_user(username, password, permission) {
            Ok(result) => Ok(ExpressionResponse(Some(result))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    fn ast_database_delete(
        &mut self,
        expr: Option<Vec<ASTNode>>,
    ) -> Result<ExpressionResponse, Error> {
        let database = if let Some(expr) = expr {
            match expr[0] {
                ASTNode::Identifier(ref ident) => ident.clone(),
                ASTNode::VariableIdentifier(ref key) => {
                    let value = self.variable_manager.get(key);

                    if value.is_none() {
                        return Err(query_error("variable not found"));
                    }

                    if let Bson::String(key) = value.unwrap() {
                        key.to_owned()
                    } else {
                        return Err(query_error("variable must be a string"));
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        } else {
            self.interface.current_database.clone()
        };

        match self.interface.delete_database(database) {
            Ok(result) => Ok(ExpressionResponse(Some(result))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    fn ast_sgl_get(&mut self, ident: Option<Box<ASTNode>>) -> Result<ExpressionResponse, Error> {
        if ident.is_none() {
            return Err(query_error("get must have an expression"));
        }

        match *ident.unwrap() {
            ASTNode::Identifier(key) => match self.interface.get_from_dustdata(key) {
                Ok(result) => Ok(ExpressionResponse(Some(result))),

                Err(e) => Err(self.dd_error(e)),
            },

            ASTNode::VariableIdentifier(ref key) => {
                let value = self.variable_manager.get(key);

                if value.is_none() {
                    return Err(query_error("variable not found"));
                }

                Ok(ExpressionResponse(value.cloned()))
            }

            ASTNode::Bson(bson) => Ok(ExpressionResponse(Some(bson))),

            _ => {
                unreachable!()
            }
        }
    }

    fn ast_sgl_delete(&mut self, ident: Option<Box<ASTNode>>) -> Result<ExpressionResponse, Error> {
        match *ident.unwrap() {
            ASTNode::Identifier(ident) => match self.interface.delete_from_dustdata(ident) {
                Ok(result) => Ok(ExpressionResponse(Some(result))),

                Err(e) => Err(self.dd_error(e)),
            },

            ASTNode::VariableIdentifier(ref key) => {
                let value = self.variable_manager.get(key);

                if value.is_none() {
                    return Err(query_error("variable not found"));
                }

                if let Bson::String(key) = value.unwrap() {
                    match self.interface.delete_from_dustdata(key.to_owned()) {
                        Ok(result) => Ok(ExpressionResponse(Some(result))),

                        Err(e) => Err(self.dd_error(e)),
                    }
                } else {
                    Err(query_error("variable must be a string"))
                }
            }

            _ => {
                unreachable!()
            }
        }
    }

    fn ast_sgl_list(&mut self) -> Result<ExpressionResponse, Error> {
        match self.interface.list_from_dustdata() {
            Ok(keys) => Ok(ExpressionResponse(Some(Bson::Array(
                keys.into_iter().map(Bson::String).collect(),
            )))),

            Err(e) => Err(self.dd_error(e)),
        }
    }

    // error
    fn dd_error(&self, error: TransactionError) -> Error {
        match error {
            TransactionError::InternalError(e) => {
                let code = parse_dd_error_code(e.code);

                Error {
                    message: code.1,
                    status: code.0,
                    query_message: None,
                }
            }
            TransactionError::ExternalError(e, message) => Error {
                message,
                status: e,
                query_message: None,
            },
        }
    }
}

fn parse_dd_error_code(code: dustdata::ErrorCode) -> (Status, String) {
    match code {
        dustdata::ErrorCode::KeyExists => (Status::AlreadyExists, "key already exists".to_string()),
        dustdata::ErrorCode::KeyNotExists => (Status::AlreadyExists, "key not exists".to_string()),
        dustdata::ErrorCode::NotFound => (Status::NotFound, "not found".to_string()),
    }
}

fn query_error(msg: &str) -> Error {
    Error {
        message: msg.to_string(),
        status: Status::InvalidQuery,
        query_message: None,
    }
}
