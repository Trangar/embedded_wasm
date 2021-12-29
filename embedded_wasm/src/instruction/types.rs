#[derive(Clone, Debug, PartialEq)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    ValType(ValType),
    Type(TypeIdx),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signedness {
    Unsigned,
    Signed,
}

pub trait IndexAlias {
    fn new(val: u32) -> Self;
}

macro_rules! impl_idx {
    ($name:ident (prefix: $prefix:expr)) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub usize);

        impl core::fmt::Debug for $name {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(fmt, "{}{}", $prefix, self.0)
            }
        }

        impl IndexAlias for $name {
            fn new(val: u32) -> Self {
                Self(val as usize)
            }
        }
    };
}

impl_idx!(TypeIdx (prefix: "$t"));
impl_idx!(LabelIdx (prefix: "$L"));
impl_idx!(FuncIdx (prefix: "$f"));
impl_idx!(TableIdx (prefix: "$t"));
impl_idx!(MemIdx (prefix: "$m"));
impl_idx!(GlobalIdx (prefix: "$g"));
impl_idx!(LocalIdx (prefix: "$l"));
impl_idx!(ElemIdx (prefix: "$e"));
impl_idx!(DataIdx (prefix: "$d"));

#[derive(Debug)]
pub enum SectionType {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    DataCount = 12,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefType {
    FuncRef = 0x70,
    ExternRef = 0x6F,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValType {
    Num(NumType),
    Ref(RefType),
}
