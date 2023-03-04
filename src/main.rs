mod files;
mod utils;

struct TermiosRestorer {
    orig: libc::termios,
}

impl TermiosRestorer {
    fn new() -> Self {
        let fd = libc::STDIN_FILENO;
        let mut termios = Self {
            orig: libc::termios {
                c_cc: [0; 20],
                c_cflag: 0,
                c_iflag: 0,
                c_ispeed: 0,
                c_ospeed: 0,
                c_lflag: 0,
                c_oflag: 0,
            },
        };
        let ret = unsafe { libc::tcgetattr(fd, &mut termios.orig as *mut libc::termios) };
        if ret == -1 {
            c_catastrophe!("unable to tcgetattr");
        }
        termios
    }
}

impl Drop for TermiosRestorer {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &mut self.orig as *mut libc::termios,
            );
        }
    }
}

fn main() {
    let _orig_termios = TermiosRestorer::new();
    println!("Hello, world!")
}
