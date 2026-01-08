#[cfg(test)]
mod test {
    use {
        crate::{
            context::Context,
            parser::{ TemplateParser, steps::Steps, },
        },
        std::path::PathBuf,
    };

    fn str_parser<'p>(input: &'p str, output: &'p mut Vec<u8>) -> TemplateParser<&'p [u8], &'p mut Vec<u8>> {
        TemplateParser::new(
            Context::default(),
            input,
            output
        ).unwrap()
    }

    #[test]
    fn buffer_all_while() {
        let mut output = Vec::new();
        let mut parser = str_parser("abc$", &mut output);
        parser.buffer_all_while(|c| matches!(c, 'a'|'b'|'c')).unwrap();
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_rest_of_tagname() {
        let mut output = Vec::new();
        let mut parser = str_parser("compile^", &mut output);
        parser.buffer_rest_of_tagname().unwrap();
        assert_eq!(Some(&'^'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn bypass_whitespace() {
        let mut output = Vec::new();
        let mut parser = str_parser("        x", &mut output);
        parser.bypass_whitespace().unwrap();
        assert_eq!(Some(&'x'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_whitespace() {
        let mut output = Vec::new();
        let mut parser = str_parser("        \n x", &mut output);
        parser.buffer_whitespace().unwrap();
        assert_eq!(Some(&'x'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_whitespace_enforce_one_1() {
        let mut output = Vec::new();
        let mut parser = str_parser("        \n x", &mut output);
        assert!(parser.buffer_whitespace_enforce_one().unwrap());
        assert_eq!(Some(&'x'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_whitespace_enforce_one_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("x", &mut output);
        assert!(!parser.buffer_whitespace_enforce_one().unwrap());
        assert_eq!(Some(&'x'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn bypass_whitespace_enforce_one_1() {
        let mut output = Vec::new();
        let mut parser = str_parser("        \n x", &mut output);
        assert!(parser.bypass_whitespace_enforce_one().unwrap());
        assert_eq!(Some(&'x'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn bypass_whitespace_enforce_one_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("x", &mut output);
        assert!(!parser.bypass_whitespace_enforce_one().unwrap());
        assert_eq!(Some(&'x'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_all_until() {
        let mut output = Vec::new();
        let mut parser = str_parser("abcdef_x\n\t$", &mut output);
        parser.buffer_all_until(|c| matches!(c, '$')).unwrap();
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_all_until_sequence_1() {
        let mut output = Vec::new();
        let mut parser = str_parser("this that the other \\/%}/%}", &mut output);
        parser.buffer_all_until_sequence("test", &['/', '%', '}']).unwrap();
        // should buffer until end
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn buffer_all_until_sequence_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("this that the other /%}/%}", &mut output);
        parser.buffer_all_until_sequence("test", &['/', '%', '}']).unwrap();
        assert_eq!(Some(&'/'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn bypass_all_until() {
        let mut output = Vec::new();
        let mut parser = str_parser("abcdef_x\n\t$", &mut output);
        parser.bypass_all_until(|c| matches!(c, '$')).unwrap();
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn tag_expect_char_1() {
        let mut output = Vec::new();
        let mut parser = str_parser("a", &mut output);
        parser.tag_expect_char("test", |c| matches!(c, 'a')).unwrap();
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn tag_expect_char_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("$", &mut output);
        match parser.tag_expect_char("test", |c| matches!(c, 'a')) {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!("Unexpected character in tag 'test'", e.message()),
            },
        }
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn tag_expect_char_3() {
        let mut output = Vec::new();
        let mut parser = str_parser("", &mut output);
        match parser.tag_expect_char("test", |c| matches!(c, 'a')) {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!(e.message(), "Unexpected EOF in tag 'test'"),
            },
        }
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn end_tag_expect_char_1() {
        let mut output = Vec::new();
        let mut parser = str_parser("a", &mut output);
        parser.end_tag_expect_char("test", |c| matches!(c, 'a')).unwrap();
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn end_tag_expect_char_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("$", &mut output);
        match parser.end_tag_expect_char("test", |c| matches!(c, 'a')) {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!("Unexpected character in end-tag 'test'", e.message()),
            },
        }
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn end_tag_expect_char_3() {
        let mut output = Vec::new();
        let mut parser = str_parser("", &mut output);
        match parser.end_tag_expect_char("test", |c| matches!(c, 'a')) {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!(e.message(), "Unexpected EOF in end-tag 'test'"),
            },
        }
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn expect_end_of_self_close_tag_1() {
        let mut output = Vec::new();
        let mut parser = str_parser(" /%}", &mut output);
        parser.expect_end_of_self_close_tag("test").unwrap();
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn expect_end_of_self_close_tag_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("$", &mut output);
        match parser.expect_end_of_self_close_tag("test") {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!("Unexpected character in tag 'test'", e.message()),
            },
        }
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn expect_end_of_self_close_tag_3() {
        let mut output = Vec::new();
        let mut parser = str_parser("", &mut output);
        match parser.expect_end_of_self_close_tag("test") {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!(e.message(), "Unexpected EOF in tag 'test'"),
            },
        }
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn expect_end_of_tag_1() {
        let mut output = Vec::new();
        let mut parser = str_parser(" %}", &mut output);
        parser.expect_end_of_tag("test").unwrap();
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn expect_end_of_tag_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("$", &mut output);
        match parser.expect_end_of_tag("test") {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!("Unexpected character in tag 'test'", e.message()),
            },
        }
        assert_eq!(Some(&'$'), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn expect_end_of_tag_3() {
        let mut output = Vec::new();
        let mut parser = str_parser("", &mut output);
        match parser.expect_end_of_tag("test") {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!(e.message(), "Unexpected EOF in tag 'test'"),
            },
        }
        assert_eq!(None, parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn parse_variable_name() {
        let mut output = Vec::new();
        let mut parser = str_parser("thename $^#*", &mut output);
        let variable = parser.parse_variable_name("test").unwrap();
        assert_eq!("thename", variable);
        assert_eq!(Some(&' '), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn parse_text_1() {
        let mut output = Vec::new();
        let mut parser = str_parser("\"this is a string\" ", &mut output);
        let string = parser.parse_text("test").unwrap();
        assert_eq!("this is a string", string);
        assert_eq!(Some(&' '), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn parse_text_2() {
        let mut output = Vec::new();
        let mut parser = str_parser("\"this is a string\" ", &mut output);
        let string = parser.parse_text_string("test").unwrap();
        assert_eq!(String::from("this is a string"), string);
        assert_eq!(Some(&' '), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn parse_text_3() {
        let mut output = Vec::new();
        let mut parser = str_parser(" \"this is a string\" ", &mut output);
        match parser.parse_text_string("test") {
            Ok(_) => panic!("Expected err (ok)"),
            Err(e) => match e {
                Ok(_) => panic!("Expected err (flow)"),
                Err(e) => assert_eq!("Unexpected character in tag 'test'", e.message()),
            },
        }

        assert_eq!(Some(&' '), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn parse_text_as_path() {
        let mut output = Vec::new();
        let mut parser = str_parser("\"../file.txt\" ", &mut output);
        let path = parser.parse_text_as_path("test").unwrap();
        assert_eq!(PathBuf::from("../file.txt"), path);
        assert_eq!(Some(&' '), parser.input.as_ref().and_then(|i| i.current()));
    }

    #[test]
    fn parse_call() {
        let mut output = Vec::new();
        // "call" would already be parsed out
        let mut parser = str_parser(" \"./resources/template.txt\" /%}", &mut output);
        parser.parse_call().unwrap();
    }

    #[test]
    fn parse_compile() {
        let mut output = Vec::new();
        // "call" would already be parsed out
        let mut parser = str_parser(" \"./resources/template.txt\" /%}", &mut output);
        parser.parse_compile().unwrap();
    }
}
