use plugins_definitions::Plugin;

#[derive(Debug)]
pub enum Expr<'a> {
    Name(SingleOp<'a>),
    Group(SingleOp<'a>),
    Type(SingleOp<'a>),
    MultipleOp(Box<Expr<'a>>, MultipleOp, Box<Expr<'a>>),
    Not(Box<Expr<'a>>),
}

#[derive(Debug)]
pub enum SingleOp<'a> {
    Is(&'a str),
    In(Vec<&'a str>),
}

#[derive(Debug)]
pub enum MultipleOp {
    And,
    Or,
}

impl<'a> Expr<'a> {
    pub fn is_match(&self, plugin: &impl Plugin) -> bool {
        match self {
            Expr::Name(op) => op.is_match(plugin.name()),
            Expr::Group(op) => op.is_match(plugin.group()),
            Expr::Type(op) => op.is_match(plugin.type_()),
            Expr::MultipleOp(l, op, r) => match op {
                MultipleOp::And => l.is_match(plugin) && r.is_match(plugin),
                MultipleOp::Or => l.is_match(plugin) || r.is_match(plugin),
            },
            Expr::Not(expr) => !expr.is_match(plugin),
        }
    }
}

impl<'a> SingleOp<'a> {
    pub fn is_match(&self, value: &str) -> bool {
        match self {
            SingleOp::Is(v) => v == &value,
            SingleOp::In(vs) => vs.contains(&value),
        }
    }
}
