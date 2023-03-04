use crate::utils::die;

pub struct Termios {
    pub orig: libc::termios,
}

impl Termios {
    fn new() -> Self {
        let fd = libc::STDIN_FILENO;
        let mut termios = Self {
            orig: unsafe { std::mem::zeroed() },
        };
        let ret = unsafe { libc::tcgetattr(fd, &mut termios.orig as *mut libc::termios) };
        if ret == -1 {
            die!("[Termios::new] unable to tcgetattr");
        }
        termios
    }
    fn enable_raw_mode(&self) {
        let mut raw = libc::termios {
            c_cc: self.orig.c_cc,
            c_cflag: self.orig.c_cflag,
            c_iflag: self.orig.c_iflag,
            c_ispeed: self.orig.c_ispeed,
            c_ospeed: self.orig.c_ospeed,
            c_lflag: self.orig.c_lflag,
            c_oflag: self.orig.c_oflag,
        };
        raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        raw.c_oflag &= !(libc::OPOST);
        raw.c_cflag |= libc::CS8;
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 1;
        if unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &mut raw as *mut libc::termios,
            )
        } == -1
        {
            die!("tcsetattr failed");
        }
    }
    pub fn enter_raw_mode() -> Self {
        let termios = Self::new();
        termios.enable_raw_mode();
        termios
    }
}

impl Drop for Termios {
    fn drop(&mut self) {
        if unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &mut self.orig as *mut libc::termios,
            )
        } == -1
        {
            die!("Termios::drop");
        }
    }
}
