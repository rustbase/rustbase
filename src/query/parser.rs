use super::{QueryError, QueryErrorType, Result};
use bson::{Bson, Document};
use pest::iterators::Pair;
use pest::Parser;

#[derive(Debug)]
pub enum Query {
    Get(GetQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
    List,
}

#[derive(Debug)]
pub struct GetQuery {
    pub key: String,
}

#[derive(Debug)]
pub struct InsertQuery {
    pub key: String,
    pub value: Bson,
}

#[derive(Debug)]
pub struct UpdateQuery {
    pub key: String,
    pub value: Bson,
}

#[derive(Debug)]
pub struct DeleteQuery {
    pub key: String,
}

#[derive(pest_derive::Parser)]
#[grammar = "query/grammar/rustbase.pest"]
struct RustbaseParser;

pub fn parse_to_bson(pair: Pair<Rule>) -> Bson {
    match pair.as_rule() {
        Rule::object => {
            let mut doc = Document::new();
            for pair in pair.into_inner() {
                let mut inner_rules = pair.into_inner();
                let key = inner_rules
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_string()
                    .replace('"', ""); // bro, this ident is ugly lol

                let value = parse_to_bson(inner_rules.next().unwrap());
                doc.insert(key, value);
            }
            Bson::Document(doc)
        }
        Rule::array => {
            let mut arr = Vec::new();
            for pair in pair.into_inner() {
                arr.push(parse_to_bson(pair));
            }
            Bson::Array(arr)
        }
        Rule::string => Bson::String(
            pair.into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_string()
                .replace('"', ""),
        ),
        Rule::number => Bson::Int64(pair.as_str().parse().unwrap()),
        Rule::boolean => Bson::Boolean(pair.as_str().parse().unwrap()),
        Rule::null => Bson::Null,
        _ => unreachable!(),
    }
}

pub fn parse(input: String) -> Result<Query> {
    let pairs = match RustbaseParser::parse(Rule::crud, &input) {
        Ok(e) => e,
        Err(e) => {
            return Err(QueryError(QueryErrorType::SyntaxError, e.to_string()));
        }
    };

    for pair in pairs {
        match pair.as_rule() {
            Rule::insert => {
                let mut inner_rules = pair.into_inner();
                let value = parse_to_bson(inner_rules.next().unwrap());
                let key = inner_rules.next().unwrap().as_str().to_string();
                return Ok(Query::Insert(InsertQuery { key, value }));
            }

            Rule::get => {
                let key = pair.into_inner().next().unwrap().as_str().to_string();
                return Ok(Query::Get(GetQuery { key }));
            }

            Rule::update => {
                let mut inner_rules = pair.into_inner();
                let key = inner_rules.next().unwrap().as_str().to_string();
                let value = parse_to_bson(inner_rules.next().unwrap());
                return Ok(Query::Update(UpdateQuery { key, value }));
            }

            Rule::delete => {
                let key = pair.into_inner().next().unwrap().as_str().to_string();
                return Ok(Query::Delete(DeleteQuery { key }));
            }

            Rule::list => return Ok(Query::List),

            _ => {
                unreachable!()
            }
        }
    }

    unreachable!()
}
