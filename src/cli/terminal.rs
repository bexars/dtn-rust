use log::*;
use linefeed;

use std::io;
use std::sync::Arc;

use linefeed::{Interface, Prompter, ReadResult};
// use linefeed::chars::escape_sequence;
use linefeed::command::COMMANDS;
use linefeed::complete::{Completer, Completion};
// use linefeed::inputrc::parse_text;
use linefeed::terminal::Terminal;
use msg_bus::*;
use crate::system::SystemModules;
use crate::bus::ModuleMsgEnum;
use crate::cla::{AdapterConfiguration, ClaConfiguration, ClaType};

type BusHandle = MsgBusHandle<SystemModules, ModuleMsgEnum>;


const HISTORY_FILE: &str = "linefeed.hst";

#[derive(Clone)]
enum Mode {
    Normal,
    Conf,
    ConfCla(AdapterConfiguration),
}






pub(super) fn start(file: Option<String>, bh: BusHandle) -> io::Result<()> {

    let file2 = file.clone();    
    let mut out: Box<dyn std::io::Write> = if let Some(file) = file2 {
        Box::new(std::fs::File::create(file).unwrap())
    } else {
        Box::new(std::io::stdout())
    };

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

    interface.set_completer(repeater);
    let mut nodename = futures::executor::block_on(crate::conf::get_nodename(&mut bh.clone()));
    interface.set_prompt(&format!("{}> ", &nodename))?;  

    if let Err(e) = interface.load_history(HISTORY_FILE) {
        if e.kind() == io::ErrorKind::NotFound {
            writeln!(out, "History file {} doesn't exist, not loading history.", HISTORY_FILE)?;
        } else {
            writeln!(out, "Could not load history file {}: {}", HISTORY_FILE, e)?;
        }
    }
   

    while let ReadResult::Input(line) = interface.read_line().unwrap() {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }
        let mut no_flag = false;
        let mode_l = mode.clone();
        let (mut cmd, mut args) = split_first_word(&line);
        if cmd == "no" { 
            let (c,a) = split_first_word(&args); 
            cmd = c;
            args = a;
            no_flag = true;
        }
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
                interface.set_prompt(&format!("{}(conf)> ", &nodename))?;  
                interface.set_completer(Arc::new(ConfCompleter));
                mode = Mode::Conf;
            }
            ("cla", Mode::Conf) if no_flag => {
                let (subcmd, mut name) = split_first_word(&args);
                    if name == "" { name = subcmd; }
                    futures::executor::block_on(crate::conf::del_cla_conf(&mut bh.clone(), name.to_string()));
            }
            ("cla", Mode::Conf) => {
                let (subcmd, args) = split_first_word(&args);
                let (name, args) = split_first_word(&args);
                let conf = futures::executor::block_on(crate::conf::get_cla_conf(&mut bh.clone()));
                let cla_conf = conf.adapters.get(name);
                
                match subcmd {
                    "loopback" => {
                        if let Some(cla_conf) = cla_conf {
                            if cla_conf.cla_type == ClaType::LoopBack {
                                mode = Mode::ConfCla(cla_conf.clone());
                            } else {
                                writeln!(out, "CLA {} is already defined, but not as loopback", name)?;
                            }
                        } else {
                            mode = Mode::ConfCla(AdapterConfiguration {name: String::from(name), cla_type: ClaType::LoopBack, ..Default::default()});
                        }
                        interface.set_prompt(&format!("{} (conf-cla-loopback:{})> ", &nodename, name))?;
                        interface.set_completer(Arc::new(ClaCompleter));
                    }
                    "stcp-listen" => {
                        if let Some(cla_conf) = cla_conf {
                            if let ClaType::StcpListener(_,_) = cla_conf.cla_type {
                                mode = Mode::ConfCla(cla_conf.clone());
                            } else {
                                writeln!(out, "CLA {} is already defined, but not as stcp-listen", name)?;
                            }
                        } else {
                            mode = Mode::ConfCla(AdapterConfiguration {name: String::from(name), cla_type: ClaType::StcpListener(String::from("0.0.0.0"),4556), ..Default::default()});
                        }
                        interface.set_prompt(&format!("{} (conf-cla-stcp-listen:{})> ", &nodename, args))?;
                        interface.set_completer(Arc::new(ClaCompleter));
                    }

                    _ => {}
                };
            }
            ("node", Mode::ConfCla(mut config)) => {
                config.peernode = args.to_owned();
                mode = Mode::ConfCla(config.clone());
                futures::executor::block_on(crate::conf::set_cla_conf(&mut bh.clone(), config)).unwrap();
            }
            ("address", Mode::ConfCla(mut config)) => {
                match config.cla_type {
                    ClaType::StcpListener(_address, ip) => {
                        config.cla_type = ClaType::StcpListener(args.to_string(), ip);  
                    }
                    ClaType::Stcp(_address, ip) => {
                        config.cla_type = ClaType::Stcp(args.to_string(), ip);  
                    }
                    ClaType::StcpIp(_address, ip, domain) => {
                        config.cla_type = ClaType::StcpIp(args.to_string(), ip, domain);  
                    }
                    _ => {}
                }
                mode = Mode::ConfCla(config);
            }
            ("port", Mode::ConfCla(mut config)) => {
                match config.cla_type {
                    ClaType::StcpListener(address, _port) => {
                        config.cla_type = ClaType::StcpListener(address, args.parse().unwrap());  
                    }
                    ClaType::Stcp(address, _port) => {
                        config.cla_type = ClaType::Stcp(address, args.parse().unwrap());  
                    }
                    ClaType::StcpIp(address, _port, domain) => {
                        config.cla_type = ClaType::StcpIp(address, args.parse().unwrap(), domain);  
                    }
                    _ => {}
                }
                mode = Mode::ConfCla(config);
            }



            ("shutdown", Mode::ConfCla(mut config)) if no_flag => {
                if config.shutdown {
                    config.shutdown = false;
                    futures::executor::block_on(crate::conf::set_cla_conf(&mut bh.clone(), config.clone())).unwrap();    
                    mode = Mode::ConfCla(config);
                }
            }
            ("shutdown", Mode::ConfCla(mut config)) => {
                if !config.shutdown {
                    config.shutdown = true;
                    futures::executor::block_on(crate::conf::set_cla_conf(&mut bh.clone(), config.clone())).unwrap();    
                    mode = Mode::ConfCla(config);
                }
            }

            ("nodename", Mode::Conf) => {
                nodename = args.to_string();
                futures::executor::block_on(crate::conf::set_nodename(&mut bh.clone(), args.to_string())).unwrap();    
            }
            ("exit", m) => {
                match m {
                    Mode::Normal => break,
                    Mode::Conf => { 
                        interface.set_prompt(&format!("{}>",nodename))?;
                        interface.set_completer(Arc::new(MainCompleter));
                        mode = Mode::Normal; 
                    },
                    Mode::ConfCla(cla_conf) => {
                        //TODO verify CLA 
                        //TODO order the CLA list
                        interface.set_prompt(&format!("{}(conf)>",nodename))?;
                        interface.set_completer(Arc::new(ConfCompleter));
                        futures::executor::block_on(crate::conf::set_cla_conf(&mut bh.clone(), cla_conf)).unwrap();
                        mode = Mode::Conf; 
                    },
                } 
            }
            ("halt", Mode::Normal) => {
                let _res = futures::executor::block_on(crate::system::halt(&mut bh.clone()));
            }
            ("save", Mode::Normal) => {
                let file_name = if args.len() > 0 { Some(args.to_owned()) } else { None };
                let res = futures::executor::block_on(crate::conf::save(&mut bh.clone(), file_name));
                if let Err(e) = res {
                    writeln!(out, "{}", e)?;
                } else { 
                    writeln!(out, "Success.")?; };
            }
            ("show", _) => {
                let (subcmd, _args) = split_first_word(&args);
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
            (_,_) if cmd.len() > 0 => { writeln!(out, "Command not found: {}", line)?; }
            (_,_) => {}
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
    ("nodename",         "Sets the visual nodename.  No effect on operation"),
    ("show",             "Display information"),
    // ("telnet",           "telnet [enabled:bool] <bind-address> <port>"),
    ("quit",             "Quit to command mode"),
];

static CLA_TYPES: &[(&str, &str)] = &[
    ("loopback",        "CLA that points back to this node"),
    ("stcp-listen",    "service that listens for stcp connections"),
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