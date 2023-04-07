use std::mem::{transmute, size_of, size_of_val};

use rsa::pss::Signature;

use super::pkg::PackageType;

pub trait Serializer {
    fn serialize(&self, dst: &mut [u8]) -> usize;
    fn deserialize(bytes: &[u8]) -> (usize, Self);
}

impl Serializer for String {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        let mut start = 0;
        start += self.len().serialize(dst);
        dst[start..start+self.len()].copy_from_slice(self.as_bytes());
        return start+self.len();
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        let mut start: usize = 0;

        let (size, str_len) = usize::deserialize(bytes);
        start += size;

        return (start+str_len, String::from_utf8_lossy(&bytes[start..start+str_len]).to_string());
    }
}

impl Serializer for u64 {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<u64>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<u64, [u8; SIZE]>(*self) });
        return SIZE;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const Self;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for u128 {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<u128>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<u128, [u8; SIZE]>(*self) });
        return SIZE
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const Self;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for u16 {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<u16>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<u16, [u8; SIZE]>(*self) });
        return SIZE;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const Self;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for u8 {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        dst[0] = *self;
        return size_of::<Self>();
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return (size_of::<Self>(), bytes[0]);
    }
}

impl Serializer for PackageType {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<PackageType>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<PackageType, [u8; SIZE]>(*self) });
        return SIZE;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const PackageType;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for usize {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<usize>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<usize, [u8; SIZE]>(*self) });
        return SIZE;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const Self;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for f64 {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<f64>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<f64, [u8; SIZE]>(*self) });
        return SIZE;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const Self;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for bool {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<bool>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<bool, [u8; SIZE]>(*self) });
        return SIZE;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        return unsafe {
            let ptr = bytes.as_ptr() as *const Self;
            (size_of::<Self>(), (*ptr).clone())
        };
    }
}

impl Serializer for Signature {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        const SIZE: usize = size_of::<usize>();

        let sign_as_bytes = Box::<[u8]>::from(self.clone());
        let sign_size = size_of_val(&*sign_as_bytes);
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<usize, [u8; SIZE]>(sign_size) });
        dst[SIZE..SIZE+sign_size].copy_from_slice(&*sign_as_bytes);

        return SIZE+sign_size;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        let mut start: usize = 0;
        let (size, sign_len) = usize::deserialize(bytes);
        start += size;

        return (start+sign_len, Signature::try_from(&bytes[start..start+sign_len]).unwrap());
    }
}
