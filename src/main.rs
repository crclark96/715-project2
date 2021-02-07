#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// heavily influenced by the safe interrupts described by this blogpost:
// https://blog.rahix.de/005-avr-hal-millis/
// https://github.com/Rahix/avr-hal/blob/master/boards/arduino-uno/examples/uno-millis.rs

extern crate panic_halt;
use arduino_uno::hal::port::mode::Output;
use arduino_uno::hal::port::portb::PB5;
use arduino_uno::prelude::*;
use core::cell;

const PRESCALER: u32 = 256;
const TIMER_COUNTS: u32 = 250;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

static LED: avr_device::interrupt::Mutex<cell::RefCell<Option<PB5<Output>>>> =
    avr_device::interrupt::Mutex::new(cell::RefCell::new(Option::None));

static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
    avr_device::interrupt::Mutex::new(cell::Cell::new(0));

fn millis_init(tc0: arduino_uno::pac::TC0) {
    // config timer for interval and enable interrupt
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| unsafe { w.bits(TIMER_COUNTS as u8) });
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter = MILLIS_COUNTER.borrow(cs).get();
        MILLIS_COUNTER.borrow(cs).set(counter + 1);

        // let option_led = LED.borrow(cs).borrow_mut().as_ref();
        // ^ this works
        if counter == 250 {
            (*LED.borrow(cs).borrow_mut()).as_mut().unwrap().toggle().void_unwrap();
            MILLIS_COUNTER.borrow(cs).set(0);
        }

    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| {
       MILLIS_COUNTER.borrow(cs).get()
    })
}


#[arduino_uno::entry]
fn main() -> ! {
    let peripherals = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(peripherals.PORTB, peripherals.PORTC, peripherals.PORTD);

    millis_init(peripherals.TC0);


    let led = pins.d13.into_output(&mut pins.ddr);
    avr_device::interrupt::free(|cs| {
        LED.borrow(cs).replace(Option::Some(led));
    });

    let mut serial = arduino_uno::Serial::new(
        peripherals.USART0,
        pins.d0,
        pins.d1.into_output(&mut pins.ddr),
        57600.into_baudrate(),
    );

    unsafe { avr_device::interrupt::enable() };

    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();

    loop {
        ufmt::uwriteln!(&mut serial, "counter value: {}\r", millis()).void_unwrap();
        arduino_uno::delay_ms(200);
    }
}
