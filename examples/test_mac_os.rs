use irdopto_im12xx::{Im12xx, BAUT};
use serialport::new;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // println!("Available ports: {:?}", available_ports());

    let mut serial_port = new("/dev/tty.usbserial-A10KGKTM", BAUT).open().unwrap();
    let mut buffer = [0; 256];
    let mut im12xx = Im12xx::new(&mut buffer).unwrap();

    loop {
        let request = im12xx.request();
        println!("***************************");
        println!("Serial port write request: {:?}", request);
        serial_port.write(request).unwrap();

        sleep(Duration::from_millis(1000));

        let mut response = [0; 128];
        let len = serial_port.read(&mut response).unwrap();
        // println!("Response len: {}", len);
        println!("Serial port read: {:?}", &response[..len]);
        println!("Parsed Data: {:?}", im12xx.response(&response[..len]));
        println!("***************************\n");

        sleep(Duration::from_millis(1000));
    }
}
