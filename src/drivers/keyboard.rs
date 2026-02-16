use spin::Mutex;
use alloc::collections::VecDeque;
use x86_64::instructions::port::Port;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

const KEYBOARD_BUFFER_SIZE: usize = 256;

static KEYBOARD: Mutex<Option<KeyboardDriver>> = Mutex::new(None);

pub struct KeyboardDriver {
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet1>,
    buffer: VecDeque<u8>,
}

impl KeyboardDriver {
    fn new() -> Self {
        Self {
            keyboard: Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ),
            buffer: VecDeque::with_capacity(KEYBOARD_BUFFER_SIZE),
        }
    }

    fn add_byte(&mut self, byte: u8) {
        if self.buffer.len() < KEYBOARD_BUFFER_SIZE {
            self.buffer.push_back(byte);
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        self.buffer.pop_front()
    }

    fn process_scancode(&mut self, scancode: u8) {
        if let Ok(Some(key_event)) = self.keyboard.add_byte(scancode) {
            if let Some(key) = self.keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        self.add_byte(character as u8);
                    }
                    DecodedKey::RawKey(key) => {
                        // 特殊キーの処理
                        crate::println!("Raw key: {:?}", key);
                    }
                }
            }
        }
    }
}

pub fn init() {
    *KEYBOARD.lock() = Some(KeyboardDriver::new());
}

/// 割り込みハンドラから呼び出される
pub fn handle_interrupt() {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Some(keyboard) = KEYBOARD.lock().as_mut() {
        keyboard.process_scancode(scancode);
    }

    // 割り込みコントローラに通知
    unsafe {
        Port::<u8>::new(0x20).write(0x20);
    }
}

pub fn read_bytes(buf: &mut [u8]) -> usize {
    let mut keyboard = KEYBOARD.lock();
    if let Some(keyboard) = keyboard.as_mut() {
        let mut count = 0;
        while count < buf.len() {
            if let Some(byte) = keyboard.read_byte() {
                buf[count] = byte;
                count += 1;
            } else {
                break;
            }
        }
        count
    } else {
        0
    }
}

pub fn has_data() -> bool {
    let keyboard = KEYBOARD.lock();
    keyboard.as_ref().map_or(false, |k| !k.buffer.is_empty())
}
