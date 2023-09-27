use cev_macros::Compress;

#[derive(Debug, Clone, Compress)]
pub enum Test<T: cev::Compress> {
  None,
  Int8(u8),
  Int16(u16),
  Int32(u32),
  Int64(u64),
  T2(T),
  T3(T, u8),
  T4 { a: T, b: u8 },
}

#[derive(Debug, Clone, Compress)]
pub struct NamedStruct {
  a: u8,
  b: u8,
  c: u8,
}

#[derive(Debug, Clone, Compress)]
pub struct EmptyStruct;

#[derive(Debug, Clone, Compress)]
pub struct UnnamedStruct(u8, u16, u32, u64);
