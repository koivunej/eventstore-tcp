extern crate hex;
extern crate eventstore_tcp;
extern crate tokio_io;
extern crate regex;
extern crate bytes;

use std::io;
use std::io::BufRead;
use std::io::Lines;
use std::mem;
use tokio_io::codec::Decoder;
use eventstore_tcp::codec::PackageCodec;
use eventstore_tcp::package::Package;
use hex::FromHex;
use regex::Regex;

fn main() {
    let stdin = io::stdin();

    println!("Exit with CTRL-C or CTRL-D");
    println!();

    try_main(&mut stdin.lock()).unwrap();
}

fn try_main<R: BufRead>(reader: &mut R) -> io::Result<()> {
    let decoder = DecodingIterator::new(reader);

    for res in decoder {
        println!();
        println!("{:?}", res?);
        println!();
    }

    Ok(())
}

#[derive(Debug)]
enum Decoding {
    /// The whole input was decoded into single package
    Full { bytes: usize, package: Package },
    /// There were unused bytes left undecoded
    Partial { unused_bytes: usize, package: Package },
    /// Not enough bytes
    Underflow,
    /// There was some error
    Failure(io::Error),
}

struct DecodingIterator<B: BufRead> {
    inner: Option<Lines<B>>,
    re: Regex,
}

impl<B: BufRead> DecodingIterator<B> {
    fn new(reader: B) -> Self {
        DecodingIterator {
            inner: Some(reader.lines()),
            re: Regex::new(r"(0x)|,|\s").unwrap(),
        }
    }
}

impl<B: BufRead> Iterator for DecodingIterator<B> {
    type Item = io::Result<Decoding>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_none() {
            return None;
        }

        let mut combined = Vec::new();

        loop {
            match self.inner.as_mut().and_then(|it| it.next()) {
                Some(Ok(line)) => {
                    let line = line.trim();

                    let continues = line.chars().last().unwrap() == ',';
                    let stripped = self.re.replace_all(line, "");

                    let bytes = Vec::<u8>::from_hex(stripped.as_ref()).unwrap();

                    combined.extend(bytes.into_iter());

                    if !continues {
                        let combined_len = combined.len();
                        let mut buf = mem::replace(&mut combined, Vec::new()).into();
                        let res = PackageCodec.decode(&mut buf);

                        return Some(Ok(match res {
                            Ok(Some(pkg)) => {
                                let len = buf.len();

                                if len > 0 {
                                    Decoding::Partial { unused_bytes: len, package: pkg }
                                } else {
                                    Decoding::Full { bytes: combined_len, package: pkg }
                                }
                            },
                            Ok(None) => Decoding::Underflow,
                            Err(e) => Decoding::Failure(e),
                        }));
                    }
                },
                Some(Err(e)) => {
                    self.inner.take();
                    return Some(Err(e))
                },
                None => {
                    self.inner.take();
                    if !combined.is_empty() {
                        return Some(Err(io::Error::new(io::ErrorKind::Other, "Unexpected EOF")));
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use super::DecodingIterator;
    use super::Decoding;

    #[test]
    fn multiline() {

        let input = b"0x21, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x61, 0xde,
    0x39, 0x7d, 0x31, 0x4f, 0x69, 0x40, 0x86, 0xd2,
    0xcb, 0x38, 0x33, 0x94, 0x8b, 0xcc, 0x0a, 0x0b,
    0x74, 0x65, 0x73, 0x74, 0x2d, 0x73, 0x74, 0x72,
    0x65, 0x61, 0x6d, 0x10, 0x00";

        let mut decoder = DecodingIterator::new(BufReader::new(&input[..]));

        match decoder.next() {
            Some(Ok(Decoding::Full{ .. })) => { /* good */ },
            x => panic!("unexpected value: {:?}", x),
        }

        let next = decoder.next();
        assert!(next.is_none(), "unexpected value: {:?}", next);
    }

    #[test]
    fn mixed() {
        use std::io::BufReader;

        let input = b"210000 0x00, 0xc0, 0x00, 0x61, 0xde,
                0x39, 0x7d, 0x31, 0x4f, 0x69, 0x40, 0x86, 0xd2,
    0xcb, 0x38,             0x33, 0x94, 0x8b, 0xcc, 0x0a, 0x0b,
    0x74, 0x65, 0x73, 0x74, 0x2d, 0x73, 0x74, 0x72,
    0x65, 0x61, 0x6d, 0x10, 0x00";

        let mut decoder = DecodingIterator::new(BufReader::new(&input[..]));

        match decoder.next() {
            Some(Ok(Decoding::Full { .. })) => { /* good */ },
            x => panic!("unexpected value: {:?}", x),
        }

        let next = decoder.next();
        assert!(next.is_none(), "unexpected value: {:?}", next);
    }

    #[test]
    fn partial() {
        use std::io::BufReader;

        let input = b"0x21, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x61, 0xde,
    0x39, 0x7d, 0x31, 0x4f, 0x69, 0x40, 0x86, 0xd2,
    0xcb, 0x38, 0x33, 0x94, 0x8b, 0xcc, 0x0a, 0x0b,
    0x74, 0x65, 0x73, 0x74, 0x2d, 0x73, 0x74, 0x72,
    0x65, 0x61, 0x6d, 0x10, 0x00 00 00";

        let mut decoder = DecodingIterator::new(BufReader::new(&input[..]));

        match decoder.next() {
            Some(Ok(Decoding::Partial { unused_bytes: 2, .. })) => { /* good */ },
            x => panic!("unexpected value: {:?}", x),
        }

        let next = decoder.next();
        assert!(next.is_none(), "unexpected value: {:?}", next);
    }

    #[test]
    fn underflow() {
        use std::io::BufReader;

        let input = b"0x21, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x61, 0xde,
    0x39, 0x7d, 0x31, 0x4f, 0x69, 0x40, 0x86, 0xd2,
    0xcb, 0x38, 0x33, 0x94, 0x8b, 0xcc, 0x0a, 0x0b,
    0x74, 0x65, 0x73, 0x74, 0x2d, 0x73, 0x74, 0x72,
    0x65, 0x61, 0x6d, 0x10";

        let mut decoder = DecodingIterator::new(BufReader::new(&input[..]));

        match decoder.next() {
            Some(Ok(Decoding::Underflow)) => { /* good */ },
            x => panic!("unexpected value: {:?}", x),
        }

        let next = decoder.next();
        assert!(next.is_none(), "unexpected value: {:?}", next);
    }

    #[test]
    fn invalid_data() {
        use std::io::BufReader;

        let input = b"0c000000c00061de397d314f00000000000000000000";

        let mut decoder = DecodingIterator::new(BufReader::new(&input[..]));

        match decoder.next() {
            Some(Ok(Decoding::Failure(_))) => { /* good */ },
            x => panic!("unexpected value: {:?}", x),
        }

        let next = decoder.next();
        assert!(next.is_none(), "unexpected value: {:?}", next);
    }
}
