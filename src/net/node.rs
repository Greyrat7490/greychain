use std::fmt::Display;

use super::serialize::Serializer;

pub struct Node {
    pub pub_key: String,
    pub port: u16,
    pub online: bool
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "127.0.0.1:{}", self.port);
    }
}

impl Serializer for Node {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        let mut start = 0;

        start += self.pub_key.serialize(&mut dst[start..]);
        start += self.port.serialize(&mut dst[start..]);
        start += self.online.serialize(&mut dst[start..]);

        return start;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        let mut start = 0;

        let (size, pub_key) = String::deserialize(&bytes[start..]);
        start += size;

        let (size, port) = u16::deserialize(&bytes[start..]);
        start += size;

        let (size, online) = bool::deserialize(&bytes[start..]);
        start += size;

        return (start, Node { pub_key, port, online });
    }
}

impl Serializer for Vec<Node> {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        let mut start = 0;

        assert!(self.len() <= u8::MAX as usize);
        start += (self.len() as u8).serialize(&mut dst[start..]);

        for node in self {
            start += node.serialize(&mut dst[start..]);
        }

        return start;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        let mut start = 0;

        let (size, len) = u8::deserialize(&bytes[start..]);
        start += size;

        let mut nodes = Vec::<Node>::with_capacity(len as usize);
        for _ in 0..len {
            let (size, pub_key) = String::deserialize(&bytes[start..]);
            start += size;

            let (size, port) = u16::deserialize(&bytes[start..]);
            start += size;

            let (size, online) = bool::deserialize(&bytes[start..]);
            start += size;

            nodes.push(Node { pub_key, port, online })
        }

        return (start, nodes);
    }
}
