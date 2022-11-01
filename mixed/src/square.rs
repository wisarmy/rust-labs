fn is_square(n: i64) -> bool {
    for x in 0..n + 1 {
        println!("{}", x);
        match x {
            x if (x - 1) * (x + 1) == n - 1 => {
                return true;
            }
            x if (x - 1) * (x + 1) > n - 1 => {
                return false;
            }
            _ => continue,
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_square() {
        //assert_eq!(is_square(1), true);
        assert_eq!(is_square(2), false);
        assert_eq!(is_square(4), true);
        assert_eq!(is_square(25), true);
        assert_eq!(is_square(26), false);
        assert_eq!(is_square(2600000000000), false);
    }
}
