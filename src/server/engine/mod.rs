use super::main::{
    create_dustdata,
    rustbase::{QueryResult, QueryResultType},
};
use crate::{config::schema, query};
use bson::to_vec;
use dustdata::DustData;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};
use tonic::Response;

pub struct Database {
    pub routers: Arc<Mutex<BTreeMap<String, DustData>>>,
    pub cache: Arc<Mutex<super::cache::Cache>>,
    pub config: schema::RustbaseConfig,
}

impl Database {
    pub fn insert(
        &self,
        query: query::parser::InsertQuery,
        database: String,
    ) -> Response<QueryResult> {
        let mut routers = self.routers.lock().unwrap();

        if let Some(dd) = routers.get_mut(&database) {
            dd.insert(&query.key, query.value).unwrap();
        } else {
            let mut dd = create_dustdata(
                Path::new(&self.config.database.path)
                    .join(database.clone())
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
            dd.insert(&query.key, query.value).unwrap();

            routers.insert(database.clone(), dd);
        }

        Response::new(QueryResult {
            error_message: None,
            result_type: QueryResultType::Ok as i32,
            bson: None,
            message: None,
        })
    }

    pub fn get(&self, query: query::parser::GetQuery, database: String) -> Response<QueryResult> {
        let mut cache = self.cache.lock().unwrap();
        if cache.contains(query.key.clone()) {
            let v = cache.get(&query.key).unwrap().clone();

            return Response::new(QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(to_vec(&v).unwrap()),
                message: None,
            });
        }

        let mut routers = self.routers.lock().unwrap();

        if !routers.contains_key(&database) {
            return Response::new(QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            });
        }

        let dd = routers.get_mut(&database).unwrap();

        let value = dd.get(&query.key).unwrap();

        if let Some(value) = value {
            cache.insert(query.key, value.clone()).unwrap();

            Response::new(QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: Some(to_vec(&value).unwrap()),
                message: None,
            })
        } else {
            Response::new(QueryResult {
                error_message: None,
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            })
        }
    }

    pub fn update(
        &self,
        query: query::parser::UpdateQuery,
        database: String,
    ) -> Response<QueryResult> {
        let mut cache = self.cache.lock().unwrap();

        if cache.contains(query.key.clone()) {
            cache.remove(&query.key).unwrap();
        }

        let mut routers = self.routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();
            dd.update(&query.key, query.value.clone()).unwrap();

            cache.insert(query.key, query.value).unwrap();

            Response::new(QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: None,
                message: None,
            })
        } else {
            Response::new(QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            })
        }
    }

    pub fn delete(
        &self,
        query: query::parser::DeleteQuery,
        database: String,
    ) -> Response<QueryResult> {
        let mut routers = self.routers.lock().unwrap();

        if routers.contains_key(&database) {
            let dd = routers.get_mut(&database).unwrap();
            dd.delete(&query.key).unwrap();
            self.cache.lock().unwrap().remove(&query.key).unwrap();

            Response::new(QueryResult {
                error_message: None,
                result_type: QueryResultType::Ok as i32,
                bson: None,
                message: None,
            })
        } else {
            Response::new(QueryResult {
                error_message: Some("database.notFound".to_string()),
                result_type: QueryResultType::NotFound as i32,
                bson: None,
                message: None,
            })
        }
    }
}
