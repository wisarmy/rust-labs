use std::{slice::from_raw_parts, str::from_utf8_unchecked};

pub fn get_str_at_location(pointer: usize, length: usize) -> &'static str {
    unsafe { from_utf8_unchecked(from_raw_parts(pointer as *const u8, length)) }
}
pub fn get_memory_location() -> (usize, usize) {
    // “Hello World” 是字符串字面量，因此它的生命周期是 `'static`.
    // 但持有它的变量 `string` 的生命周期就不一样了，它完全取决于变量作用域，对于该例子来说，也就是当前的函数范围
    let string = "Hello World!";
    let pointer = string.as_ptr() as usize;
    let length = string.len();
    (pointer, length)
    // `string` 在这里被 drop 释放
    // 虽然变量被释放，无法再被访问，但是数据依然还会继续存活
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_str_at_location() {
        let (pointer, length) = get_memory_location();
        println!("{} {}", pointer, length);
        assert_eq!("Hello World!", get_str_at_location(pointer, length));
    }
}
