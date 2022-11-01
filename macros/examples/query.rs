use macros::query;

fn main() {
    query!(SELECT * FROM users u JOIN (SELECT * from profiles p) WHERE u.id = p.id and u.age > 10);
    hello()
}
