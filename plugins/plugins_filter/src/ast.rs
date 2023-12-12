use plugins_definitions::PluginMeta;
use regex::Regex;

#[derive(Debug)]
pub enum Expr {
    Name(SingleOp),
    Group(SingleOp),
    Type(SingleOp),
    MultipleOp(Box<Expr>, MultipleOp, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug)]
pub enum SingleOp {
    Is(String),
    In(Vec<String>),
    Matches(Regex),
}

#[derive(Debug)]
pub enum MultipleOp {
    And,
    Or,
}
impl Expr {
    pub fn is_match(&self, meta: &PluginMeta) -> bool {
        match self {
            Expr::Name(op) => op.is_match(meta.name),
            Expr::Group(op) => op.is_match(meta.group),
            Expr::Type(op) => op.is_match(meta.type_),
            Expr::MultipleOp(l, op, r) => match op {
                MultipleOp::And => l.is_match(meta) && r.is_match(meta),
                MultipleOp::Or => l.is_match(meta) || r.is_match(meta),
            },
            Expr::Not(expr) => !expr.is_match(meta),
        }
    }
}

impl SingleOp {
    pub fn is_match(&self, value: &str) -> bool {
        match self {
            SingleOp::Is(v) => v == value,
            SingleOp::In(vs) => vs.contains(&value.to_string()),
            SingleOp::Matches(re) => re.is_match(value),
        }
    }
}
