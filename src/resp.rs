use bytes::{Buf, BytesMut};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
    Array(Option<Vec<Frame>>),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("incomplete frame")]
    Incomplete,
    #[error("invalid frame format")]
    Invalid,
}

impl Frame {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Frame::Simple(s) => format!("+{}\r\n", s).into_bytes(),
            Frame::Error(msg) => format!("-{}\r\n", msg).into_bytes(),
            Frame::Integer(num) => format!(":{}\r\n", num).into_bytes(),
            Frame::Bulk(None) => "$-1\r\n".into(),
            Frame::Bulk(Some(data)) => {
                let mut res = format!("${}\r\n", data.len()).into_bytes();
                res.extend(data);
                res.extend(b"\r\n");
                res
            }
            Frame::Array(None) => "*-1\r\n".into(),
            Frame::Array(Some(items)) => {
                let mut res = format!("*{}\r\n", items.len()).into_bytes();
                for item in items {
                    res.extend(&item.encode());
                }
                res
            }
        }
    }

    pub fn parse(src: &mut BytesMut) -> Result<Option<Frame>, Error> {
        if src.is_empty() {
            return Ok(None);
        }

        match src[0] as char {
            '+' => parse_simple(src),
            '-' => parse_error(src),
            ':' => parse_integer(src),
            '$' => parse_bulk(src),
            '*' => parse_array(src),
            _ => Err(Error::Invalid),
        }
    }
}

fn parse_simple(src: &mut BytesMut) -> Result<Option<Frame>, Error> {
    if let Some(i) = find_crlf(src) {
        let line = String::from_utf8_lossy(&src[1..i]).to_string();
        src.advance(i + 2);
        Ok(Some(Frame::Simple(line)))
    } else {
        Ok(None)
    }
}

fn parse_error(src: &mut BytesMut) -> Result<Option<Frame>, Error> {
    if let Some(i) = find_crlf(src) {
        let line = String::from_utf8_lossy(&src[1..i]).to_string();
        src.advance(i + 2);
        Ok(Some(Frame::Error(line)))
    } else {
        Ok(None)
    }
}

fn parse_integer(src: &mut BytesMut) -> Result<Option<Frame>, Error> {
    if let Some(i) = find_crlf(src) {
        let line = &src[1..i];
        let num = atoi::atoi::<i64>(line).ok_or(Error::Invalid)?;
        src.advance(i + 2);
        Ok(Some(Frame::Integer(num)))
    } else {
        Ok(None)
    }
}

fn parse_bulk(src: &mut BytesMut) -> Result<Option<Frame>, Error> {
    if let Some(i) = find_crlf(src) {
        let len = atoi::atoi::<i64>(&src[1..i]).ok_or(Error::Invalid)?;
        
        if len < 0 {
            src.advance(i + 2);
            return Ok(Some(Frame::Bulk(None)));
        }

        let len = len as usize;
        let end = i + 2 + len + 2;

        if src.len() < end {
            return Ok(None);
        }

        let data = src[i + 2..i + 2 + len].to_vec();
        src.advance(end);
        Ok(Some(Frame::Bulk(Some(data))))
    } else {
        Ok(None)
    }
}

fn parse_array(src: &mut BytesMut) -> Result<Option<Frame>, Error> {
    if let Some(i) = find_crlf(src) {
        let len = atoi::atoi::<i64>(&src[1..i]).ok_or(Error::Invalid)?;

        if len < 0 {
            src.advance(i + 2);
            return Ok(Some(Frame::Array(None)));
        }

        let len = len as usize;
        src.advance(i + 2);

        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            match Frame::parse(src)? {
                Some(frame) => items.push(frame),
                None => return Ok(None),
            }
        }

        Ok(Some(Frame::Array(Some(items))))
    } else {
        Ok(None)
    }
}

fn find_crlf(src: &[u8]) -> Option<usize> {
    src.windows(2).position(|bytes| bytes == b"\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_parse_simple_string() {
        let mut bytes = BytesMut::from("+OK\r\n");
        let frame = Frame::parse(&mut bytes).unwrap().unwrap();
        assert_eq!(frame, Frame::Simple("OK".to_string()));
    }

    #[test]
    fn test_parse_error() {
        let mut bytes = BytesMut::from("-Error message\r\n");
        let frame = Frame::parse(&mut bytes).unwrap().unwrap();
        assert_eq!(frame, Frame::Error("Error message".to_string()));
    }

    #[test]
    fn test_parse_integer() {
        let mut bytes = BytesMut::from(":1234\r\n");
        let frame = Frame::parse(&mut bytes).unwrap().unwrap();
        assert_eq!(frame, Frame::Integer(1234));
    }

    #[test]
    fn test_parse_bulk() {
        let mut bytes = BytesMut::from("$5\r\nhello\r\n");
        let frame = Frame::parse(&mut bytes).unwrap().unwrap();
        assert_eq!(frame, Frame::Bulk(Some(b"hello".to_vec())));
    }

    #[test]
    fn test_parse_null_bulk() {
        let mut bytes = BytesMut::from("$-1\r\n");
        let frame = Frame::parse(&mut bytes).unwrap().unwrap();
        assert_eq!(frame, Frame::Bulk(None));
    }
} 