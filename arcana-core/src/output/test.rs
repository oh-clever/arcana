use crate::output::Output;

// proof that char to u8 is not utf-8 compliant
#[test]
#[should_panic]
fn low_level_utf_1() {
    let c: char = 'ń';
    let mut source = Vec::<u8>::new();
    source.push(c as u8);
    assert_eq!('ń', source.into_iter().next().unwrap() as char);
}

#[test]
fn low_level_utf_2() {
    // a utf-8 character
    let c: char = 'ń';
    // a 4 byte vec, the max size for a utf-8 character
    let mut source = Vec::from([0x0, 0x0, 0x0, 0x0]);
    c.encode_utf8(&mut source);
    assert_eq!('ń', String::from_utf8(source).unwrap().chars().next().unwrap());
}

#[test]
fn write_utf8_1() {
    let mut destination = Vec::<u8>::new();
    let mut output = Output::from(&mut destination);
    output.write_char('ń');
    output.flush_buffer_to_content();
    let content = output.take_content();
    let taken = String::from_utf8(content).unwrap();
    assert_eq!('ń', taken.chars().next().unwrap());
}
