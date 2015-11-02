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

#[inline]
pub fn is_alpha(b: u8) -> bool { (b as char).is_alphabetic() }
#[inline]
pub fn is_digit(b: u8) -> bool { (b as char).is_digit(10) }

pub fn digit<'a>(input: &'a [u8]) -> Consumed<'a, u8> {
    match input.first() {
        None    => Consumed::Empty(Reply::Err(input)),
        Some(b) => match (*b as char).is_digit(10) {
            true  => Consumed::Consumed(Reply::Ok(*b, &input[1..])),
            false => Consumed::Empty(Reply::Err(input)),
        }
    }
}

pub fn char<'a>(input: &'a [u8]) -> Consumed<'a, u8> {
    match input.first() {
        None    => Consumed::Empty(Reply::Err(input)),
        Some(b) => match (*b as char).is_alphabetic() {
            true  => Consumed::Consumed(Reply::Ok(*b, &input[1..])),
            false => Consumed::Empty(Reply::Err(input)),
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

// parser! first_two<&[u8], O> {
//     d1 <- digit
//     d2 <- digit
//     d3 <- char | digit
//     return (d1, d2, d3)
// }

fn many_chars<'a>(i: &'a [u8]) -> Consumed<'a, &[u8]> {
    use Consumed::{Consumed, Empty};
    use Reply::{Ok, Err};
    
    //char(i).bind(|x, i|
    //many_chars(i).choice(|i| Consumed(Ok(&[], i)))).bind(|xs, i|
    //Consumed(Ok(xs, i)))
    char(i).bind(|x, i|
    Consumed(Ok(&[x], i)))
}

fn test_parser<'a>(i: &'a [u8]) -> Consumed<'a, (u8, u8, u8)> {
    digit(i).bind(|d1, i|
    digit(i).bind(|d2, i| 
    char(i).choice(|i| digit(i)).bind(|d3, i|
    Consumed::Consumed(Reply::Ok((d1, d2, d3), i)))))
}

#[cfg(test)]
mod tests {
    use super::{test_parser, many_chars};

    #[test]
    fn test_do_block() {
        let string = String::from("123145skfjhalb1");
        let input = string.as_bytes();
        let result = test_parser(input);

        println!("{:?}", result);
    }

    #[test]
    fn test_repeating_parser() {
        let string = String::from("asdabfkjasbf123145skfjhalb1");
        let input = string.as_bytes();
        let result = many_chars(input);
    
        println!("{:?}", result);
    }
}
