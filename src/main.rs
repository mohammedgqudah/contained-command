use curium::Container;

fn main() {
    //Container::new("/bin/cat")
    //    .env("PATH=/")
    //    .arg("/proc/self/uid_map")
    //    .spawn()
    //    .unwrap();
    //Container::new("/bin/id").arg("-un").spawn().unwrap();
    Container::new("/bin/sh").spawn().unwrap();
}
