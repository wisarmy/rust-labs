#[macro_export]
macro_rules! my_vec {
    () => {
        std::vec::Vec::new()
    };
    ($($el:expr),*) => ({
        let mut v = std::vec::Vec::new();
        $(v.push($el);)*
        v
    });
    ($el:expr; $n:expr) => {
        std::vec::from_elem($el, $n)
    }
}

fn main() {
    let mut v = my_vec![2;5];
    v.push(1);
    // 调用时可以使用 [], (), {}
    let _v = my_vec!(1, 2, 3, 4);
    let _v = my_vec![1, 2, 3, 4];
    let v = my_vec! {1, 2, 3, 4};
    println!("{:?}", v);

    println!("{:?}", v);
    //
    let v = my_vec![1; 10];
    println!("{:?}", v);
}
