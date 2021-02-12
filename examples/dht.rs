#![no_std]
#![no_main]

extern crate panic_halt;
use dht_sensor::*;
use arduino_uno::prelude::*;

#[arduino_uno::entry]
fn main() -> ! {
    let peripherals = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(peripherals.PORTB, peripherals.PORTC, peripherals.PORTD);

    let mut dht_pin = pins.d13.into_tri_state(&mut pins.ddr);

    let mut delay = arduino_uno::Delay::new();

    let mut serial = arduino_uno::Serial::new(
        peripherals.USART0,
        pins.d0,
        pins.d1.into_output(&mut pins.ddr),
        57600.into_baudrate(),
    );
    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();


    loop {
        arduino_uno::delay_ms(2000); // wait 1s before reading
        let result = dht11::Reading::read(&mut delay, &mut dht_pin).unwrap();

        ufmt::uwriteln!(&mut serial,
            "Temperature C: {}\r",
            result.temperature).void_unwrap();
        ufmt::uwriteln!(&mut serial,
            "Humidity: {}\r",
            result.relative_humidity).void_unwrap();
    }
}


