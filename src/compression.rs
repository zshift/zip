//! Possible ZIP compression methods.

use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct CompressionMethod(u16);

pub const METHOD_STORED: CompressionMethod = CompressionMethod(0);
pub const METHOD_DEFLATE: CompressionMethod = CompressionMethod(8);
pub const METHOD_BZIP2: CompressionMethod = CompressionMethod(12);
pub const METHOD_LZMA: CompressionMethod = CompressionMethod(14);

impl CompressionMethod {
    pub fn from_u16(val: u16) -> CompressionMethod {
        CompressionMethod(val)
    }

    pub fn to_u16(self) -> u16 {
        self.0
    }

    pub fn get_name(self) -> Option<&'static str> {
        Some(match self {
            METHOD_STORED => "Stored",
            METHOD_DEFLATE => "Deflated",
            METHOD_BZIP2 => "Bzip2",
            METHOD_LZMA => "LZMA",
            _ => return None,
        })
    }
}

impl fmt::Debug for CompressionMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get_name() {
            Some(name) => write!(f, "CompressionMethod({:?})", name),
            None => write!(f, "CompressionMethod({})", self.0),
        }
    }
}

impl fmt::Display for CompressionMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/*
impl CompressionMethod {
    /// Converts an u16 to its corresponding CompressionMethod
    pub fn from_u16(val: u16) -> CompressionMethod {
        match val {
            0 => CompressionMethod::Stored,
            8 => CompressionMethod::Deflated,
            12 => CompressionMethod::Bzip2,
            v => CompressionMethod::Other(v),
        }
    }

    /// Converts a CompressionMethod to a u16
    pub fn to_u16(self) -> u16 {
        match self {
            CompressionMethod::Stored => 0,
            #[cfg(feature = "deflate")]
            CompressionMethod::Deflated => 8,
            #[cfg(feature = "bzip2")]
            CompressionMethod::Bzip2 => 12,
            CompressionMethod::Other(v) => v,
        }
    }
}
impl fmt::Display for CompressionMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Just duplicate what the Debug format looks like, i.e, the enum key:
        write!(f, "{:?}", self)
    }
}
*/

#[cfg(test)]
mod test {
    use super::CompressionMethod;

    #[test]
    fn from_eq_to() {
        for v in 0..(::std::u16::MAX as u32 + 1)
        {
            let from = CompressionMethod::from_u16(v as u16);
            let to = from.to_u16() as u32;
            assert_eq!(v, to);
        }
    }

    fn methods() -> Vec<CompressionMethod> {
        let mut methods = Vec::new();
        for i in 0 ..= std::u16::MAX {
            methods.push(CompressionMethod(i));
        }
        //methods.push(compression::METHOD_STORED);
        //#[cfg(feature="deflate")] methods.push(compression::METHOD_DEFLATECompressionMethod::Deflated);
        //#[cfg(feature="bzip2")] methods.push(CompressionMethod::Bzip2);
        methods
    }


    #[test]
    fn to_eq_from() {
        fn check_match(method: CompressionMethod) {
            let to = method.to_u16();
            let from = CompressionMethod::from_u16(to);
            let back = from.to_u16();
            assert_eq!(to, back);
        }

        for method in methods() {
            check_match(method);
        }
    }

    #[test]
    fn to_display_fmt() {
        fn check_match(method: CompressionMethod) {
            let debug_str = format!("{:?}", method);
            let display_str = format!("{}", method);
            assert_eq!(debug_str, display_str);
        }

        for method in methods() {
            check_match(method);
        }
    }
}
