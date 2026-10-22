//! `cargo run -p eval -- <collection> [--deck NAME]`: score scheduler variants on your own
//! review history, or on one user of the FSRS dataset (`.../user_id=N/data.parquet`). See the
//! crate docs for what this can and can't tell you.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use eval::{DayClock, by_intake, fit, samples, score, split_by_time};
use jiff::tz::TimeZone;

#[derive(Parser)]
#[command(about = "Score scheduler variants on a real review history")]
struct Args {
    /// An Anki export (.colpkg, .apkg), a bare collection file, or a revlogs `.parquet` file
    /// from anki-revlogs-10k.
    collection: PathBuf,
    /// Only cards whose deck name contains this (case-insensitive), e.g. "japanese". Anki
    /// collections only: the dataset has no deck names.
    #[arg(long)]
    deck: Option<String>,
    /// Time zone the reviews were done in (default: this machine's). Anki collections only: the
    /// dataset's days are already local.
    #[arg(long)]
    tz: Option<String>,
    /// Hour the learning day rolls over. Anki collections only.
    #[arg(long, default_value_t = 4)]
    rollover: i8,
    /// Share of the newest reviews held out for testing.
    #[arg(long, default_value_t = 0.2)]
    test_fraction: f64,
    /// Skip fitting parameters (it takes a while on big collections).
    #[arg(long)]
    no_fit: bool,
}

/// Lower bounds of the intake slices: cards introduced that day.
const INTAKE_EDGES: [u32; 5] = [1, 6, 11, 21, 41];

#[tokio::main]
async fn main() -> ExitCode {
    match run(Args::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let tz = match &args.tz {
        Some(name) => TimeZone::get(name)?,
        None => TimeZone::system(),
    };
    let clock = DayClock {
        tz,
        rollover_hour: args.rollover,
    };

    let reviews = if args.collection.extension().is_some_and(|e| e == "parquet") {
        if args.deck.is_some() {
            return Err("--deck needs an Anki collection; the dataset has no deck names".into());
        }
        eval::revlogs_10k::load(&args.collection)?
    } else {
        let collection = eval::anki::load(&args.collection, &clock).await?;
        match &args.deck {
            Some(d) => collection.in_deck(d),
            None => collection.reviews,
        }
    };
    let all = samples(&reviews);
    let split = split_by_time(all, args.test_fraction);
    let (Some(first_test), Some(last)) = (split.test.first(), split.test.last()) else {
        return Err("not enough reviews to hold any out for testing".into());
    };
    println!(
        "{} reviews → {} train / {} test targets",
        reviews.len(),
        split.train.len(),
        split.test.len()
    );
    println!(
        "test window: the last {} learning days\n",
        last.day - first_test.day + 1
    );

    let mut variants: Vec<(&str, Vec<f32>)> = vec![("FSRS defaults", Vec::new())];
    if !args.no_fit {
        match fit(&split.train) {
            Ok(p) => variants.push(("fitted on train", p)),
            Err(e) => println!("(couldn't fit parameters: {e})\n"),
        }
    }

    println!("{:<18} {:>9} {:>11}", "variant", "log loss", "RMSE(bins)");
    for (name, params) in &variants {
        let m = score(params, &split.test)?;
        println!(
            "{:<18} {:>9.4} {:>10.2}%",
            name,
            m.log_loss,
            m.rmse_bins * 100.0
        );
    }
    println!(
        "\nlower is better for both; RMSE(bins) is how far predicted recall is from actual recall"
    );

    let (name, params) = variants.last().expect("defaults are always there");
    println!("\nrecall by cards introduced that day ({name}, test set)");
    println!(
        "{:<8} {:>6} {:>10} {:>8} {:>7}",
        "new/day", "n", "predicted", "actual", "gap"
    );
    for b in by_intake(params, &split.test, &INTAKE_EDGES)? {
        if b.n == 0 {
            continue;
        }
        let gap = (b.actual - b.predicted) * 100.0;
        println!(
            "{:<8} {:>6} {:>9.1}% {:>7.1}% {:>+6.1}pp",
            b.label,
            b.n,
            b.predicted * 100.0,
            b.actual * 100.0,
            gap
        );
    }
    println!(
        "\ncompare the gaps between rows: if they drop as new/day grows, big intake days fade faster than the model expects"
    );
    Ok(())
}
