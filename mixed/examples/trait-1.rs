#[derive(Debug)]
struct Buffer<T>(Vec<T>);

impl<T> Buffer<T> {
    pub fn new(v: impl Into<Vec<T>>) -> Self {
        // 对Vec实现from(T)的转换
        Self(v.into())
    }
}

fn main() {
    let v = [1, 2, 3, 4];
    let b = Buffer::new(v);
    let vf = Vec::from([1, 2, 3]);
    println!("{:?} {:?}", b, vf);
}
