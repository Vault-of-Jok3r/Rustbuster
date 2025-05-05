use std::fs::File;
use std::io::{self, BufRead};

use clap::Parser;
use colored::*;
use reqwest;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long)]
    wordlist: String,

    #[arg(short, long, default_value_t = 1)]
    threads: usize,
}

fn wordlist(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let lines = reader.lines().filter_map(Result::ok).collect();
    Ok(lines)
}

fn main() {
    let args = Args::parse();

    println!(r#"  
    ________              ___________               _____             
    ___  __ \___  __________  /___  /_____  __________  /_____________
    __  /_/ /  / / /_  ___/  __/_  __ \  / / /_  ___/  __/  _ \_  ___/
    _  _, _// /_/ /_(__  )/ /_ _  /_/ / /_/ /_(__  )/ /_ /  __/  /    
    /_/ |_| \____/ /____/ \__/ /_____/\____/ /____/ \__/ \___//_/                                                                   
    "#);

    println!("=======================================================================\n");

    println!("Parameters:\n");
    println!("{}", format!("[*] URL: {}", args.url.blue().bold()).blue().bold());
    println!("{}", format!("[*] Wordlist: {}", args.wordlist.blue().bold()).blue().bold());
    println!("{}", format!("[*] Threads: {}", args.threads.to_string().blue().bold()).blue().bold());

    println!("\n=======================================================================\n");

    let entries = wordlist(&args.wordlist).expect("Impossible de lire la wordlist");

    let client = reqwest::blocking::Client::builder()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/122.0.0.0 Safari/537.36")
    .build()
    .expect("Failed to build HTTP client");

    for entry in &entries {
        let full_url = format!("{}/{}", args.url.trim_end_matches('/'), entry);

        match client.get(&full_url).send() {
            Ok(response) => {
                let status = response.status();
                let code = status.as_u16();
                let size = response.text().unwrap_or_default().len();

                let status_str = format!("Status: {}", code);
                let colored_status = if code >= 200 && code < 300 {
                    status_str.green()
                } else if code >= 300 && code < 400 {
                    status_str.blue()
                } else if code == 403 {
                    status_str.truecolor(255, 140, 0)
                } else {
                    status_str.red()
                };

                if code >= 200 && code < 400 {
                    println!(
                        "{:<20} ({}) [Size: {}] [→ {}]",
                        format!("./{}", entry),
                        colored_status,
                        size,
                        full_url.blue()
                    );
                } else {
                    println!(
                        "{:<20} ({}) [Size: {}]",
                        format!("./{}", entry),
                        colored_status,
                        size
                    );
                }
            }
            Err(err) => {
                println!(
                    "{:<20} ({})",
                    format!("./{}", entry),
                    format!("Error: {}", err).red()
                );
            }
        }
    }

    println!("\n=======================================================================");
    println!("Finished scanning {} entries.", entries.len());
    println!("=======================================================================\n");
    println!("Thanks for using RustBuster!\n");
    
}
