use crate::resp::Frame;
use crate::db::Db;
use std::sync::Arc;

#[derive(Debug)]
pub enum Command {
    Get { key: String },
    Set { key: String, value: Vec<u8> },
    Del { key: String },
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command, String> {
        match frame {
            Frame::Array(Some(array)) => {
                let mut array = array.into_iter();
                
                let command = match array.next() {
                    Some(Frame::Bulk(Some(bytes))) => {
                        String::from_utf8_lossy(&bytes).to_uppercase()
                    }
                    _ => return Err("expected bulk string".to_string()),
                };

                match command.as_str() {
                    "GET" => {
                        let key = match array.next() {
                            Some(Frame::Bulk(Some(bytes))) => {
                                String::from_utf8_lossy(&bytes).to_string()
                            }
                            _ => return Err("GET expects key".to_string()),
                        };
                        Ok(Command::Get { key })
                    }
                    "SET" => {
                        let key = match array.next() {
                            Some(Frame::Bulk(Some(bytes))) => {
                                String::from_utf8_lossy(&bytes).to_string()
                            }
                            _ => return Err("SET expects key".to_string()),
                        };
                        let value = match array.next() {
                            Some(Frame::Bulk(Some(bytes))) => bytes,
                            _ => return Err("SET expects value".to_string()),
                        };
                        Ok(Command::Set { key, value })
                    }
                    "DEL" => {
                        let key = match array.next() {
                            Some(Frame::Bulk(Some(bytes))) => {
                                String::from_utf8_lossy(&bytes).to_string()
                            }
                            _ => return Err("DEL expects key".to_string()),
                        };
                        Ok(Command::Del { key })
                    }
                    _ => Err(format!("unknown command '{}'", command)),
                }
            }
            _ => Err("expected array".to_string()),
        }
    }

    pub fn execute(self, db: &Arc<Db>) -> Frame {
        match self {
            Command::Get { key } => {
                match db.get(&key) {
                    Some(value) => Frame::Bulk(Some(value)),
                    None => Frame::Bulk(None),
                }
            }
            Command::Set { key, value } => {
                db.set(key, value);
                Frame::Simple("OK".to_string())
            }
            Command::Del { key } => {
                let deleted = db.delete(&key);
                Frame::Integer(if deleted { 1 } else { 0 })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get() {
        let frame = Frame::Array(Some(vec![
            Frame::Bulk(Some(b"GET".to_vec())),
            Frame::Bulk(Some(b"key1".to_vec())),
        ]));
        
        match Command::from_frame(frame).unwrap() {
            Command::Get { key } => assert_eq!(key, "key1"),
            _ => panic!("expected GET command"),
        }
    }

    #[test]
    fn test_parse_set() {
        let frame = Frame::Array(Some(vec![
            Frame::Bulk(Some(b"SET".to_vec())),
            Frame::Bulk(Some(b"key1".to_vec())),
            Frame::Bulk(Some(b"value1".to_vec())),
        ]));
        
        match Command::from_frame(frame).unwrap() {
            Command::Set { key, value } => {
                assert_eq!(key, "key1");
                assert_eq!(value, b"value1");
            }
            _ => panic!("expected SET command"),
        }
    }

    #[test]
    fn test_execute_commands() {
        let db = Arc::new(Db::new());
        
        // Test SET
        let cmd = Command::Set {
            key: "key1".to_string(),
            value: b"value1".to_vec(),
        };
        let result = cmd.execute(&db);
        assert_eq!(result, Frame::Simple("OK".to_string()));
        
        // Test GET
        let cmd = Command::Get {
            key: "key1".to_string(),
        };
        let result = cmd.execute(&db);
        assert_eq!(result, Frame::Bulk(Some(b"value1".to_vec())));
        
        // Test DEL
        let cmd = Command::Del {
            key: "key1".to_string(),
        };
        let result = cmd.execute(&db);
        assert_eq!(result, Frame::Integer(1));
    }
} 