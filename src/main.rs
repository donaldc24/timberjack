use std::fs::File;
use std::io::{self, BufRead, BufReader};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "timber")]
#[clap(about = "Timber: Fell Your Logs Fast", long_about = None)]
struct Args {
    /// Log file to analyze
    file: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    println!("Waking Lumberjacks! Chopping: {}", args.file);

    let file = File::open(&args.file)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{}", line?);
    }

    println!("Timber finished chopping the log!");
    Ok(())
}
