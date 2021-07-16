use std::error::Error;

use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Timelike};
use git2::{Repository, Signature};
use rayon::prelude::*;

type CountByHour = [usize; 24];

fn is_me(author: Signature) -> bool {
    if let Some(email) = author.email() {
        return email.contains("@hackery.site") || email.contains("@som.codes");
    }

    false
}

fn commit_hours(repo: Repository) -> Result<CountByHour, Box<dyn Error>> {
    let mut hours = [0usize; 24];
    // let mut commits = vec![];

    let commits = {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk
            .filter_map(|r| r.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
    };

    commits.filter(|c| is_me(c.author())).for_each(|c| {
        let t = c.time();

        let date_time: DateTime<Local> = DateTime::from_utc(
            NaiveDateTime::from_timestamp(t.seconds(), 0),
            FixedOffset::east(t.offset_minutes() * 60),
        );

        let hour = date_time.time().hour() as usize;
        hours[hour] += 1;
    });

    Ok(hours)
}

fn main() -> Result<(), Box<dyn Error>> {
    let dev_dir = std::env::var("DEV_DIR")
        .unwrap_or("/home/half-kh-hacker/Documents/Development".to_string());

    let counts = glob::glob(&(dev_dir + "/**/.git"))?
        .par_bridge()
        .filter_map(|dir_result| dir_result.ok())
        .filter_map(|git_dir| git_dir.parent().map(|d| d.to_owned()))
        .filter_map(|proj_dir| {
            eprintln!(
                "Processing: {}…",
                proj_dir
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("<unknown>")
            );
            Repository::open(proj_dir).ok()
        })
        .map(|git_repo| match commit_hours(git_repo) {
            Ok(hours) => hours,
            Err(_) => [0usize; 24],
        })
        .collect::<Vec<_>>();

    eprintln!();

    let sums = counts.iter().fold([0usize; 24], |a, b| {
        let mut result = [0usize; 24];
        for i in 0..24usize {
            result[i] = a[i] + b[i];
        }

        result
    });

    let total_count: usize = sums.iter().sum();
    let total_count = total_count;

    for i in 0..24 {
        let pct = (sums[i] as f64) / (total_count as f64) * 100.0;
        println!("[{:02}:xx] {:.3}%", i, pct)
    }

    Ok(())
}
