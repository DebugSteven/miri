//@ignore-windows: File handling is not implemented yet
//@error-pattern: `open` not available when isolation is enabled

fn main() {
    let _file = std::fs::File::open("file.txt").unwrap();
}
