use std::fs::File;
use std::io::{BufWriter, Write};

pub(crate) fn generate_log() -> std::io::Result<()> {
    // Vi opretter en ny fil med navnet big_log.txt.
    // Hvis filen allerede findes, bliver den overskrevet.
    let file = File::create("big_log.txt")?;

    // BufWriter gør skrivning hurtigere, især når vi skal skrive mange linjer.
    let mut writer = BufWriter::new(file);

    // Vi genererer 200.000 loglinjer.
    // Du kan ændre tallet, hvis du vil have en mindre eller større fil.
    for i in 0..200_000 {
        // Vi varierer logniveauet ud fra resten af i % 10.
        // På den måde får vi både INFO, WARNING og ERROR i filen.
        let line = match i % 10 {
            0 => format!(
                "2026-04-19 08:{:02}:00 ERROR payment-service Timeout order_id={}\n",
                i % 60,
                i
            ),
            1 | 2 => format!(
                "2026-04-19 08:{:02}:00 WARNING auth-service Too many attempts user_id={}\n",
                i % 60,
                i
            ),
            _ => format!(
                "2026-04-19 08:{:02}:00 INFO catalog-service Product viewed product_id={}\n",
                i % 60,
                i
            ),
        };

        // Vi skriver linjen til filen.
        writer.write_all(line.as_bytes())?;
    }

    // flush sikrer, at alt bliver skrevet helt færdigt til filen.
    writer.flush()?;

    println!("Filen big_log.txt er blevet oprettet.");

    Ok(())
}