use clap::{App, Arg};
use rustyline::error::ReadlineError;
use rustyline::Editor;
//use std::io::Read;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;
use rain_lang::parser::{
    ast::{Expr, Sexpr},
    parse_bool, parse_expr_list,
};

use std::fs::File;
//use std::io::BufReader;

/// A repl statement
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReplStatement<'a> {
    //Let(Let<'a>),
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

/// Parse a standalone rain expression
pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    let (rest, mut list) = parse_expr_list(true, input)?;
    let result = if list.len() == 1 {
        list.swap_remove(0)
    } else {
        Expr::Sexpr(Sexpr(list))
    };
    Ok((rest, result))
}

/// Parse a repl statement
pub fn parse_repl_statement(input: &str) -> IResult<&str, ReplStatement> {
    terminated(
        alt((
            map(parse_command, |c| ReplStatement::Command(c)),
            //map(parse_statement, |l| ReplStatement::Let(l)),
            map(parse_expr, |e| ReplStatement::Expr(e)),
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
    //builder: Builder<String>,
    show_parse: bool,
    show_definitions: bool,
    prompt: &'static str,
}

const DEFAULT_PROMPT: &'static str = ">>> ";

impl Repl {
    pub fn new() -> Repl {
        Repl {
            //builder: Builder::new(),
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
                let _file = match File::open(&f) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("Error opening file {:?}: {:?}", f, err);
                        return;
                    }
                };
                unimplemented!()
            }
            Command::PrintBuilderState(_pretty) => {
                unimplemented!()
                /*
                if pretty {
                    println!("Builder state: {:#?}", self.builder)
                } else {
                    println!("{:?}", self.builder)
                }
                */
            }
        }
    }
    pub fn handle_input(&mut self, input: &str) {
        println!("Got input line {}", input)
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
                repl.handle_input(line.as_str());
                rl.add_history_entry(line.as_str());
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
