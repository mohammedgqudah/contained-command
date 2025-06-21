use curium::{
    Container,
    clone3::{CloneResult, clone3},
};

fn main() {
    Container::new("ls".into()).spawn().unwrap();
}
