pub enum Command {
    Start,
    New(String),
    Delete(String),
}

impl Command {
    pub fn parse(text: &str) -> Result<Self, &'static str> {
        let mut parts = text.split_whitespace();
        let cmd = parts.next().ok_or("Empty command")?;
        let arg = parts.next();

        if parts.next().is_some() {
            return Err("Too much arguments");
        }

        match cmd {
            "/start" => Ok(Command::Start),
            "/new" => {
                let val = arg.ok_or("You have to specify an argument")?;
                Ok(Command::New(val.to_string()))
            }
            "/del" => {
                let val = arg.ok_or("You have to specify an argument")?;
                Ok(Command::Delete(val.to_string()))
            }
            _ => Err("Command not found"),
        }
    }
}
