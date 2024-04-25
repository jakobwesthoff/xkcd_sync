use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

mod cli_progress;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::cli_progress::ProgressBar;

#[derive(Deserialize, Serialize, Debug)]
struct Xkcd {
    month: String,
    link: String,
    year: String,
    news: String,
    safe_title: String,
    transcript: String,
    alt: String,
    title: String,
    day: String,
    num: usize,
    img: String,
}

type SyncState = HashMap<usize, Xkcd>;

fn fetch_json(url: &str) -> Result<Xkcd> {
    let reader = ureq::get(url)
        .call()
        .context(format!("fetching {url}"))?
        .into_reader();

    let xkcd: Xkcd =
        serde_json::from_reader(BufReader::new(reader)).context("deserializing xkcd json")?;

    Ok(xkcd)
}

fn build_json_url_for_num(num: usize) -> String {
    format!("http://xkcd.com/{num}/info.0.json")
}

fn create_image_file_path(num: usize, comic_url: &str, comic_dir: &str) -> Result<PathBuf> {
    let comic_file_name = comic_url.split('/').last().context(format!(
        "extracting filename from image url {url}",
        url = comic_url
    ))?;
    let mut comic_path_name = PathBuf::new();
    comic_path_name.push(comic_dir);
    comic_path_name.push(format!("{num:05}_{file}", file = comic_file_name));
    Ok(comic_path_name)
}

fn download_xkcd_image_to_dir(xkcd: &Xkcd, target_file: &Path) -> Result<()> {
    let img_reader = ureq::get(&xkcd.img)
        .call()
        .context(format!("fetching {url}", url = xkcd.img))?
        .into_reader();
    let writer =
        fs::File::create(target_file).context(format!("open {target_file:?} for writing"))?;

    std::io::copy(&mut BufReader::new(img_reader), &mut BufWriter::new(writer)).context(
        format!(
            "stream data from {url} to {file:?}",
            url = xkcd.img,
            file = target_file
        ),
    )?;
    Ok(())
}

fn main() -> Result<()> {
    // @TODO: Extract to commandline arguments
    let comic_dir = "comics";
    let sync_state_file = "xkcd_sync_state.json";

    fs::create_dir_all(comic_dir)
        .context(format!("create commic storage directory {comic_dir}"))?;

    println!("Opening {file} as sync state", file = sync_state_file);
    let mut sync_state = match fs::File::open(sync_state_file) {
        Ok(file) => serde_json::from_reader(BufReader::new(file)).context(format!(
            "deserializing sync state from {file}",
            file = sync_state_file
        ))?,
        Err(_) => SyncState::new(),
    };

    let pb = ProgressBar {
        full_chars: Vec::from(cli_progress::UNICODE_BAR_FULL_CHARS),
        empty_char: ' ',
        ..ProgressBar::default()
    };

    pb.update(0f32, "Fetching latest comic information...")?;

    let lastest_url = "https://xkcd.com/info.0.json";
    let latest = fetch_json(lastest_url)?;

    let mut updated = 0;
    let mut skipped = 0;
    for num in 1..=latest.num {
        let mut already_updated = false;
        if let Entry::Vacant(e) = sync_state.entry(num) {
            pb.update(
                num as f32 / latest.num as f32 * 100f32,
                &format!("Fetching comic metadata #{num}"),
            )?;
            let json_url = build_json_url_for_num(num);
            match fetch_json(&json_url) {
                Ok(xkcd) => {
                    e.insert(xkcd);
                    updated += 1;
                    already_updated = true;
                }
                Err(error) => {
                    println!(
                        "Error retrieving metadata for #{num}: {err}",
                        err = error.root_cause()
                    );
                    println!("Note: Skipping #{num} as it will be retieved next time.");
                    continue;
                }
            }
        }
        let xkcd = sync_state.get(&num).unwrap();

        let comic_target_path = create_image_file_path(num, &xkcd.img, comic_dir)?;
        if comic_target_path.try_exists().context(format!(
            "establishing whether {file:?} exists",
            file = comic_target_path
        ))? {
            skipped += 1;
        } else {
            pb.update(
                num as f32 / latest.num as f32 * 100f32,
                &format!("Fetching comic image #{num}"),
            )?;
            match download_xkcd_image_to_dir(xkcd, &comic_target_path) {
                Ok(_) => {
                    if !already_updated {
                        updated += 1;
                    }
                }
                Err(error) => {
                    println!(
                        "Error retrieving image for #{num}: {err}",
                        err = error.root_cause()
                    );
                    println!("Note: Skipping #{num} as it will be retieved next time.");
                    continue;
                }
            }
        }

        if updated > 0 && updated % 50 == 0 {
            pb.update(
                num as f32 / latest.num as f32 * 100f32,
                &format!("Saving sync state to {file}", file = sync_state_file),
            )?;
            let file = fs::File::create(sync_state_file)
                .context(format!("open {file} for writing", file = sync_state_file))?;
            serde_json::to_writer(BufWriter::new(file), &sync_state)
                .context("serialize sync state")?;
        }
    }

    println!("Saving sync state to {file}", file = sync_state_file);
    let file = fs::File::create(sync_state_file)
        .context(format!("open {file} for writing", file = sync_state_file))?;
    serde_json::to_writer(BufWriter::new(file), &sync_state).context("serialize sync state")?;

    println!("Finished sync run: Updated {updated} comics, skipped {skipped} comics.");

    Ok(())
}
