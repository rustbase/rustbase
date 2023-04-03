use bson::Bson;
use dustdata::DustData;
use dustdata::Error as DustDataError;
use rand::Rng;
use rustbase_scram::hash_password;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::config;
use crate::server;

use config::schema;
use server::cache;
use server::route;
use server::wirewave;

use cache::Cache;
use wirewave::authorization::UserPermission;
use wirewave::server::Status;

pub enum TransactionError {
    InternalError(DustDataError),
    ExternalError(Status, String),
}

pub struct DustDataInterface {
    cache: Arc<RwLock<Cache>>,
    routers: Arc<RwLock<HashMap<String, DustData>>>,
    config: Arc<schema::RustbaseConfig>,
    pub current_database: String,
    system_db: Arc<RwLock<DustData>>,
    current_user: Option<String>,
}

impl DustDataInterface {
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

    pub fn insert_into_dustdata(
        &mut self,
        key: String,
        value: Bson,
    ) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Reserved,
                "database reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Write)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
                ));
            }
        }

        let mut routers = self.routers.write().unwrap();

        if !routers.contains_key(&self.current_database) {
            let dd = route::create_dustdata(
                &Path::new(&self.config.storage.path).join(self.current_database.clone()),
            );

            routers.insert(self.current_database.clone(), dd);
            println!("[Engine] created database {}", self.current_database);
        }

        let dd = routers.get_mut(&self.current_database).unwrap();

        dd.insert(&key, value)
            .map_err(TransactionError::InternalError)
    }

    pub fn update_dustdata(&mut self, key: String, value: Bson) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Reserved,
                "database reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Write)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
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
                Status::NotFound,
                "database not found".to_string(),
            ))
        }
    }

    pub fn delete_from_dustdata(&mut self, key: String) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Reserved,
                "database reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Write)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
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
                Status::NotFound,
                "database not found".to_string(),
            ))
        }
    }

    pub fn get_from_dustdata(&mut self, key: String) -> Result<Bson, TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Reserved,
                "database reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Read)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
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
                    Status::NotFound,
                    "key not found".to_string(),
                ))
            }
        } else {
            Err(TransactionError::ExternalError(
                Status::NotFound,
                "database not found".to_string(),
            ))
        }
    }

    pub fn list_from_dustdata(&mut self) -> Result<Vec<String>, TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Reserved,
                "database reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Read)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
                ));
            }
        }

        let routers = self.routers.read().unwrap();
        let dd = routers.get(&self.current_database).unwrap();

        dd.list_keys().map_err(TransactionError::InternalError)
    }

    pub fn delete_database(&mut self, database: String) -> Result<(), TransactionError> {
        if self.current_database == "_default" {
            return Err(TransactionError::ExternalError(
                Status::Reserved,
                "database reserved".to_string(),
            ));
        }

        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
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
            let c_path = self.config.storage.path.clone();
            std::thread::spawn(move || {
                route::remove_dustdata(&c_path, c_db);
            });

            println!("[Engine] database {} deleted", database);

            Ok(())
        } else {
            Err(TransactionError::ExternalError(
                Status::NotFound,
                "database not found".to_string(),
            ))
        }
    }

    pub fn create_user(
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

    pub fn delete_user(&mut self, username: String) -> Result<(), TransactionError> {
        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
                ));
            }
        }

        let mut dd = self.system_db.write().unwrap();

        dd.delete(&username)
            .map_err(TransactionError::InternalError)
    }

    pub fn update_user(
        &mut self,
        username: String,
        password: Option<String>,
        user_permission: Option<UserPermission>,
    ) -> Result<(), TransactionError> {
        if let Some(current_user) = &self.current_user {
            if !self.user_has_perm(current_user.clone(), UserPermission::Admin)? {
                return Err(TransactionError::ExternalError(
                    Status::NotAuthorized,
                    "permission denied".to_string(),
                ));
            }
        }

        let mut dd = self.system_db.write().unwrap();

        let user = dd.get(&username).map_err(TransactionError::InternalError)?;

        if user.is_none() {
            return Err(TransactionError::ExternalError(
                Status::NotFound,
                "user not found".to_string(),
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

        dd.update(&username, bson::to_bson(user).unwrap())
            .map_err(TransactionError::InternalError)
    }

    pub fn user_has_perm(
        &self,
        username: String,
        perm: UserPermission,
    ) -> Result<bool, TransactionError> {
        let dd = self.system_db.read().unwrap();

        let user = dd.get(&username).map_err(TransactionError::InternalError)?;

        if user.is_none() {
            return Err(TransactionError::ExternalError(
                Status::NotFound,
                "user not found".to_string(),
            ));
        }

        let user = user.unwrap();
        let user = user.as_document().unwrap();

        let user_permission = user.get("permission").unwrap().as_i32().unwrap();
        let user_permission = UserPermission::from_i32(user_permission).unwrap();

        Ok(user_permission.cmp(&perm))
    }
}
