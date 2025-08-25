use clap::{Arg, ArgAction, arg, command};
use regex::Regex;
use std::time::{UNIX_EPOCH, Duration};
use chrono::prelude::DateTime;
use chrono::Utc;


static RE_PC: &str = r"\s*?PC\s*?:(0x4[0-2][0-9][0-9A-Fa-f]*)";
static RE_BT: &str = r"Backtrace:\s?(.*)";
static RE_IN: &str = r"^[0-9A-Fa-f:x ]+$";
static RE_OT: &str = r"[^0-9a-zA-Z](0x4[0-2][0-9][0-9A-Fa-f]{5})[^0-9a-zA-Z]";

struct LogTimestamp {
    time_utc: i64,
    time_boot: i64,
}

impl LogTimestamp {
    fn to_absolute(&self, logtime: i64) -> i64 {
        if self.time_boot == 0 {
            return logtime; // no sync yet
        }
        self.time_utc + (logtime - self.time_boot)
    }

    fn to_string(&self, logtime: i64) -> String {
        let d = UNIX_EPOCH + Duration::from_secs(self.to_absolute(logtime) as u64);
        DateTime::<Utc>::from(d).to_string()
    }
}

#[derive(Debug)]
struct StackEntry {
    name: String,
    addr: String,
}

#[derive(Debug)]
struct Backtrace {
    location: i64,
    timestamp: String,
    pc: String,
    stack: Vec<StackEntry>,
}

fn extract_backtraces(log: &str) -> Vec<Backtrace> {
    let mut backtraces: Vec<Backtrace> = Vec::new();
    let mut current_bt: Option<Backtrace> = None;

    let mut running_logtime: i64 = 0;
    let mut log_timestamp = LogTimestamp {
        time_utc: 0,
        time_boot: 0,
    };

    // precompile regexes
    let pc_reg = Regex::new(RE_PC).unwrap();
    let bt_reg = Regex::new(RE_BT).unwrap();
    let ts_reg: Regex = Regex::new(r"[A-Z] \((\d+)\)").unwrap();
    let sync_reg: Regex = Regex::new(r"updated time:\s*(\d+)").unwrap();

    for (lineno, line) in log.lines().enumerate() {
        if let Some(cap) = ts_reg.captures(line) {
            running_logtime = cap.get(1).unwrap().as_str().parse::<i64>().unwrap_or(0) / 1000; // convert ms to s
        }

        // special line containing new timestamp
        if line.contains("updated time:") {
            if let Some(cap) = ts_reg.captures(line) {
                log_timestamp.time_boot =
                    cap.get(1).unwrap().as_str().parse::<i64>().unwrap_or(0) / 1000; // convert ms to s
            }

            if let Some(caps) = sync_reg.captures(line) {
                log_timestamp.time_utc = caps.get(1).unwrap().as_str().parse::<i64>().unwrap_or(0);
            }
            continue;
        }

        // speedup search by skipping lines not containing relevant info
        if !line.contains("Backtrace:") && !line.contains("PC:") {
            continue;
        }

        if let Some(caps) = pc_reg.captures(line) {
            let pc = caps.get(1).unwrap().as_str().to_string();
            let _bt = current_bt
                .get_or_insert(Backtrace {
                    location: (lineno + 1) as i64,
                    timestamp: log_timestamp.to_string(running_logtime),
                    pc: String::new(),
                    stack: Vec::new(),
                })
                .pc = pc;
        } else if let Some(caps) = bt_reg.captures(line) {
            let bt_str = caps.get(1).unwrap().as_str();
            let bt = current_bt.get_or_insert(Backtrace {
                location: (lineno + 1) as i64,
                timestamp: log_timestamp.to_string(running_logtime),
                pc: String::new(),
                stack: Vec::new(),
            });
            if Regex::new(RE_IN).unwrap().is_match(bt_str) {
                bt_str
                    .split_whitespace()
                    .map(|entry|
                        // split string at ':' to separate function name and address
                        {
                            let mut parts = entry.split(':');
                            StackEntry {
                                name: parts.next().unwrap_or("").to_string(),
                                addr: parts.next().unwrap_or("").to_string(),
                            }
                        })
                    .for_each(|entry| bt.stack.push(entry));
            } else {
                let cap = regex::Regex::new(RE_OT).unwrap();

                let mut parts = cap.split(":");
                bt.stack.push(StackEntry {
                    name: parts.next().unwrap_or("").to_string(),
                    addr: parts.next().unwrap_or("").to_string(),
                })
            }
        }

        if current_bt.is_some() {
            let bt = current_bt.as_mut().unwrap();
            if !bt.pc.is_empty() && !bt.stack.is_empty() {
                backtraces.push(current_bt.take().unwrap());
                current_bt = None;
            }
        }
    }
    backtraces
}

impl Backtrace {
    fn print_unwrap(&self, name: &String, elf: &String) {
        let gdb_cmd = std::process::Command::new("addr2line")
            .arg("-pif") // pretty print functions
            .args(["-e", elf])
            .arg(self.pc.as_str())
            .args(self.stack.iter().map(|entry| entry.name.as_str()))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start addr2line process");

        let output = gdb_cmd
            .wait_with_output()
            .expect("Failed to wait on gdb process");
        if !output.status.success() {
            eprintln!("gdb process failed with status: {}", output.status);
            return;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Stacktrace {}", name);
        for (i, line) in stdout.lines().enumerate() {
            if i < self.stack.len() {
                println!("SP-{:2}: {} ({})", i, self.stack[i].addr, line);
            }
        }
        println!();
    }
}

fn main() {
    let matches = command!()
        .next_line_help(true)
        .about(
            "Parses function pointers of backtrace from esp-idf to functions names using addr2line",
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Path to log containing Backtrace:... and PC:...")
                .required(true),
        )
        .arg(
            Arg::new("elf")
                .short('e')
                .long("elf")
                .help("Path to elf-file")
                .required(true),
        )
        .arg(arg!(-v --verbose ... "Sets the level of verbosity").action(ArgAction::Count))
        .get_matches();

    let file = matches.get_one::<String>("file").unwrap();
    let elf = matches.get_one::<String>("elf").unwrap();

    let log = std::fs::read_to_string(file).expect("Failed to read log file");
    let backtraces: Vec<Backtrace> = extract_backtraces(&log);
    if backtraces.is_empty() {
        eprintln!("No backtraces found in log file");
        return;
    }

    for (i, backtrace) in backtraces.iter().enumerate() {
        if matches.get_count("verbose") > 0 {
            println!("Parsed backtrace: {:?}", backtrace);
        }
        backtrace.print_unwrap(
            &format!(
                "#{} @ offset {} ({})",
                i, backtrace.location, backtrace.timestamp
            ),
            elf,
        );
    }
}
