use curium::Container;

fn main() {
    Container::new("/tmp/bbox".into(), "/bin/sh")
        .spawn()
        .unwrap();
}
