pub mod console;
pub mod uart;

pub fn stdin() -> console::Stdin {
    console::Stdin
}

pub fn stdout() -> console::Stdout {
    console::Stdout
}

