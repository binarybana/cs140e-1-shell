extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use xmodem::{Xmodem, Progress};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{copy, Read, BufReader};

    let opt = Opt::from_args();

    let mut serial = serial::open(&opt.tty_path).expect("path points to invalid TTY");
    serial.set_timeout(Duration::from_secs(opt.timeout)).expect("Couldn't set timeout on serial port");
    let mut settings = serial.read_settings().expect("Couldn't read serial settings");
    settings.set_baud_rate(opt.baud_rate).expect("Couldn't change baud rate");
    settings.set_char_size(opt.char_width);
    settings.set_stop_bits(opt.stop_bits);
    settings.set_flow_control(opt.flow_control);

    let mut input: BufReader<Box<Read>> = match opt.input {
        None => BufReader::new(Box::new(std::io::stdin())),
        Some(path) => {
            let fid = File::open(path).expect("Could not open input file");
            BufReader::new(Box::new(fid))
        }
    };

    let num_bytes = if opt.raw {
        copy(&mut input, &mut serial).expect("Raw copy failed") as usize
    } else {
        Xmodem::transmit(&mut input, &mut serial).expect("Error performing xmodem transfer")
    };
    println!("{}", num_bytes);
}
