use clap::{App, Arg};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io::Read;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::{Err, IResult};
use rain_ir::parser::{
    ast::{Expr, Statement},
    builder::Builder,
    parse_bool, parse_expr, parse_statement,
};

use std::fs::File;
use std::io::BufReader;

/// A repl statement
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReplStatement<'a> {
    Statement(Statement<'a>),
    Expr(Expr<'a>),
    Command(Command),
}

/// A repl command
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    Include(String),
    PrintBuilderState(bool),
    ShowDefinitions(bool),
    ShowParse(bool),
}

/// Parse a repl statement
pub fn parse_repl_statement(input: &str) -> IResult<&str, ReplStatement> {
    terminated(
        alt((
            map(parse_expr, |e| ReplStatement::Expr(e)),
            map(parse_statement, |s| ReplStatement::Statement(s)),
            map(parse_command, |c| ReplStatement::Command(c)),
        )),
        multispace0,
    )(input)
}

/// Parse a repl command
pub fn parse_command(input: &str) -> IResult<&str, Command> {
    delimited(
        delimited(multispace0, tag("#"), multispace0),
        alt((
            map(
                separated_pair(tag("show_parse"), multispace1, opt(parse_bool)),
                |(_, b)| Command::ShowParse(b.unwrap_or(true)),
            ),
            map(
                separated_pair(tag("builder_state"), multispace1, opt(parse_bool)),
                |(_, b)| Command::PrintBuilderState(b.unwrap_or(true)),
            ),
            map(
                separated_pair(tag("show_definitions"), multispace1, opt(parse_bool)),
                |(_, b)| Command::ShowDefinitions(b.unwrap_or(true)),
            ),
            map(
                separated_pair(
                    tag("include"),
                    multispace0,
                    delimited(tag("<"), take_until(">"), tag(">")),
                ),
                |(_, f)| Command::Include(String::from(f)),
            ),
        )),
        preceded(multispace0, tag(";")),
    )(input)
}

/// A very simple repl for the `rain` IR
#[derive(Debug)]
pub struct Repl {
    buffer: String,
    builder: Builder<String>,
    cursor: usize,
    show_parse: bool,
    show_definitions: bool,
    prompt: &'static str,
}

const DEFAULT_PROMPT: &'static str = ">>> ";

impl Repl {
    pub fn new() -> Repl {
        Repl {
            buffer: String::new(),
            builder: Builder::new(),
            cursor: 0,
            show_parse: false,
            show_definitions: false,
            prompt: DEFAULT_PROMPT,
        }
    }
    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::ShowParse(b) => {
                self.show_parse = b;
            }
            Command::ShowDefinitions(d) => {
                self.show_definitions = d;
            }
            Command::Include(f) => {
                let file = match File::open(&f) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("Error opening file {:?}: {:?}", f, err);
                        return;
                    }
                };
                let mut buf_reader = BufReader::new(file);
                match buf_reader.read_to_string(&mut self.buffer) {
                    Ok(bytes) => println!("Read {} bytes from {:?}", bytes, f),
                    Err(err) => eprintln!("Error reading file {:?}: {:?}", f, err),
                }
            }
            Command::PrintBuilderState(pretty) => {
                if pretty {
                    println!("Builder state: {:#?}", self.builder)
                } else {
                    println!("{:?}", self.builder)
                }
            }
        }
    }
    pub fn handle_line(&mut self, line: &str) -> bool {
        self.buffer.push_str(line);
        self.buffer.push('\n');
        self.prompt = DEFAULT_PROMPT;
        while self.buffer.len() > self.cursor {
            let command = {
                let (rest, statement) = match parse_repl_statement(&self.buffer[self.cursor..]) {
                    Ok(res) => res,
                    Err(err) => match err {
                        Err::Incomplete(_) => {
                            self.prompt = "";
                            return false;
                        }
                        err => {
                            eprintln!("Parse error: {:?}", err);
                            return true;
                        }
                    },
                };
                let begin = self.cursor;
                let end = self.buffer.len() - rest.len();
                self.cursor = end;
                match statement {
                    ReplStatement::Command(c) => Some(c),
                    ReplStatement::Statement(s) => {
                        match self.builder.build_statement(&s) {
                            Ok(_defs) => {
                                if self.show_parse {
                                    println!(
                                        "Parsed statement: {:?} => {:?}",
                                        &self.buffer[begin..end],
                                        s
                                    )
                                }
                                /*
                                if self.show_definitions {
                                    println!("Defined {} symbols:", defs.len());
                                    for def in defs.iter() {
                                        print!("{} = {}", def.name, def.value);
                                        if let Some(previous) = &def.previous {
                                            println!(" (previously {})", previous)
                                        } else {
                                            print!("\n")
                                        }
                                    }
                                }
                                */
                            }
                            Err(err) => println!(
                                "Error building let IR: {:#?}\n===========\nAST:\n{:?}\n",
                                err, s
                            ),
                        }
                        None
                    }
                    ReplStatement::Expr(e) => {
                        match self.builder.build_expr(&e) {
                            Ok(val) => {
                                if self.show_parse {
                                    println!(
                                        "Parsed expr: {:?} => {:?}",
                                        &self.buffer[begin..end],
                                        e
                                    )
                                }
                                println!("{}", val)
                            }
                            Err(err) => println!(
                                "Error building expr IR: {:#?}\n===========\nAST:\n{:?}\n",
                                err, e
                            ),
                        }
                        None
                    }
                }
            };
            if let Some(command) = command {
                self.handle_command(command)
            }
        }
        true
    }
    pub fn buffer(&self) -> &str {
        &self.buffer
    }
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}

fn main() {
    let matches = App::new("rain repl")
        .version("0.0")
        .author("Jad Ghalayini <jad.ghalayini@hotmail.com>")
        .about("repl for rain programs")
        .arg(
            Arg::with_name("history")
                .short("h")
                .long("history")
                .value_name("FILE")
                .help("Sets a file to save/load history")
                .takes_value(true),
        )
        .get_matches();
    let history = matches.value_of("history");
    let mut rl = Editor::<()>::new();
    let mut repl = Repl::new();
    if let Some(history) = history {
        if rl.load_history(history).is_err() {
            println!("No previous history loaded from {:?}.", history);
        } else {
            println!("Loaded history from {:?}.", history)
        }
    } else {
        println!("No previous history loaded.")
    }
    loop {
        let line = rl.readline(repl.prompt);
        match line {
            Ok(line) => {
                if repl.handle_line(line.as_str()) {
                    rl.add_history_entry(repl.buffer());
                    repl.clear_buffer();
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    if let Some(history) = history {
        rl.save_history(history).unwrap();
    }
}
