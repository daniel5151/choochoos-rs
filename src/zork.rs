use embcrusted::{Options, Ui, Zmachine};

const DATA: &[u8] = include_bytes!("../../../tmp/ZORK1.DAT");

pub fn run_zork(uart: crate::uart::Uart) {
    let opts = Options { rand_seed: 0x1337 };
    let mut zvm = Zmachine::new(&DATA, DumbUi::new(uart), opts);

    while !zvm.step() {
        zvm.ui.fill_input_buf();
        zvm.ack_input();
    }
}

pub struct DumbUi {
    uart: crate::uart::Uart,
    buf: [u8; 64],
}

impl DumbUi {
    fn new(uart: crate::uart::Uart) -> DumbUi {
        DumbUi { uart, buf: [0; 64] }
    }

    fn fill_input_buf(&mut self) {
        let mut i = 0;
        loop {
            let c = self.uart.read_byte_blocking();
            // self.uart.write_blocking(format!("<{}>", c).as_bytes());

            match c {
                b'\r' => {
                    self.uart.write_byte_blocking(b'\r');
                    self.uart.write_byte_blocking(b'\n');
                    self.buf[i] = b'\0';
                    break;
                }
                // backspace
                0x08 => {
                    self.uart.write_blocking(b"\x1b[1D \x1b[1D");
                    i -= 1;
                    self.buf[i] = b'\0';
                }
                _ => {
                    self.uart.write_byte_blocking(c);
                    self.buf[i] = c;
                    i += 1;
                }
            }
        }
    }
}

impl Ui for DumbUi {
    fn print(&self, text: &str) {
        for b in text.as_bytes() {
            self.uart.write_byte_blocking(*b);
            if *b == b'\n' {
                self.uart.write_byte_blocking(b'\r');
            }
        }
    }

    fn print_object(&mut self, object: &str) {
        self.print(&format!("\x1b[1m{}\x1b[m", object));
    }

    fn set_status_bar(&self, left: &str, right: &str) {
        // let _ = (left, right);
        self.print(&format!("{}  -  {}", left, right));
    }

    fn get_input_buf(&mut self) -> &str {
        &core::str::from_utf8(&self.buf[..self.buf.iter().position(|x| *x == 0).unwrap()])
            .expect("cannot parse invalid utf8")
    }
}
