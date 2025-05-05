use std::fs::File;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long)]
    wordlist: String,

    #[arg(short, long, default_value_t = 1)]
    threads: usize,

    #[arg(short, long, help = "Enable verbose output (show all status codes)")]
    verbose: bool,

    #[arg(short, long, help = "Output results to a file")]
    output: Option<String>,
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

    if args.verbose {
        println!("{}", "[*] Verbose mode enabled\n".cyan().bold());
    }

    let entries = wordlist(&args.wordlist).expect("Impossible de lire la wordlist");

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .expect("Failed to configure thread pool");

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/122.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to build HTTP client");

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let results = Arc::new(Mutex::new(Vec::new()));

    entries.par_iter().for_each(|entry| {
        let full_url = format!("{}/{}", args.url.trim_end_matches('/'), entry);

        match client.get(&full_url).send() {
            Ok(response) => {
                let code = response.status().as_u16();
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

                let is_relevant = matches!(code, 200 | 301 | 302 | 307 | 308 | 403 | 401);

                if args.verbose || is_relevant {
                    let display_line = format!(
                        "{:<20} ({}) [Size: {}] [→ {}]",
                        format!("./{}", entry),
                        colored_status,
                        size,
                        full_url
                    );

                    pb.println(display_line.clone()); // proprement au-dessus de la barre

                    if let Some(_) = args.output {
                        let mut res = results.lock().unwrap();
                        res.push(display_line);
                    }
                }
            }
            Err(err) => {
                if args.verbose {
                    let error_line = format!(
                        "{:<20} ({})",
                        format!("./{}", entry),
                        format!("Error: {}", err).red()
                    );
                    pb.println(error_line.clone());

                    if let Some(_) = args.output {
                        let mut res = results.lock().unwrap();
                        res.push(error_line);
                    }
                }
            }
        }

        pb.inc(1);
    });

    pb.finish_with_message("Scan complete");

    if let Some(path) = args.output {
        let res = results.lock().unwrap();
        let mut file = File::create(&path).expect("Unable to create output file");
        for line in res.iter() {
            writeln!(file, "{}", line).expect("Unable to write to file");
        }
        println!("{}", format!("[*] Results saved to {}", path).green());
    }

    println!("\n=======================================================================");
    println!("Finished scanning {} entries.", entries.len());
    println!("=======================================================================\n");
    println!("Thanks for using RustBuster!\n");
}
