use crate::context::{ Context, Variable, };

#[test]
fn source_from_file_1() {
    let mut ctx = Context::default();
    ctx.add_variable("person.name", "./resources/context/source_from_file_1/config.cfg", "A Test");

    assert_eq!("A Test", ctx.value("person.name").unwrap());
}

#[test]
fn parse_2() {
    let mut ctx = Context::default();
    ctx.add_variable("main.second", "./resources/context/source_from_file_2/config.cfg", "1");
    ctx.add_variable("main.andthis", "./resources/context/source_from_file_2/config.cfg", "");
    ctx.add_variable("main.list", "./resources/context/source_from_file_2/config.cfg", "1");
    ctx.add_variable("main.list", "./resources/context/source_from_file_2/config.cfg", "2");
    ctx.add_variable("main.list", "./resources/context/source_from_file_2/config.cfg", "3");
    ctx.add_variable("main.list", "./resources/context/source_from_file_2/config.cfg", "4");

    assert!(Variable::value_is_truthy(ctx.value("main.second")));
    assert!(!Variable::value_is_truthy(ctx.value("main.andthis")));
    assert_eq!("", ctx.value("main.andthis").unwrap());
    assert_eq!("4", ctx.value("main.list").unwrap());
    assert_eq!(&"3", ctx.values("main.list").unwrap().get(2).unwrap());
}
