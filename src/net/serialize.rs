use std::mem::{transmute, size_of, size_of_val};

use rsa::pss::Signature;

use super::pkg::PackageType;

pub trait Serializer {
    fn serialize(&self, dst: &mut [u8]);
    fn deserialize(bytes: &[u8]) -> Self;
}

impl Serializer for String {
    fn serialize(&self, dst: &mut [u8]) {
        const SIZE: usize = size_of::<usize>();

        self.len().serialize(dst);
        dst[SIZE..SIZE+self.len()].copy_from_slice(self.as_bytes());
    }

    fn deserialize(bytes: &[u8]) -> Self {
        const SIZE: usize = size_of::<usize>();

        let size = usize::deserialize(bytes);
        return String::from_utf8_lossy(&bytes[SIZE..SIZE+size]).to_string();
    }
}

impl Serializer for u64 {
    fn serialize(&self, dst: &mut [u8]) {
        const SIZE: usize = size_of::<usize>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<u64, [u8; SIZE]>(*self) });
    }

    fn deserialize(bytes: &[u8]) -> Self {
        return unsafe {
            let ptr = bytes.as_ptr() as *const u64;
            (*ptr).clone()
        };
    }
}

impl Serializer for PackageType {
    fn serialize(&self, dst: &mut [u8]) {
        const SIZE: usize = size_of::<PackageType>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<PackageType, [u8; SIZE]>(*self) });
    }

    fn deserialize(bytes: &[u8]) -> Self {
        return unsafe {
            let ptr = bytes.as_ptr() as *const PackageType;
            (*ptr).clone()
        };
    }
}

impl Serializer for usize {
    fn serialize(&self, dst: &mut [u8]) {
        const SIZE: usize = size_of::<usize>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<usize, [u8; SIZE]>(*self) });
    }

    fn deserialize(bytes: &[u8]) -> Self {
        return unsafe {
            let ptr = bytes.as_ptr() as *const usize;
            (*ptr).clone()
        };
    }
}

impl Serializer for f64 {
    fn serialize(&self, dst: &mut [u8]) {
        const SIZE: usize = size_of::<usize>();
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<f64, [u8; SIZE]>(*self) });
    }

    fn deserialize(bytes: &[u8]) -> Self {
        return unsafe {
            let ptr = bytes.as_ptr() as *const f64;
            (*ptr).clone()
        };
    }
}

impl Serializer for Signature {
    fn serialize(&self, dst: &mut [u8]) {
        const SIZE: usize = size_of::<usize>();

        let sign_as_bytes = Box::<[u8]>::from(self.clone());
        let sign_size = size_of_val(&*sign_as_bytes);
        dst[..SIZE].copy_from_slice(unsafe { &transmute::<usize, [u8; SIZE]>(sign_size) });
        dst[SIZE..SIZE+sign_size].copy_from_slice(&*sign_as_bytes);
    }

    fn deserialize(bytes: &[u8]) -> Self {
        const SIZE: usize = size_of::<usize>();
        let sign_size = usize::deserialize(bytes);
        return Signature::try_from(&bytes[SIZE..SIZE+sign_size]).unwrap();
    }
}
