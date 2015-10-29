use std::fmt;

pub fn is_alpha(b: u8) -> bool { (b as char).is_alphabetic() }
pub fn is_digit(b: u8) -> bool { (b as char).is_digit(10) }

pub enum Reply<I:Iterator, O> {
    Ok(O, I),
    Err
}

pub enum Consumed<I:Iterator, O> {
    Consumed(Reply<I, O>),
    Empty(Reply<I, O>),
}

impl <I:Iterator, O:fmt::Debug> fmt::Debug for Reply<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Reply::Ok(ref t, _) => write!(f, "Ok({:?}, Iterator)", t),
            &Reply::Err          => write!(f, "Err"),
        } 
    }
}

impl <I:Iterator, O:fmt::Debug> fmt::Debug for Consumed<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Consumed::Consumed(ref reply) => write!(f, "Consumed({:?})", reply),
            &Consumed::Empty(ref reply)    => write!(f, "Empty({:?})", reply),
        } 
    }
}

pub fn unit<I, O> (t: O, input: I) -> Consumed<I, O> 
    where I:Iterator {
    use Consumed::Consumed;
    use Reply::Ok;

    Consumed(Ok(t, input)) 
}

pub fn satisfy<I, F> (f: F, mut input: I) -> Consumed<I, I::Item>
    where I:Iterator,
          I::Item:Copy,
          F:FnOnce(I::Item) -> bool {
    use Consumed::{Empty, Consumed};
    use Reply::{Ok, Err};

    match input.next() {
        None    => Empty(Err),
        Some(b) => match f(b) {
            true  => Consumed(Ok(b, input)),
            false => Empty(Err),
        }
    }
}

pub fn digit<I>(mut input: I) -> Consumed<I, u8> {
    match input.next() {
        None    => Empty(Err),
        Some(b) => match is_digit(b) {
            true  => Consumed(Ok(b, input)),
            false => Empty(Err),
        }
    }
}

pub fn bind<I, O, T, F>(c: Consumed<I, O>, f: F) -> Consumed<I, T>
    where I:Iterator,
          F:FnOnce(O, I) -> Consumed<I, T> {
    use Consumed::{Empty, Consumed};
    use Reply::{Ok, Err};

    match c {
        Empty(reply) => match reply {
            Ok(o, i) => f(o, i),
            Err      => Empty(Err),
        },
        Consumed(reply) => Consumed(match reply {
            Ok(o, i) => match f(o, i) {
                Consumed(reply2) => reply2,
                Empty(reply2)    => reply2,
            },
            Err      => Err,
        })
    } 
}

//pub fn choice<I, O, F>(c: Consumed<I, O>, f:F) -> Consumed<I, O>
//    where I:Iterator,
//          F:FnOnce(O, I) -> Consumed<I, O> {
//    use Consumed::{Empty, Consumed};
//    use Reply::{Ok, Err};
//
//    match c {
//        Empty(Err) => f
//    }
//}

#[test]
fn test_unit() {
    let string = String::from("asfkja1231241");
    let bytes = string.bytes();
    let result = unit('a', bytes);

    println!("{:?}", result);
}

#[test]
fn test_satisfy() {
    let string = String::from("asfkja1231241");
    let bytes = string.bytes();
    let result = satisfy(is_alpha, bytes);

    println!("{:?}", result);
}

#[test]
fn test_bind() {
    let string = String::from("asfkja1231241");
    let bytes = string.bytes();
    let result = bind(satisfy(is_alpha, bytes), |o, i| {
                 bind(satisfy(is_alpha, i), |o2, i| {
                 bind(satisfy(|c| c as char == 'f', i), |o3, i| {
                 Consumed::Consumed(Reply::Ok(vec![o, o2, o3], i))})})});

    println!("{:?}", result);
}
