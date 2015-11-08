use std::fmt;

pub enum Reply<'a, O> {
    Ok(O, &'a [u8]),
    Err(&'a [u8])
}

pub enum Consumed<'a, O> {
    Consumed(Reply<'a, O>),
    Empty(Reply<'a, O>),
}

pub type Parser<'a, O> = Fn(&[u8]) -> Consumed<'a, O>;

impl <'a, O:fmt::Debug> fmt::Debug for Reply<'a, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Reply::Ok(ref t, _) => write!(f, "Ok({:?}, Iterator)", t),
            &Reply::Err(_)       => write!(f, "Err"),
        } 
    }
}

impl <'a, O:fmt::Debug> fmt::Debug for Consumed<'a, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Consumed::Consumed(ref reply) => write!(f, "Consumed({:?})", reply),
            &Consumed::Empty(ref reply)    => write!(f, "Empty({:?})", reply),
        } 
    }
}

impl <'a, A> Consumed<'a, A> {
    pub fn bind<F, B> (self, f: F) -> Consumed<'a, B>
        where F:FnOnce(A, &'a [u8]) -> Consumed<'a, B> {
        
        use Consumed::{Empty, Consumed};
        use Reply::{Ok, Err};

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
        
        use Consumed::Empty;
        use Reply::{Ok, Err};

        match self {
            Empty(Err(i))   => f(i),
            Empty(Ok(o, i)) => match f(i) {
                Empty(Ok(_, i)) => Empty(Ok(o, i)),
                consumed @ _ => consumed,
            },
            consumed @ _ => consumed
        }
    }
}

#[inline]
pub fn is_alpha(b: u8) -> bool { (b as char).is_alphabetic() }

#[inline]
pub fn is_digit(b: u8) -> bool { (b as char).is_digit(10) }

#[inline]
pub fn satisfy<'a, F>(f: F, input: &'a [u8]) -> Consumed<'a, u8> 
    where F: FnOnce(u8) -> bool {
    match input.first() {
        None    => Consumed::Empty(Reply::Err(input)),
        Some(b) => match f(*b) {
            true  => Consumed::Consumed(Reply::Ok(*b, &input[1..])),
            false => Consumed::Empty(Reply::Err(input)),
        }
    }
}

#[inline]
fn many<'a, P, O>(p: P, i: &'a [u8]) -> Consumed<'a, Vec<O>>
    where P: Fn(&[u8]) -> Consumed<O> {
    use Consumed::{Consumed, Empty};
    use Reply::{Ok, Err};

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

fn test_parser<'a>(i: &'a [u8]) -> Consumed<'a, (u8, u8, u8)> {
    digit(i).bind(|d1, i|
    digit(i).bind(|d2, i| 
    character(i).choice(digit).bind(|d3, i|
    Consumed::Consumed(Reply::Ok((d1, d2, d3), i)))))
}

// parser! text_parser<&[u8], O> {
//     d1 <- digit
//     d2 <- digit
//     d3 <- character | digit
//     return (d1, d2, d3)
// }


#[cfg(test)]
mod tests {
    use super::{test_parser, many, character, digit, many_digit, many_char};
    use super::Consumed::Consumed;
    use super::Reply::Ok;

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
                     Consumed(Ok((chars, digits), i))));
    
        println!("{:?}", result);
    }

    #[test]
    fn test_many_complex() {
        let str = String::from("123145skfjhalb1");
        let input = str.as_bytes();
        let result = many(test_parser, input);

        println!("{:?}", result);
    }
}
