pub mod structs {
    #[derive(Debug, Eq, PartialEq)]
    pub enum Reply<'a, O> {
        Ok(O, &'a [u8]),
        Err(&'a [u8])
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum Consumed<'a, O> {
        Consumed(Reply<'a, O>),
        Empty(Reply<'a, O>),
    }
}

pub mod operators {
    use super::structs::{Consumed, Reply};

    impl <'a, A> Consumed<'a, A> {
        pub fn bind<F, B> (self, f: F) -> Consumed<'a, B>
            where F:FnOnce(A, &'a [u8]) -> Consumed<'a, B> {
            
            use super::structs::Consumed::{Empty, Consumed};
            use super::structs::Reply::{Ok, Err};

            match self {
                Empty(reply) => match reply {
                    Ok(o, i) => f(o, i),
                    Err(i)   => Empty(Err(i)),
                },
                Consumed(reply) => Consumed(match reply {
                    Ok(o, i) => match f(o, i) {
                        Consumed(reply2) => reply2,
                        Empty(reply2)    => reply2,
                    },
                    Err(i)  => Err(i)
                })
            } 
        }

        pub fn choice<F> (self, f:F) -> Consumed<'a, A>
            where F:FnOnce(&'a [u8]) -> Consumed<'a, A> {
            
            use super::structs::Consumed::{Empty, Consumed};
            use super::structs::Reply::{Ok, Err};

            match self {
                Empty(Err(i))   => f(i),
                Empty(Ok(o, i)) => match f(i) {
                    Empty(Ok(_, i)) => Empty(Ok(o, i)),
                    consumed @ _    => consumed,
                },
                consumed @ _ => consumed
            }
        }
    }
}

pub mod predicates {
    #[inline]
    pub fn is_alpha(b: u8) -> bool { (b as char).is_alphabetic() }

    #[inline]
    pub fn is_digit(b: u8) -> bool { (b as char).is_digit(10) }
}

pub mod parsers {
    use std::fmt::Debug;
    use super::structs::Consumed;
    use super::structs::Reply;
    use super::predicates::*;

    #[inline]
    pub fn satisfy<'a, F>(f: F, input: &'a [u8]) -> Consumed<'a, u8> 
        where F: FnOnce(u8) -> bool {

        use super::structs::Consumed::{Empty, Consumed};
        use super::structs::Reply::{Ok, Err};

        match input.first() {
            None    => Empty(Err(input)),
            Some(b) => match f(*b) {
                true  => Consumed(Ok(*b, &input[1..])),
                false => Empty(Err(input)),
            }
        }
    }

    #[inline]
    pub fn many<'a, P, O>(p: P, i: &'a [u8]) -> Consumed<'a, Vec<O>>
        where P: Fn(&[u8]) -> Consumed<O>,
              O: Debug {

        use super::structs::Consumed::{Empty, Consumed};
        use super::structs::Reply::{Ok, Err};

        let mut matches = vec![];
        let mut changing_input = i;

        loop {
            match p(changing_input) {
                Empty(Ok(_, _)) => continue,
                Consumed(Ok(res, i)) => {
                    matches.push(res);
                    changing_input = i;
                },
                Consumed(Err(i)) => match matches.len() {
                    0 => return Consumed(Err(i)),
                    _ => return Consumed(Ok(matches, i))
                },
                Empty(Err(i)) => match matches.len() {
                    0 => return Empty(Err(i)),
                    _ => return Consumed(Ok(matches, i)),
                },
            }
        } 
    }

    #[inline]
    pub fn digit<'a>(i: &'a [u8]) -> Consumed<'a, u8> {
        satisfy(is_digit, i)
    }

    #[inline]
    pub fn character<'a>(i: &'a [u8]) -> Consumed<'a, u8> {
        satisfy(is_alpha, i)
    }

    #[inline]
    pub fn many_char<'a>(i: &'a [u8]) -> Consumed<'a, Vec<u8>> {
        many(character, i)
    }

    #[inline]
    pub fn many_digit<'a>(i: &'a [u8]) -> Consumed<'a, Vec<u8>> {
        many(digit, i)
    }
}
