use libc::*;

pub fn build_terminated_if_name(if_name: &str) -> [c_char; IFNAMSIZ] {
    let mut buf = [0 as c_char; IFNAMSIZ];
    let name_bytes = if_name.as_bytes();
    let mut i = 0;
    for c in name_bytes {
        buf[i] = *c as c_char;
        i += 1;
        if i >= IFNAMSIZ - 1 {
            break;
        }
    }
    buf
}
