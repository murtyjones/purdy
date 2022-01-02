pub enum Command {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    Fill,
}

pub struct Commands(pub Vec<Command>);

impl Commands {
    pub fn new(c: Vec<Command>) -> Self {
        Commands(c)
    }
}
