extern crate festival;

fn main() {
    let (mut f, rx) = festival::hold().unwrap();
    f.clear().unwrap();
    f.set_cursor(3, 3).unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.hide_cursor().unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.set_cursor(4, 4).unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));

    loop {
        let v = rx.recv().unwrap();
        if v.len() == 1 && v[0] == 'q' as u8 {
            break;
        }
        f.set_cursor(4, 4).unwrap();
        for b in v {
            f.putbyte(b).unwrap();
        }
    }
}
