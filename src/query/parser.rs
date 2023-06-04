use super::{GrammarError, Result};
use bson::{Bson, Document};
use pest::iterators::Pair;
use pest::Parser;

#[derive(Debug, Clone)]
pub enum Keywords {
    Insert,
    Get,
    Update,
    Delete,
    List,
}

#[derive(Debug, Clone)]
pub enum Verbs {
    User,
    Database,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    // expressions
    AssignmentExpression {
        ident: String,
        value: Box<ASTNode>,
    },

    MonadicExpression {
        keyword: Keywords,
        verb: Verbs,
        expr: Option<Vec<ASTNode>>,
    },

    IntoExpression {
        keyword: Keywords,
        value: Box<ASTNode>,
        ident: Box<ASTNode>,
    },

    SingleExpression {
        keyword: Keywords,
        ident: Option<Box<ASTNode>>,
    },

    Bson(Bson),
    Identifier(String),
    VariableIdentifier(String),
}

#[derive(pest_derive::Parser)]
#[grammar = "query/grammar/rustbase.pest"]
struct RustbaseParser;

pub fn parse(input: &str) -> Result<Vec<ASTNode>> {
    let pairs = match RustbaseParser::parse(Rule::program, input) {
        Ok(e) => e,
        Err(e) => {
            return Err(GrammarError::with_message(&e.to_string()));
        }
    };

    let mut ast = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => break,
            _ => ast.push(build_expr(pair)?),
        }
    }

    Ok(ast)
}

fn build_expr(pair: Pair<Rule>) -> Result<ASTNode> {
    match pair.as_rule() {
        Rule::assgmtExpr => {
            let mut inner_rules = pair.into_inner();
            let ident = inner_rules.next().unwrap();
            let value = inner_rules.next().unwrap();

            Ok(ASTNode::AssignmentExpression {
                ident: ident.as_str().to_string(),
                value: Box::new(build_expr(value)?),
            })
        }

        Rule::monadicExpr => {
            let mut inner_rules = pair.clone().into_inner();
            let keyword = inner_rules.next().unwrap();
            let verb = inner_rules.next().unwrap();

            let mut exprs = Vec::new();

            for pair in inner_rules {
                match pair.as_rule() {
                    Rule::ident => exprs.push(build_term(pair)?),
                    _ => exprs.push(build_expr(pair)?),
                }
            }

            Ok(ASTNode::MonadicExpression {
                keyword: match keyword.as_str() {
                    "insert" => Keywords::Insert,
                    "delete" => Keywords::Delete,
                    "update" => Keywords::Update,
                    _ => return Err(GrammarError::with_pair("invalid keyword", keyword)),
                },
                verb: match verb.as_str() {
                    "user" => Verbs::User,
                    "database" => Verbs::Database,
                    _ => return Err(GrammarError::with_pair("invalid verb", verb)),
                },
                expr: if exprs.is_empty() { None } else { Some(exprs) },
            })
        }

        Rule::intoExpr => {
            let mut inner_rules = pair.into_inner();
            let keyword = inner_rules.next().unwrap();
            let json = inner_rules.next().unwrap();
            let ident = inner_rules.next().unwrap();

            Ok(ASTNode::IntoExpression {
                keyword: match keyword.as_str() {
                    "insert" => Keywords::Insert,
                    "update" => Keywords::Update,
                    _ => return Err(GrammarError::with_pair("invalid keyword", keyword)),
                },
                value: Box::new(build_term(json)?),
                ident: Box::new(build_term(ident)?),
            })
        }

        Rule::sglExpr => {
            let mut inner_rules = pair.into_inner();
            let keyword = inner_rules.next().unwrap();
            let ident = inner_rules.next();

            Ok(ASTNode::SingleExpression {
                keyword: match keyword.as_str() {
                    "get" => Keywords::Get,
                    "delete" => Keywords::Delete,
                    "list" => Keywords::List,
                    _ => return Err(GrammarError::with_pair("invalid keyword", keyword)),
                },
                ident: if let Some(ident) = ident {
                    Some(Box::new(build_term(ident)?))
                } else {
                    None
                },
            })
        }

        Rule::term => Ok(build_term(pair)?),

        _ => {
            println!("{:#?}", pair.as_rule());
            unreachable!()
        }
    }
}

fn build_term(pair: Pair<Rule>) -> Result<ASTNode> {
    match pair.as_rule() {
        Rule::integer
        | Rule::float
        | Rule::string
        | Rule::boolean
        | Rule::null
        | Rule::array
        | Rule::object => Ok(ASTNode::Bson(parse_to_bson(pair))),
        Rule::ident => Ok(ASTNode::Identifier(pair.as_str().to_string())),
        Rule::var_ident => Ok(ASTNode::VariableIdentifier(
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),
        Rule::term => Ok(build_term(pair.into_inner().next().unwrap())?),
        _ => {
            unreachable!()
        }
    }
}

fn parse_to_bson(pair: Pair<Rule>) -> Bson {
    match pair.as_rule() {
        Rule::object => {
            let mut doc = Document::new();
            for pair in pair.into_inner() {
                let mut inner_rules = pair.into_inner();
                let key = parse_to_bson(inner_rules.next().unwrap());

                let value = parse_to_bson(inner_rules.next().unwrap());
                doc.insert(key.as_str().unwrap(), value);
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
        Rule::string => Bson::String(pair.into_inner().next().unwrap().as_str().to_string()),
        Rule::integer => Bson::Int64(pair.as_str().parse().unwrap()),
        Rule::float => Bson::Double(pair.as_str().parse().unwrap()),
        Rule::boolean => Bson::Boolean(pair.as_str().parse().unwrap()),
        Rule::null => Bson::Null,
        _ => {
            unreachable!();
        }
    }
}
