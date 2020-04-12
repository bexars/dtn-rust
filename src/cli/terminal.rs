use log::*;
use linefeed;
use rand;

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};

use linefeed::{Interface, Prompter, ReadResult};
use linefeed::chars::escape_sequence;
use linefeed::command::COMMANDS;
use linefeed::complete::{Completer, Completion};
// use linefeed::inputrc::parse_text;
use linefeed::terminal::Terminal;
use msg_bus::*;
use crate::router::RouterModule;
use crate::bus::ModuleMsgEnum;
use crate::cla::ClaType;

type BusHandle = MsgBusHandle<RouterModule, ModuleMsgEnum>;


const HISTORY_FILE: &str = "linefeed.hst";

#[derive(Clone)]
enum Mode {
    Normal,
    Conf,
    ConfCla(ClaType, String),
}






pub(super) fn start(file: Option<String>, bh: BusHandle) -> io::Result<()> {

    let file2 = file.clone();    
    let mut out: Box<dyn std::io::Write> = if let Some(file) = file2 {
        Box::new(std::fs::File::create(file).unwrap())
    } else {
        Box::new(std::io::stdout())
    };

    writeln!(out, "Hello World")?;

    let interface = if let Some(file) = file {
        // Interface::with_term("my-app", mortal::unix::OpenTerminalExt::from_path(file).map(linefeed::DefaultTerminal).unwrap()
        Interface::with_term("my-app", linefeed::DefaultTerminal::new_path(file).unwrap()).unwrap()
    } else {
        Interface::new("my-application").unwrap()

    };


    // let interface = Arc::new(Interface::new("demo").unwrap());
    // println!("After new interface");
    let repeater = Arc::new(MainCompleter);
    let mut mode = Mode::Normal;
    // level.push(mode);

    writeln!(out, "dtnrouter - ALPHA Unstable")?;
    writeln!(out, "Enter \"help\" for a list of commands.")?;
    writeln!(out, "Press Ctrl-D or enter \"quit\" to exit.")?;
    writeln!(out, "")?;
    debug!("Printed the info banner for CLI");

//    interface.set_completer(repeater);
//    interface.set_prompt("> ")?;  //TODO set a Hostname in conf

    // if let Err(e) = interface.load_history(HISTORY_FILE) {
    //     if e.kind() == io::ErrorKind::NotFound {
    //         writeln!(out, "History file {} doesn't exist, not loading history.", HISTORY_FILE);
    //     } else {
    //         writeln!(out, "Could not load history file {}: {}", HISTORY_FILE, e);
    //     }
    // }

    while let ReadResult::Input(line) = interface.read_line().unwrap() {
        writeln!(out, "In: {}", line)?;
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }
        let mut mode_l = mode.clone();
        let (cmd, args) = split_first_word(&line);

        match (cmd, mode_l) {
            ("help", Mode::Normal) => {
                writeln!(out, "dtn commands:")?;
                writeln!(out)?;
                for &(cmd, help) in MAIN_COMMANDS {
                    writeln!(out, "  {:15} - {}", cmd, help)?;
                }
                writeln!(out)?;
            }
            ("list-commands", Mode::Normal) => {
                for cmd in COMMANDS {
                    writeln!(out, "{}", cmd)?;
                }
            }
            ("show", _) => {
                let (subcmd, args) = split_first_word(&args);
                match subcmd {
                    "help" => {
                        writeln!(out, "dtn commands:")?;
                        writeln!(out, )?;
                        for &(cmd, help) in SHOW_COMMANDS {
                            writeln!(out, "show {:15} - {}", cmd, help)?;
                        }
                        writeln!(out)?;
                    }
                    "configuration" => {
                        let res = futures::executor::block_on(crate::conf::get_conf(&mut bh.clone()));
                        writeln!(out, "{}", res)?;
                    }
                    _ => { 
                        writeln!(out, "read input: {:?}", line)?; 
                    }

                }
            }
            ("halt", Mode::Normal) => {
                let res = futures::executor::block_on(crate::router::halt(&mut bh.clone()));
            }
            ("history", _) => {
                let w = interface.lock_writer_erase()?;

                for (i, entry) in w.history().enumerate() {
                    writeln!(out, "{}: {}", i, entry)?;
                }
            }
            ("save-history",_) => {
                if let Err(e) = interface.save_history(HISTORY_FILE) {
                    writeln!(out, "Could not save history file {}: {}", HISTORY_FILE, e)?;
                } else {
                    writeln!(out, "History saved to {}", HISTORY_FILE)?;
                }
            }
            ("configuration", Mode::Normal) => {
                    interface.set_prompt("(conf)> ")?;
                    interface.set_completer(Arc::new(ConfCompleter));
                    mode = Mode::Conf;
                }
            ("cla", Mode::Conf) => {
                let (subcmd, args) = split_first_word(&args);
                match subcmd {
                    "loopback" => {
                        mode = Mode::ConfCla(ClaType::LoopBack, args.to_string());
                        interface.set_prompt(&format!("(conf-cla-loopback:{}> ", args))?;
                        interface.set_completer(Arc::new(ClaCompleter));
                    }
                    _ => {}
                };
            }
            ("exit", m) => {
                match m {
                    Mode::Normal => break,
                    Mode::Conf => { 
                        interface.set_prompt(">")?;
                        interface.set_completer(Arc::new(MainCompleter));
                        mode = Mode::Normal; 
                    },
                    Mode::ConfCla(_,_) => {
                        interface.set_prompt("(conf)>")?; 
                        mode = Mode::Conf; 
                    },
                } 
            }
            ("save", Mode::Normal) => {
                futures::executor::block_on(crate::conf::save(&mut bh.clone(), None));
            }
            ("telnet", Mode::Conf) => {
                let (enabled, args) = split_first_word(&args);
                let mut cli_conf = futures::executor::block_on(crate::conf::get_cli_conf(&mut bh.clone()));
                if enabled == "true" { cli_conf.telnet_enabled = true } else { cli_conf.telnet_enabled = false };
                futures::executor::block_on(crate::conf::set_cli_conf(&mut bh.clone(), cli_conf));

            }
            (_,_) => { writeln!(out, "read input: {:?}", line)?; }
        }
    }

    writeln!(out, "Goodbye.")?;

    Ok(())
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, "")
    }
}

static MAIN_COMMANDS: &[(&str, &str)] = &[
    ("help",             "You're looking at it"),
    ("configuration",    "Configuration mode"),
    ("list-commands",    "List command names"),
    ("history",          "Print history"),
    ("show",             "Display information"),
    ("halt",             "Stops all processes"),
    ("save",             "Write to disk"),
    ("quit",             "Quit the demo"),
];

static SHOW_COMMANDS: &[(&str, &str)] = &[
    ("configuration",             "Shows running configuration"),
];

static CONF_COMMANDS: &[(&str, &str)] = &[
    ("cla",              "<name> CL adapter configuration"),
    ("help",             "You're looking at it"),
    ("list-commands",    "List command names"),
    ("history",          "Print history"),
    ("show",             "Display information"),
    ("telnet",           "telnet [enabled:bool] <bind-address> <port>"),
    ("quit",             "Quit to command mode"),
];

static CLA_TYPES: &[(&str, &str)] = &[
    ("loopback",        "CLA that points back to this node"),
    ("stcp-service",    "service that listens for stcp connections"),
    ("stcp",            "CLA that sends to a specific node via stcp"),
    ("stcp-ip",         "CLA that sends to a node looked up by dtn srv records"),
];

static CLA_COMMANDS: &[(&str, &str)] = &[
    ("address",         "Hostname or IPv4/6 address to connect or listen to"),
    ("port",            "tcp port to connect or listen to"),
    ("node",            "dtn node of the peer (ip.earth for srv lookup)"),
];

struct MainCompleter;

impl<Term: Terminal> Completer<Term> for MainCompleter {
    fn complete(&self, word: &str, prompter: &Prompter<Term>,
            start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = prompter.buffer();

        let mut words = line[..start].split_whitespace();

        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in MAIN_COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            Some("show") => {
                let mut compls = Vec::new();

                if words.count() == 0 {
                    for &(cmd, _) in SHOW_COMMANDS {
                        if cmd.starts_with(word) {
                            compls.push(Completion::simple(cmd.to_owned()));
                        }
                    }
                }
                Some(compls)
            }
            _ => None
        }
    }
}

struct ConfCompleter;

impl<Term: Terminal> Completer<Term> for ConfCompleter {
    fn complete(&self, word: &str, prompter: &Prompter<Term>,
            start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = prompter.buffer();

        let mut words = line[..start].split_whitespace();

        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in CONF_COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            Some("show") => {
                let mut compls = Vec::new();

                if words.count() == 0 {
                    for &(cmd, _) in SHOW_COMMANDS {
                        if cmd.starts_with(word) {
                            compls.push(Completion::simple(cmd.to_owned()));
                        }
                    }
                }
                Some(compls)
            }
            Some("cla") => {
                let mut compls = Vec::new();

                if words.count() == 0 {
                    for &(cmd, _) in CLA_TYPES {
                        if cmd.starts_with(word) {
                            compls.push(Completion::simple(cmd.to_owned()));
                        }
                    }
                }
                Some(compls)
            }

            _ => None
        }
    }
}

struct ClaCompleter;

impl<Term: Terminal> Completer<Term> for ClaCompleter {
    fn complete(&self, word: &str, prompter: &Prompter<Term>,
            start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = prompter.buffer();

        let mut words = line[..start].split_whitespace();

        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in CLA_COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            Some("show") => {
                let mut compls = Vec::new();

                if words.count() == 0 {
                    for &(cmd, _) in SHOW_COMMANDS {
                        if cmd.starts_with(word) {
                            compls.push(Completion::simple(cmd.to_owned()));
                        }
                    }
                }
                Some(compls)
            }
            _ => None
        }
    }
}