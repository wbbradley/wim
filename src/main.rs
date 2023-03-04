mod files;
mod utils;

struct TermiosRestorer {
    pub orig: libc::termios,
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
    let termios = TermiosRestorer::new();
    let mut raw = libc::termios {
        c_cc: termios.orig.c_cc,
        c_cflag: termios.orig.c_cflag,
        c_iflag: termios.orig.c_iflag,
        c_ispeed: termios.orig.c_ispeed,
        c_ospeed: termios.orig.c_ospeed,
        c_lflag: termios.orig.c_lflag,
        c_oflag: termios.orig.c_oflag,
    };
    raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
    raw.c_oflag &= !(libc::OPOST);
    raw.c_cflag |= libc::CS8;
    raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
    raw.c_cc[libc::VMIN] = 0;
    raw.c_cc[libc::VTIME] = 1;
    unsafe {
        libc::tcsetattr(
            libc::STDIN_FILENO,
            libc::TCSAFLUSH,
            &mut raw as *mut libc::termios,
        );
    }
    println!("Hello, world!");
    std::thread::sleep(std::time::Duration::from_secs(2));
}
