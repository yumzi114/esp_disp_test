#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, 
    delay::Delay, 
    gpio::{self, Event, Input, PullDown, PullUp, IO}, 
    i2c::I2C, 
    peripherals::Peripherals, 
    prelude::*, 
    rtc_cntl::Rtc, 
    system::SystemClockControl, 
    timer::{TimerGroup, TimerInterrupts} 
};
use core::{borrow::Borrow, cell::RefCell, fmt::Debug};

use critical_section::Mutex;
// use embedded_hal::i2c::{I2c, Error};
use lcd1602_driver::{
    command::{DataWidth, MoveDirection, State},
    lcd::{self, Anim, Basic, Ext, FlipStyle, Lcd, MoveStyle},
    sender::I2cSender,
    utils::BitOps,
};
use heapless::String;
const HEART: [u8; 8] = [
    0b00000, 0b00000, 0b01010, 0b11111, 0b01110, 0b00100, 0b00000, 0b00000,
];

static BUTTON: Mutex<RefCell<Option<gpio::Gpio0<Input<PullUp>>>>> =
    Mutex::new(RefCell::new(None));


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut rtc = Rtc::new(peripherals.LPWR,None);
    let mut delay = Delay::new(&clocks);
    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(handler);
    let timerinter = TimerInterrupts::default();
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks,None);
    let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks,None);
    let mut wdt0 = timg0.wdt;
    let mut wdt1 = timg1.wdt;
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();
    esp_println::logger::init_logger_from_env();
    let mut led = io.pins.gpio4.into_push_pull_output();
    // #[cfg(any(feature = "esp32", feature = "esp32c2", feature = "esp32c2"))]
    let mut button = io.pins.gpio0.into_pull_up_input();
    
    // #[cfg(not(any(feature = "esp32", feature = "esp32c2", feature = "esp32c3")))]
    // let mut button = io.pins.gpio9.into_pull_down_input();
    // let button = io.pins.gpio0.into_pull_up_input();
    let mut i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio3,
        io.pins.gpio2,
        100.kHz(),
        &clocks,
        None,
    );
    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });
    led.set_low();
    // let mut del_var = 2000_u32.millis();
    
    let mut sender = I2cSender::new(&mut i2c, 0x27);
    let lcd_config = lcd::Config::default().set_data_width(DataWidth::Bit4);
    let mut delay1=delay.clone();
    let mut lcd= Lcd::new(&mut sender, &mut delay1, lcd_config, 10);
    lcd.write_graph_to_cgram(1, &HEART);
    let mut graph_data = lcd.read_graph_from_cgram(1);
    graph_data[1].set_bit(2);
    graph_data[2].set_bit(2);
    lcd.write_graph_to_cgram(2, &graph_data);
    lcd.set_cursor_blink_state(State::On);
    lcd.set_cursor_pos((1, 0));
    // lcd.offset_cursor_pos((1, 0));
    lcd.write_str_to_cur("whatup");
    lcd.set_cursor_blink_state(State::Off);
    lcd.set_cursor_state(State::Off);
    let s1: String<6> = String::try_from(" : ").unwrap();
    lcd.write_str_to_cur(s1.as_str());
    let s2: String<4> = String::try_from("Hz").unwrap();
    let mut num = 1;
    // lcd.set_backlight(State::Off);
    loop {
        // lcd.typewriter_write("yumi,", 250_000);
        critical_section::with(|cs| {
            if BUTTON.borrow_ref_mut(cs).as_mut().unwrap().is_low(){
                led.set_high();
                num+=1;
                let nums: String<10> = String::try_from(num).unwrap();
                // lcd.set_backlight(State::On);
                // 되는거 ---------------------------------
                lcd.set_cursor_pos((10, 0));
                // lcd.write_str_to_cur("whatup");
                // lcd.delay_ms(250);
                // delay.delay_millis(250);
                // lcd.write_str_to_cur(" : 100Mz");
                // lcd.write_str_to_cur(s1.as_str());
                lcd.write_str_to_cur(nums.as_str());
                lcd.write_str_to_cur(s2.as_str());
                // esp_println::println!("dasd");
                // lcd.delay_ms(150);
                
                delay.delay_millis(1);
                // 되는거 ---------------------------------
                // 테스트 ---------------------------------
                // lcd.set_cursor_pos((6, 0));

            }else{
                // lcd.set_backlight(State::Off);
                led.set_low();
            }
        });
        
        // led.toggle();
        delay.delay_millis(100);
    }
}
#[handler]
#[ram]
fn handler() {
    if critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        esp_println::println!("Button was the source of the interrupt");
    } else {
        esp_println::println!("Button was not the source of the interrupt");
    }

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}