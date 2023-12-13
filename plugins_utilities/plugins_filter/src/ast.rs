use plugins_definitions::PluginMeta;
use regex::Regex;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Expr {
    Name(SingleOp),
    Group(SingleOp),
    Type(SingleOp),
    MultipleOp(Box<Expr>, MultipleOp, Box<Expr>),
    Not(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Name(op) => write!(f, "(name {})", op),
            Expr::Group(op) => write!(f, "(group {})", op),
            Expr::Type(op) => write!(f, "(type {})", op),
            Expr::MultipleOp(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Expr::Not(expr) => write!(f, "(not {})", expr),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SingleOp {
    Is(String),
    IsNot(String),
    In(Vec<String>),
    NotIn(Vec<String>),
    Matches(Regex),
}

impl Display for SingleOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingleOp::Is(v) => write!(f, "is {}", v),
            SingleOp::IsNot(v) => write!(f, "is not {}", v),
            SingleOp::In(vs) => write!(f, "in {:?}", vs),
            SingleOp::NotIn(vs) => write!(f, "not in {:?}", vs),
            SingleOp::Matches(re) => write!(f, "matches {}", re.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MultipleOp {
    And,
    Or,
}

impl Display for MultipleOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipleOp::And => write!(f, "and"),
            MultipleOp::Or => write!(f, "or"),
        }
    }
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

    pub fn reduce(self) -> Self {
        match self {
            Expr::MultipleOp(l, op, r) => match (l.reduce(), r.reduce()) {
                (Expr::Not(l), Expr::Not(r)) => Expr::Not(Box::new(Expr::MultipleOp(
                    l,
                    match op {
                        MultipleOp::And => MultipleOp::Or,
                        MultipleOp::Or => MultipleOp::And,
                    },
                    r,
                ))),
                (l, r) => Expr::MultipleOp(Box::new(l), op, Box::new(r)),
            },
            Expr::Not(expr) => match expr.reduce() {
                Expr::Not(expr) => *expr,
                expr => Expr::Not(Box::new(expr)),
            },
            expr => expr,
        }
    }
}

impl SingleOp {
    pub fn is_match(&self, value: &str) -> bool {
        match self {
            SingleOp::Is(v) => v == value,
            SingleOp::IsNot(v) => v != value,
            SingleOp::In(vs) => vs.contains(&value.to_string()),
            SingleOp::NotIn(vs) => !vs.contains(&value.to_string()),
            SingleOp::Matches(re) => re.is_match(value),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::filter;

    #[test]
    fn parse() {
        let expr = filter::ExprParser::new()
            .parse(
                "!(not (not name is some_name and not group is some_group_name or type is a_type))",
            )
            .unwrap();
        println!("{}", expr);

        let exp_red = expr.clone().reduce();
        println!("{}", exp_red);

        let meta = plugins_definitions::PluginMeta {
            name: "_",
            group: "some_group_name",
            type_: "_",
        };

        assert!(!expr.is_match(&meta));
        assert_eq!(exp_red.is_match(&meta), expr.is_match(&meta));

        let meta = plugins_definitions::PluginMeta {
            name: "_",
            group: "_",
            type_: "a_type",
        };

        assert_eq!(exp_red.is_match(&meta), expr.is_match(&meta));

        let meta = plugins_definitions::PluginMeta {
            name: "name",
            group: "group",
            type_: "type",
        };

        assert_eq!(exp_red.is_match(&meta), expr.is_match(&meta));

        let expr = filter::ExprParser::new()
            .parse("!name in [some_name, some_other_name, a_name] && group is some_group_name || type is a_type")
            .unwrap();
        println!("{}", expr);

        let expr = filter::ExprParser::new()
            .parse(
                "name in [plugin_SoS, plug-in] and not (group in [group990_ss, group_defaults] or type in [wrtier, reader])",
            )
            .unwrap();
        println!("{}", expr);

        let expr_1 = filter::ExprParser::new()
            .parse("name matches /^postgres.*$/")
            .unwrap();
        println!("{}", expr_1);

        let expr_2 = filter::ExprParser::new()
            .parse("not name matches /^postgres.*$/")
            .unwrap();
        println!("{}", expr_2);

        let meta = plugins_definitions::PluginMeta {
            name: "postgres_18",
            group: "group",
            type_: "type",
        };

        assert!(expr_1.is_match(&meta));
        assert!(!expr_2.is_match(&meta));

        let meta = plugins_definitions::PluginMeta {
            name: "mysql_18",
            group: "group",
            type_: "type",
        };

        assert!(!expr_1.is_match(&meta));
        assert!(expr_2.is_match(&meta));

        let expr = filter::ExprParser::new()
            .parse("name is not plugin_1")
            .unwrap();
        println!("{}", expr);

        let expr = filter::ExprParser::new().parse("name != plugin_1").unwrap();
        println!("{}", expr);

        let expr = filter::ExprParser::new().parse("name == plugin_1").unwrap();
        println!("{}", expr);

        let expr = filter::ExprParser::new()
            .parse("name not in [plugin_1, plugin_2]")
            .unwrap();
        println!("{}", expr);

        let meta = plugins_definitions::PluginMeta {
            name: "plugin_3",
            group: "group",
            type_: "type",
        };

        assert!(expr.is_match(&meta));

        let meta = plugins_definitions::PluginMeta {
            name: "plugin_1",
            group: "group",
            type_: "type",
        };

        assert!(!expr.is_match(&meta));

        let expr_1 = filter::ExprParser::new()
            .parse("not name not in [plugin_1, plugin_2]")
            .unwrap();
        println!("{}", expr_1);

        let expr_2 = filter::ExprParser::new()
            .parse("name in [plugin_1, plugin_2]")
            .unwrap();
        println!("{}", expr_2);

        let meta = plugins_definitions::PluginMeta {
            name: "plugin_3",
            group: "group",
            type_: "type",
        };

        assert_eq!(expr_1.is_match(&meta), expr_2.is_match(&meta));
    }
}
