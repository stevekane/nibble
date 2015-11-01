use std::fmt;

pub enum Reply<I:Iterator<Item=u8>, O> {
    Ok(O, I),
    Err(I)
}

pub enum Consumed<I:Iterator<Item=u8>, O> {
    Consumed(Reply<I, O>),
    Empty(Reply<I, O>),
}

pub type Parser<I, O> = Fn(I) -> Consumed<I, O>;

impl <I:Iterator<Item=u8>, O:fmt::Debug> fmt::Debug for Reply<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Reply::Ok(ref t, _) => write!(f, "Ok({:?}, Iterator)", t),
            &Reply::Err(_)       => write!(f, "Err"),
        } 
    }
}

impl <I:Iterator<Item=u8>, O:fmt::Debug> fmt::Debug for Consumed<I, O> {
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

pub fn digit<I>(mut input: I) -> Consumed<I, u8> 
    where I:Iterator<Item=u8> {

    match input.next() {
        None    => Consumed::Empty(Reply::Err(input)),
        Some(b) => match (b as char).is_digit(10) {
            true  => Consumed::Consumed(Reply::Ok(b, input)),
            false => Consumed::Empty(Reply::Err(input)),
        }
    }
}

pub fn char<I>(mut input: I) -> Consumed<I, u8>
    where I:Iterator<Item=u8> {

    match input.next() {
        None    => Consumed::Empty(Reply::Err(input)),
        Some(b) => match (b as char).is_alphabetic() {
            true  => Consumed::Consumed(Reply::Ok(b, input)),
            false => Consumed::Empty(Reply::Err(input)),
        }
    }
}

impl <I:Iterator<Item=u8>, A> Consumed<I, A> {
    pub fn bind<F, B> (self, f: F) -> Consumed<I, B>
        where F:FnOnce(A, I) -> Consumed<I, B> {
        
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

    //TODO: consider switching to consume a fixed buffer instead of iterator
    //this solves some complexities with look ahead on iterators
    pub fn choice<F> (self, f:F) -> Consumed<I, A>
        where F:FnOnce(I) -> Consumed<I, A> {
        
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

// parser! first_two<I:Iterator<Item=u8>, O> {
//     d1 <- digit
//     d2 <- digit
//     d3 <- char | digit
//     return (d1, d2, d3)
// }

fn test_parser<I:Iterator<Item=u8>> (i: I) -> Consumed<I, (u8, u8, u8)> {
    digit(i).bind(|d1, i|
    digit(i).bind(|d2, i| 
    char(i).choice(digit).bind(|d3, i|
    Consumed::Consumed(Reply::Ok((d1, d2, d3), i)))))
}

#[cfg(test)]
mod tests {
    use super::test_parser;

    #[test]
    fn test_do_block() {
        let string = String::from("123145skfjhalb1");
        let bytes = string.bytes();
        let result = test_parser(bytes);

        println!("{:?}", result);
    }
}
