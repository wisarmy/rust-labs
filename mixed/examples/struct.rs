use std::ops::Deref;

#[derive(Debug)]
struct CreatePayload {
    name: String,
}

#[derive(Debug)]
struct UpdatePayload {
    id: u32,
    create: CreatePayload,
}

impl Deref for UpdatePayload {
    type Target = CreatePayload;
    fn deref(&self) -> &Self::Target {
        &self.create
    }
}

fn main() {
    let cp = CreatePayload {
        name: "x".to_owned(),
    };
    let up = UpdatePayload { id: 1, create: cp };

    println!("up: {:?} {:?}", up.id, up.name);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deref_name() {
        let cp = CreatePayload {
            name: "x".to_owned(),
        };
        let up = UpdatePayload { id: 1, create: cp };
        assert_eq!(up.create.name, up.name);
    }
}
