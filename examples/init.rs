extern crate festival;

use festival::Festival;

fn main() {
    let mut f = Festival::new().unwrap();
    f.clear().unwrap();
    f.set_cursor(3, 3).unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.hide_cursor().unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.set_cursor(4, 4).unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
}
