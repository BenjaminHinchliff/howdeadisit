use anyhow::Result;
use chrono::{offset::FixedOffset, DateTime, Local, NaiveDateTime, Utc, Duration};
use clap::{load_yaml, App};
use git2::Repository;
use lazy_static::lazy_static;

fn get_remote() -> String {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    return matches.value_of("URL").unwrap().to_string();
}

const CLONE_PATH: &str = "tmp";

lazy_static!{
    static ref DEAD_DURATION: Duration = Duration::weeks(4);
}

fn main() -> Result<()> {
    let remote_url = get_remote();

    rm_rf::ensure_removed(CLONE_PATH)?;
    {
        let repo = Repository::clone(&remote_url, CLONE_PATH)?;

        let last_commit = repo.find_commit(repo.head()?.peel_to_commit()?.id())?;

        let commit_time = last_commit.time();
        let ts = NaiveDateTime::from_timestamp(commit_time.seconds(), 0);
        let commit_datetime: DateTime<FixedOffset> =
            DateTime::from_utc(ts, FixedOffset::east(commit_time.offset_minutes() * 60));
        let time_since_commit =
            Local::now().with_timezone(&Utc) - commit_datetime.with_timezone(&Utc);
        if time_since_commit > *DEAD_DURATION {
            println!("no commits in the last {} weeks, the repo is probably dead", DEAD_DURATION.num_weeks());
        } else {
            println!("there has been a commit in the last {} weeks, the repo is probably alive", DEAD_DURATION.num_weeks());
        }
    }

    rm_rf::ensure_removed(CLONE_PATH)?;

    Ok(())
}
