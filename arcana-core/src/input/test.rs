use crate::input::TryIntoInput;

#[test]
fn read_str_1() {
    let mut input = "This is the input.".try_into_input().unwrap();
    assert_eq!(Some(&'T'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'h'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'i'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'s'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&' '), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'i'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'s'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&' '), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'t'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'h'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'e'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&' '), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'i'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'n'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'p'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'u'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'t'), input.current());

    input.step().unwrap();
    assert_eq!(Some(&'.'), input.current());

    input.step().unwrap();
    assert_eq!(None, input.current());
    assert!(input.is_end());
}
