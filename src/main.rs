mod generate_log;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;
use dialoguer::{Input, Select};
use std::error::Error;
use rayon::prelude::*;
struct Stats {
    total_lines: usize,
    info: usize,
    warning: usize,
    error: usize,
    invalid: usize,
}
fn main() {
    println!("---------------LOGANALYZER 2000----------------");
    if let Err(e) = run() {
        eprintln!("Noget gik galt: {e}");
    }
}
fn run() -> Result<(), Box<dyn Error>> {
    let input: String = Input::new()
        .with_prompt("Angiv sti til logfil(er) (komma-separeret)")
        .interact_text()?;

    let paths: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let start = Instant::now();

    // Parallel analyse
    let results = analyze_multiple_files(&paths);

    println!("\nAlle filer analyseret på: {:?}", start.elapsed());

    // Vis opsummering for hver fil
    for (p, result) in &results {
        match result {
            Ok(s) => show_summary(p, s),
            Err(e) => eprintln!("Fejl ved analyse af filen {p}: {e}"),
        }
    }

    if paths.len() > 1 {
        let merged = merge_stats(&results);
        println!("=== Samlet statistik for {} filer ===", paths.len());
        println!("Total linjer:  {}", merged.total_lines);
        println!("INFO:          {}", merged.info);
        println!("WARNING:       {}", merged.warning);
        println!("ERROR:         {}", merged.error);
        println!("Ugyldige:      {}", merged.invalid);
        println!();
    }

    loop {
        let menu_items = vec![
            "Vis opsummering",
            "Søg efter ord (i alle filer)",
            "Filtrér på INFO/WARNING/ERROR (i alle filer)",
            "Top 5 fejlbeskeder (i alle filer)",
            "Afslut"
        ];

        let choice = Select::new()
            .with_prompt("Vælg en handling")
            .items(&menu_items)
            .default(0)
            .interact()?;

        match choice {
            0 => {
                let start = Instant::now();
                for (p, result) in &results {
                    if let Ok(s) = result {
                        show_summary(p, s);
                    }
                }
                if paths.len() > 1 {
                    let merged = merge_stats(&results);
                    show_summary("(Samlet)", &merged);
                }
                println!("Analysen tog {:?}", start.elapsed());
            },
            1 => {
                let start = Instant::now();
                for p in &paths {
                    println!("\n--- {} ---", p);
                    search_word(p)?;
                }
                println!("Analysen tog {:?}", start.elapsed());
            },
            2 => {
                let start = Instant::now();
                for p in &paths {
                    println!("\n--- {} ---", p);
                    filter_level(p)?;
                }
                println!("Analysen tog {:?}", start.elapsed());
            },
            3 => {
                let start = Instant::now();
                for p in &paths {
                    println!("\n--- {} ---", p);
                    top_5_errors(p)?;
                }
                println!("Analysen tog {:?}", start.elapsed());
            },
            4 => {
                println!("Farvel!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn analyze_log(path: &str) -> Result<Stats, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut stats = Stats {
        total_lines: 0,
        info: 0,
        warning: 0,
        error: 0,
        invalid: 0,
    };
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Fejl ved læsning af linje: {e}");
                continue;
            }
        };

        stats.total_lines += 1;
        if line.contains(" INFO ") {
            stats.info += 1;
        } else if line.contains(" WARNING ") {
            stats.warning += 1;
        } else if line.contains(" ERROR ") {
            stats.error += 1;
        } else {
            stats.invalid += 1;
        }
    }
    Ok(stats)

}

fn show_summary(path: &str, stats: &Stats ) {
    println!("\n=== Log-opsummering ===");
    println!("Fil:           {path}");
    println!("Total linjer:  {}", stats.total_lines);
    println!("INFO:          {}", stats.info);
    println!("WARNING:       {}", stats.warning);
    println!("ERROR:         {}", stats.error);
    println!("Ugyldige:      {}", stats.invalid);
    println!();
}

fn search_word(path: &str) -> Result<(), Box<dyn Error>> {
    let word: String = Input::new()
        .with_prompt("Søg efter ord")
        .interact_text()?;

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    println!("\nLinjer der indeholder '{word}':");

    for line in reader.lines() {
        let line = line?;
        if line.contains(&word) {
            println!("{line}");
        }
    }
    println!();
    Ok(())
}

fn filter_level(path: &str) -> Result<(), Box<dyn Error>> {
    let levels = vec!["INFO", "WARNING", "ERROR"];

    let choice = Select::new()
        .with_prompt("Vælg logniveau")
        .items(&levels)
        .default (0)
        .interact()?;

    let selected = levels[choice];

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    println!("\nLinjer med {selected}:");

    for line in reader.lines() {
        let line = line?;
        if line.contains(selected) {
            println!("{line}");
        }
    }

    println!();
    Ok(())
}

fn top_5_errors(path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut counts: HashMap<String, usize> = HashMap::new ();

    for line in reader.lines() {
        let line = line?;

        if line.contains(" ERROR ") {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 5 {
                let message = parts[4..].join(" ");

                * counts.entry(message).or_insert(0) += 1;
            }
        }
    }
    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n=== Top 5 fejlbekeder ===");

    for(i, (msg, count)) in sorted.iter().take(5).enumerate() {
        println!("{}, {}, ({} gange)", i + 1, msg, count)
    }

    println!();
    Ok(())
}

fn analyze_multiple_files(paths: &[String]) -> Vec<(String, Result<Stats, String>)> {
    paths.par_iter()
        .map(|path| {
            let result = analyze_log(path).map_err(|e| e.to_string());
            (path.clone(), result)
        })
        .collect()
}
fn merge_stats(results: &[(String, Result<Stats, String>)]) -> Stats {
    results.iter()
        .filter_map(| (_, r) | r.as_ref().ok())
        .fold(Stats {total_lines: 0, info: 0, warning: 0, error: 0, invalid: 0}, |mut acc, s| {
            acc.total_lines += s.total_lines;
            acc.info += s.info;
            acc.warning += s.warning;
            acc.error += s.error;
            acc.invalid += s.invalid;
            acc
        })
}