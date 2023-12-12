use lalrpop_util::lalrpop_mod;

pub mod ast;
lalrpop_mod!(pub filter);

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[test]
    fn parse() {
        let expr = filter::ExprParser::new()
            .parse(
                "!(not (not name is some_name and not group is some_group_name or type is a_type))",
            )
            .unwrap();
        println!("{:?}", expr);

        let expr = filter::ExprParser::new()
            .parse("!name in [some_name, some_other_name, a_name] && group is some_group_name || type is a_type")
            .unwrap();
        println!("{:?}", expr);

        let expr = filter::ExprParser::new()
            .parse(
                "name in [plugin_SoS, plug-in] and not (group in [group990_ss, group_defaults] or type in [wrtier, reader])",
            )
            .unwrap();
        println!("{:?}", expr);

        let expr = filter::ExprParser::new()
            .parse("name matches /^postgres.*$/")
            .unwrap();
        println!("{:?}", expr);
    }
}
