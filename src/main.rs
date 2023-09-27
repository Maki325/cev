use cev::Compressed;

mod example;
use example::*;

fn _tests() {
  let _ = Vec::<i32>::with_capacity(10);

  let val = Test::<u32>::Int8(8);
  let vec = vec![
    val.clone(),
    val.clone(),
    val.clone(),
    val.clone(),
    val.clone(),
  ];
  // vec.push(value)

  let test1 = std::mem::size_of::<Test<u32>>();
  let test2 = std::mem::size_of_val(&val);
  let test3 = std::mem::size_of_val(&*vec);

  println!("{} {} {}", test1, test2, test3);

  struct A {
    a: u16,
    b: u16,
  }

  let a = A { a: 1, b: 2 };
  // let a = A { a: 1, b: 2, c: 3 };

  println!(
    "std::mem::align_of<u8>(): {}",
    std::mem::align_of::<[u8; 3]>()
  );
  println!("std::mem::align_of<A>(): {}", std::mem::align_of::<A>());
  println!("std::mem::align_of_val(a): {}", std::mem::align_of_val(&a));
  println!("std::mem::align_of<i16>(): {}", std::mem::align_of::<i16>());
  println!("std::mem::align_of<i32>(): {}", std::mem::align_of::<i32>());
  println!("std::mem::align_of<i64>(): {}", std::mem::align_of::<i64>());
  println!(
    "std::mem::align_of<i128>(): {}",
    std::mem::align_of::<i128>()
  );

  println!("Hello, world!");

  let mut s = [1, 2, 3];
  let ptr: *const u32 = s.as_mut_ptr();
  unsafe {
    println!("{}", *ptr.offset(1));
    println!("{}", *ptr.offset(2));
  }

  // let mut v = vec![1, 2, 3];
  // let b = &v[1];
  // let a = &mut v[0];
  // let z = b + 1;
  // let c = &v[2];
}

fn main() {
  let mut compressed = Compressed::<Test<i32>>::new();
  compressed.push(Test::Int8(8));
  compressed.push(Test::Int8(9));
  compressed.push(Test::Int16(16));
  compressed.push(Test::Int16(17));
  compressed.push(Test::Int16(18));
  compressed.push(Test::Int16(19));
  compressed.push(Test::Int16(20));
  compressed.push(Test::None);
  compressed.push(Test::Int32(32));
  compressed.push(Test::Int16((u8::MAX as u16) + 2));

  let slice = unsafe { std::slice::from_raw_parts(compressed.data, compressed.cap) };

  // compressed.get(2);
  println!("compressed.get(7): {:#?}", compressed.get(7));
  println!("compressed.get(8): {:#?}", compressed.get(8));
  println!("compressed.get(9): {:#?}", compressed.get(9));

  println!("{:#?}", slice);
  // compressed.update(7, |_old| Test::None);
  compressed.update(7, |_old| Test::Int64(64));

  println!("compressed.get(7): {:#?}", compressed.get(7));
  println!("compressed.get(8): {:#?}", compressed.get(8));
  println!("compressed.get(9): {:#?}", compressed.get(9));

  // compressed.get(idx)

  let slice = unsafe { std::slice::from_raw_parts(compressed.data, compressed.cap) };
  println!("{:#?}", slice);
}
