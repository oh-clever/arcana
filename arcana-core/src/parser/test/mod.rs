use {
    crate::{
        context::Context,
        parser::{ Parser, TemplateParser },
    },
    std::path::PathBuf,
};

#[test]
fn parse_chars_1() {
    let mut output = Vec::<u8>::new();
    let input = "this is just some input";
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(input, &output_str);
}

#[test]
fn parse_chars_2() {
    let mut output = Vec::<u8>::new();
    let input = "this is some input with a < sign";
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(input, &output_str);
}

#[test]
fn parse_extend() {
    let mut output = Vec::<u8>::new();
    let input = "Some text, and a tag: {% extend \"./resources/template.txt\" /%}";
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("**Some text, and a tag: **\n", &output_str);
}

#[test]
fn parse_set_text() {
    let mut output = Vec::<u8>::new();
    let input = r#"Some text, and a tag: {% set hello %}Hello, World!{% /set %}"#;
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    let ctx = parser.take_context().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("Some text, and a tag: ", &output_str);

    let hello = ctx.value("hello").unwrap();
    let against_hello = "Hello, World!";
    assert_eq!(&against_hello, hello);
}

#[test]
#[should_panic]
fn parse_set_text_esc() {
    let mut output = Vec::<u8>::new();
    let input = "Some text, and a tag: {% set hello %}\\{%Hello, World %}{% /set %}";
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    let ctx = parser.take_context().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("Some text, and a tag: ", &output_str);

    let hello = ctx.value("hello").unwrap();
    let against_hello = "{%Hello, World %}";
    assert_eq!(&against_hello, hello);
}

#[test]
fn parse_set_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_set_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("Hello, World!", output_str);
}

#[test]
#[should_panic]
fn parse_escape_1() {
    let input = r#"This should be included: \{% extend "../something.txt" /%}"#;
    let mut output = Vec::new();
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        "This should be included: {% extend \"../something.txt\" /%}",
        &output_str
    );
}

#[test]
#[should_panic]
fn parse_escape_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_escape_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
}

#[test]
fn parse_escape_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_escape_3/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "{Title}",
        output_str
    );
}

#[test]
fn parse_escape_4() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_escape_4/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "False was 0.",
        output_str
    );
}

#[test]
#[should_panic]
fn parse_unknown_tag() {
    let input = "{% execut `something` %}";
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
}

#[test]
#[should_panic]
fn parse_unexpected_eof_in_tag_name() {
    let input = "{% execut";
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
}

#[test]
fn parse_include_1() {
    let input = "File: {% include \"./resources/parse-include-1.txt\" /%}";
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(
        "File: {% execute `this should just be included` %}",
        output
    );
}

#[test]
fn parse_include_2() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_include_2/item.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("The name of the item is \"Item.\"", output_str);
}

#[test]
fn parse_comment_1() {
    let input = "Here is some text{# and a comment #} and some more text.";
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(
        "Here is some text and some more text.",
        output,
    );
}

#[test]
#[should_panic]
fn parse_comment_2() {
    let input = "Here is some text{# and a comment \\#} with escaped #} and some more text.";
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(
        "Here is some text and some more text.",
        output,
    );
}

#[test]
fn parse_file_1() {
    let input_file = PathBuf::from("./resources/extended.txt");
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input_file, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(
        "**User(1): test.user, User(2): second.user**\n",
        output
    );
}

#[test]
fn parse_compile_1() {
    let input_file = PathBuf::from("./resources/compile-page.arct");
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(Context::default(), input_file, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output = String::from_utf8(output).unwrap();

    assert_eq!(
        concat!(
            "Compile this: Below are the paragraphs.\n",
            "This is some text here.\n",
            "And some more text.\n",
            "Here is another paragraph. Blah."
        ),
        &output
    );
}

#[test]
fn parse_for_multiple() {
    let mut output = Vec::<u8>::new();
    let mut input = String::new();
    input.push_str(r#"Some text, and a tag: \
        {% set users %}test.user{% /set %}\
        {% set users %}second.user{% /set %}\
        {% foreach user in users %}|{{ user }}{% /foreach %}"#);
    let mut parser = TemplateParser::new(Context::default(), input.as_str(), &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        "Some text, and a tag: |test.user|second.user",
        output_str,
    );
}

#[test]
fn parse_for_none_else() {
    let mut output = Vec::<u8>::new();
    let mut input = String::new();
    input.push_str(r#"Some text, and a tag: \
        {% foreach user in users %}|{{ user }}{% else %}No users.{%/ foreach %}\
    "#);
    let mut parser = TemplateParser::new(Context::default(), input.as_str(), &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        "Some text, and a tag: No users.",
        output_str,
    );
}

#[test]
fn parse_for_none_no_else() {
    let mut output = Vec::<u8>::new();
    let mut input = String::new();
    input.push_str(r#"Some text, and a tag: \
        {% foreach user in users %}|{{ user.username }}{%/ foreach %}\
    "#);
    let mut parser = TemplateParser::new(Context::default(), input.as_str(), &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        "Some text, and a tag: ",
        output_str,
    );
}

#[test]
#[should_panic]
fn parse_assert_1() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id", "./", "1");
    let input = r#"{% assert id == "0" /%}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
}

#[test]
fn parse_assert_2() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id", "./", "1");
    let input = r#"{% assert id == "1" /%}True"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

#[test]
fn parse_if_inline_1() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id", "./", "1");
    let input = r#"{% if id == "0" %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("False", output_str);
}

#[test]
fn parse_if_inline_2() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id", "./", "2");
    context.add_variable("id2", "./", "5");
    let input = r#"{% if id == "1" || id2 > "4" %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

#[test]
fn parse_if_inline_3() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id1", "./", "2");
    context.add_variable("id2", "./", "5");
    context.add_variable("id3", "./", "10");
    // str(10) < str(2.5)
    let input = r#"{% if (id1 == "0" || id2 > "4") && "2.5" < id3 %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("False", output_str);
}

#[test]
fn parse_if_inline_4() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id1", "./", "2");
    context.add_variable("id2", "./", "5");
    context.add_variable("id3", "./", "10");
    let input = r#"{% if id1 == "2" || id2 < "4" || id3 < "2.5" %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

// should evaluate identically to 4
#[test]
fn parse_if_inline_5() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id1", "./", "2");
    context.add_variable("id2", "./", "5");
    context.add_variable("id3", "./", "10");
    let input = r#"{% if (id1 == "2" || id2 < "4") || id3 < "2.5" %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

#[test]
fn parse_if_truthy_1() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id", "./", "2");
    let input = r#"{% if id %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

#[test]
fn parse_if_truthy_2() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("id", "./", "0");
    let input = r#"{% if id %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("False", output_str);
}

#[test]
fn parse_if_truthy_3() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("a", "./", "1");
    context.add_variable("b", "./", "0");
    let input = r#"{% if b || a %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

// should evaluate to the same as 3
#[test]
fn parse_if_truthy_4() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("a", "./", "1");
    context.add_variable("b", "./", "0");
    let input = r#"{% if (b || a) %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

#[test]
fn parse_if_mixed_1() {
    let mut output = Vec::<u8>::new();
    let mut context = Context::default();
    context.add_variable("a", "./", "1");
    context.add_variable("b", "./", "0");
    let input = r#"{% if (b || (b == "0" && a)) %}True{% else %}False{% /if %}"#;
    let mut parser = TemplateParser::new(context, input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("True", output_str);
}

#[test]
fn nested_parse_if_and_for() {
    let mut output = Vec::<u8>::new();
    let mut input = String::new();
    input.push_str(r#"Some text, and a tag: \
    {% set users %}test.user{% /set %}\
    {% set users %}second.user{% /set %}\
    {% foreach user in users as loop %}\
        {% if loop.isfirst %}{% else %}, {% /if %}\
        {{ user }}\
        {% if loop.islast %}.{% /if %}\
    {% else %}\
        No users.\
    {%/ foreach %}"#);
    let mut parser = TemplateParser::new(Context::default(), input.as_str(), &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        "Some text, and a tag: test.user, second.user.",
        output_str,
    );
}

#[test]
fn parse_complex_1() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_complex_1/page.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        concat!(
            "TEMPLATE START\n",
            "Mark is 33 year(s) old.\n",
            "Fred is 31 year(s) old.\n",
            "Karissa is 27 year(s) old.\n",
            "TEMPLATE END\n",
        ),
        output_str
    );
}

#[test]
fn parse_complex_2() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_complex_2/page.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    let split = parser.context().unwrap().value(&"split");
    assert_eq!(None, split);
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("<h1><span>The H</span><span>eader</span></h1>", output_str);
}

#[test]
#[should_panic]
fn unexpected_eof() {
    let mut output = Vec::<u8>::new();
    let input = r#"{% let str1 = "First section"#;
    let mut parser = TemplateParser::new(Context::default(), input, &mut output).unwrap();
    parser.parse().unwrap();
    drop(parser);
}

#[test]
fn parse_call_1() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_call_1/page.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output = String::from_utf8(output).unwrap();
    assert_eq!(
        "1\n", output
    );
}

#[test]
fn parse_complex_3() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_complex_3/page.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("name: Fred\nage: 31\nchildren: ", output_str);
}

#[test]
fn parse_complex_4() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_complex_4/page.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("#ffb4a1", output_str);
}

#[test]
fn call_compile_chain_1() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/call_compile_chain_1/page.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("1", output_str);
}

#[test]
fn parse_complex_5() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_complex_5/person/fred.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        concat!(
            "<h1 class=\"lead split\"><span>Fred</span></h1>\n",
            "<div class=\"child\">\n",
            "\t<table class=\"lr\">\n",
            "\t<tbody>\n",
            "\t\t<tr>\n",
            "\t\t\t<td>Child</td>\n",
            "\t\t\t<td><strong>Fred Jr</strong></td>\n",
            "\t\t</tr>\n",
            "\t</tbody>\n",
            "</table>\n",
            "\t</div>\n",
            "<div class=\"child\">\n",
            "\t<table class=\"lr\">\n",
            "\t<tbody>\n",
            "\t\t<tr>\n",
            "\t\t\t<td>Child</td>\n",
            "\t\t\t<td><strong>Fred II</strong></td>\n",
            "\t\t</tr>\n",
            "\t</tbody>\n",
            "</table>\n",
            "\t</div>\n",
        ),
        output_str
    );
}

#[test]
fn parse_complex_6() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_complex_6/allsiblings.arct"),
        &mut output
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);

    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(
        include_str!("../../../resources/parse_complex_6/against.txt"),
        output_str
    );
}

#[test]
fn parse_function_1() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_function_1/page.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("one, two, three", output_str);
}

#[test]
fn parse_function_2() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_function_2/page.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("One, Two, Third\nFourth, Fifth\n", output_str);
}

#[test]
fn parse_function_3() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_function_3/test.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!("../test.txt", output_str);
}

#[test]
fn parse_nth_1() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_1/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Second", output_str);
}

#[test]
fn parse_nth_2() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_2/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Second", output_str);
}

#[test]
#[should_panic]
fn parse_nth_3() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_3/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
}

#[test]
fn parse_nth_4() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_4/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("First", output_str);
}

#[test]
fn parse_nth_5() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_5/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Fifth", output_str);
}

#[test]
fn parse_nth_6() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_6/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Fourth", output_str);
}

#[test]
fn parse_nth_7() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_7/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Fourth", output_str);
}

#[test]
fn parse_nth_8() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_nth_8/file.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Third", output_str);
}

#[test]
fn parse_path_1() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_path_1/page.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Fred", output_str);
}

#[test]
fn parse_path_2() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_path_2/item.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("Fred is 31 year(s) old.", output_str);
}

#[test]
fn parse_path_3() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_path_3/page.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("File", output_str);
}

#[test]
fn parse_path_4() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_path_4/page.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("File", output_str);
}

#[test]
fn parse_path_5() {
    let mut output = Vec::<u8>::new();
    let mut parser = TemplateParser::new(
        Context::default(),
        PathBuf::from("./resources/parse_path_5/page.arct"),
        &mut output,
    ).unwrap();
    parser.parse().unwrap();
    drop(parser);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!("File", output_str);
}

#[test]
fn parse_assert_eq_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/assert_not_eq_1/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();

	assert_eq!(
		include_str!("../../../resources/assert_not_eq_1/against.txt"),
		output_str
	);
}

#[test]
fn parse_add_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_add_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("5", output_str);
}

#[test]
fn parse_add_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_add_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("4", output_str);
}

#[test]
fn parse_add_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_add_3/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("4", output_str);
}

#[test]
fn parse_sub_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_sub_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("-5", output_str);
}

#[test]
fn parse_sub_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_sub_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("1", output_str);
}

#[test]
fn parse_sub_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_sub_3/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("1", output_str);
}

#[test]
fn parse_mul_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_mul_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("0", output_str);
}

#[test]
fn parse_mul_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_mul_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("38", output_str);
}

#[test]
fn parse_mul_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_mul_3/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("38", output_str);
}

#[test]
fn parse_div_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_div_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("1", output_str);
}

#[test]
fn parse_div_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_div_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("3", output_str);
}

#[test]
fn parse_div_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_div_3/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("3", output_str);
}

#[test]
fn parse_mod_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_mod_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("5", output_str);
}

#[test]
fn parse_mod_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_mod_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("0", output_str);
}

#[test]
fn parse_mod_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_mod_3/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("0", output_str);
}

#[test]
fn parse_pow_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_pow_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("25", output_str);
}

#[test]
fn parse_pow_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_pow_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("27", output_str);
}

#[test]
fn parse_pow_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_pow_3/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("27", output_str);
}

#[test]
fn parse_math_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_math_1/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("99", output_str);
}

#[test]
fn parse_math_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_math_2/file.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("99", output_str);
}

#[test]
fn parse_fordir_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_fordir_1/loop.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        concat!(
            "./resources/parse_fordir_1/./4, ",
            "./resources/parse_fordir_1/./3, ",
            "./resources/parse_fordir_1/./2, ",
            "./resources/parse_fordir_1/./1, ",
            "./resources/parse_fordir_1/./0",
        ),
        output_str
    );
}

#[test]
fn parse_fordir_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_fordir_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        concat!(
            "./resources/parse_fordir_2/./items/2, ",
            "./resources/parse_fordir_2/./items/1",
        ),
        output_str
    );
}

#[test]
fn parse_foreach_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_foreach_1/loop.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("4, 3, 2, 1, 0", output_str);
}

#[test]
fn parse_forfile_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forfile_1/loop.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        concat!(
            "./resources/parse_forfile_1/./files/4, ",
            "./resources/parse_forfile_1/./files/3, ",
            "./resources/parse_forfile_1/./files/2, ",
            "./resources/parse_forfile_1/./files/1, ",
            "./resources/parse_forfile_1/./files/0",
        ),
        output_str
    );
}

#[test]
fn parse_if_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_1/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "./resources/parse_if_1/./testdir/5",
        output_str
    );
}

#[test]
fn parse_if_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_2/if.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "0",
        output_str
    );
}

#[test]
fn parse_if_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_3/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "true",
        output_str
    );
}

#[test]
fn parse_if_4() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_4/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "false",
        output_str
    );
}

#[test]
fn parse_if_5() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_5/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "true",
        output_str
    );
}

#[test]
fn parse_if_6() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_6/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "Mark, Fred, and Karissa.",
        output_str
    );
}

#[test]
fn parse_if_7() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_7/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("Y", output_str);
}

#[test]
fn parse_if_8() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_if_8/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("true", output_str);
}

#[test]
fn parse_fn_call_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
        concat!(
            "{% set fnpath %}{% path \"./resources/parse_fn_call_1/functions/testobj.arct\" /%}{% /set %}",
            "{% call fnpath /%}",
            "{{ test.pagesize() }}\\",
        ),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "10",
        output_str
    );
}

#[test]
fn parse_fn_call_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_fn_call_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "<h4 id=\"the_id\">Some text</h4>",
        output_str
    );
}

#[test]
fn parse_forsplit_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forsplit_1/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "0, 1, 2, 3, 4, 5",
        output_str
    );
}

#[test]
fn parse_forsplit_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forsplit_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "5+4+3+2+1+0",
        output_str
    );
}

#[test]
fn parse_forsplit_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forsplit_3/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "iii",
        output_str
    );
}

#[test]
fn parse_forsplit_4() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forsplit_4/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "No items.",
        output_str
    );
}

#[test]
fn parse_forsplit_5() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forsplit_5/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "No items.",
        output_str
    );
}

#[test]
fn parse_forsplit_6() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_forsplit_6/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
	drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("1, 2", output_str);
}

#[test]
#[should_panic]
fn parse_assert_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_assert_3/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
}

#[test]
fn parse_assert_4() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_assert_4/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "Passed",
        output_str
    );
}

#[test]
fn parse_assert_5() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_assert_5/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "Passed",
        output_str
    );
}

#[test]
fn parse_assert_6() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_assert_6/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("true", output_str);
}

#[test]
fn parse_foreach_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_foreach_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "3, 2, 1",
        output_str
    );
}

#[test]
fn parse_foreach_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_foreach_3/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "3, 2, 1, 0",
        output_str
    );
}

#[test]
fn parse_foreach_4() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_foreach_4/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "6, 5, 4, 3, 2, 1",
        output_str
    );
}

#[test]
fn parse_number_literal_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_number_literal_1/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!(
        "2",
        output_str
    );
}

#[test]
#[should_panic]
fn parse_number_literal_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_number_literal_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
}

#[test]
fn parse_length_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_length_1/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("16", output_str);
}

#[test]
fn parse_length_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_length_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("16", output_str);
}

#[test]
fn parse_length_3() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_length_3/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("6", output_str);
}

#[test]
fn parse_count_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_count_1/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("1", output_str);
}

#[test]
fn parse_count_2() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_count_2/test.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("3", output_str);
}

#[test]
fn parse_dirname_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_dirname_1/page.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("parse_dirname_1", output_str);
}

#[test]
fn parse_basename_1() {
	let mut output = Vec::<u8>::new();
	let mut parser = TemplateParser::new(
		Context::default(),
		PathBuf::from("./resources/parse_basename_1/page.arct"),
		&mut output,
	).unwrap();
	parser.parse().unwrap();
    drop(parser);
	let output_str = String::from_utf8(output).unwrap();
	assert_eq!("page.arct", output_str);
}
