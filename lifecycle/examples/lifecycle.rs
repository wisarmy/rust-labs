/// 如果一个值的生命周期贯穿整个进程的生命周期, 那么我们就称这种生命周期为静态生命周期
/// 当值拥有静态生命周期，其引用也具有静态生命周期。我们在表述这种引用的时候，可以用'static来表示。比如&'static
/// str 代表这是一个具有静态生命周期的字符串引用。
/// 生命周期标注的目的是，在参数和返回值之间建立联系或者约束

pub fn strtok<'a>(s: &mut &'a str, delemiter: char) -> &'a str {
    //pub fn strtok<'a>(s: &'a mut &str, delemiter: char) -> &'a str {
    if let Some(i) = s.find(delemiter) {
        let prefix = &s[..i];

        let suffix = &s[(i + delemiter.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        prefix
    }
}

fn main() {
    let s = "hello world".to_owned();
    let mut s1 = s.as_str();
    let hello = strtok(&mut s1, ' ');
    println!("hello is: {}, s1: {}, s:{}", hello, s1, s)
}
