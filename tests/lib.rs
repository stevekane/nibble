extern crate nibble;

use nibble::structs::{Consumed, Reply};
use nibble::operators::*;
use nibble::parsers::*;

// desired do-block macro syntax for composition of parsers
// parser! text_parser<'a>(&'a [u8]) -> Consumed<'a, (u8, u8, u8)> {
//     d1 <- digit
//     d2 <- digit
//     d3 <- character | digit
//     return (d1, d2, d3)
// }
//
// ideally the syntax below should be used and the lifetime annotations and 
// surrounding return-type boilerplate could be added by the macro
// parser! test_parser -> (u8, u8, u8) { //.... } 

fn test_parser<'a>(i: &'a [u8]) -> Consumed<'a, (u8, u8, u8)> {
    digit(i).bind(|d1, i|
    digit(i).bind(|d2, i| 
    character(i).choice(digit).bind(|d3, i|
    Consumed::Consumed(Reply::Ok((d1, d2, d3), i)))))
}

#[test]
fn test_do_block() {
    let str1 = String::from("123145skfjhalb1");
    let str2 = String::from("12a145skfjhalb1");
    let input1 = str1.as_bytes();
    let input2 = str2.as_bytes();
    let result1 = test_parser(input1);
    let result2 = test_parser(input2);

    println!("{:?}", result1);
    println!("{:?}", result2);
}

#[test]
fn test_many() {
    let string = String::from("asdabfkjasbf123145skfjhalb1");
    let input = string.as_bytes();
    let result = many_char(input).bind(|chars, i| 
                 many_digit(i).bind(|digits, i| 
                 Consumed::Consumed(Reply::Ok((chars, digits), i))));

    println!("{:?}", result);
}

#[test]
fn test_many_complex() {
    let str = String::from("123145skfjhalb1");
    let input = str.as_bytes();
    let result = many(test_parser, input);

    println!("{:?}", result);
}
