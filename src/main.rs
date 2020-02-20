#![forbid(unsafe_code)]

extern crate clap;
extern crate crypto;
extern crate failure;
extern crate filebuffer;
extern crate indicatif;
extern crate reqwest;
extern crate toml;
extern crate url;

use chrono::{Duration, Local, NaiveDate};
use clap::{App, Arg};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use failure::{err_msg, Error};
use filebuffer::FileBuffer;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use toml::Value;
use url::Url;

const UPSTREAM_URL: &str = "https://static.rust-lang.org/";

fn file_sha256(file_path: &Path) -> Option<String> {
    let file = Path::new(file_path);
    if file.exists() {
        let buffer = FileBuffer::open(&file).unwrap();
        let mut hasher = Sha256::new();
        hasher.input(&buffer);
        Some(hasher.result_str())
    } else {
        None
    }
}

fn download(dir: &str, path: &str) -> Result<PathBuf, Error> {
    let manifest = format!("{}{}", UPSTREAM_URL, path);
    let mut response = reqwest::blocking::get(&manifest)?;
    let mirror = Path::new(dir);
    let file_path = mirror.join(&path);
    create_dir_all(file_path.parent().unwrap())?;
    let mut dest = File::create(file_path)?;

    println!("File /{} downloading", path);
    let length = match response.content_length() {
        None => return Err(err_msg("Not found")),
        Some(l) => l,
    };
    let pb = ProgressBar::new(length);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} (ETA {eta_precise})")
        .progress_chars("#>-"));

    let mut buffer = [0u8; 4096];
    let mut read = 0;

    while read < length {
        let len = response.read(&mut buffer)?;
        dest.write(&buffer[..len])?;
        read = read + len as u64;
        pb.set_position(read);
    }

    pb.finish_and_clear();
    println!("File /{} downloaded", path);
    Ok(mirror.join(path))
}

fn main() {
    let args = App::new("rustup-mirror")
        .version("0.3.1")
        .author("Jiajie Chen <noc@jiegec.ac.cn>")
        .about("Make a mirror for rustup")
        .arg(
            Arg::with_name("orig")
                .short("o")
                .long("orig")
                .value_name("orig_path")
                .help("Where to store original manifest, e.g. ./orig")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("mirror")
                .short("m")
                .long("mirror")
                .value_name("mirror_path")
                .help("Where to store mirror files, e.g. ./mirror")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("mirror_url")
                .help("Where mirror is served, e.g. http://127.0.0.1:8000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("gc")
                .short("g")
                .long("gc")
                .value_name("garbage_collect_days")
                .help("Keep how many days of nightly toolchains, e.g. 365")
                .takes_value(true),
        )
        .get_matches();

    let orig_path = args.value_of("orig").unwrap_or("./orig");
    let mirror_path = args.value_of("mirror").unwrap_or("./mirror");
    let mirror_url = args.value_of("url").unwrap_or("http://127.0.0.1:8000");
    let gc_days = args.value_of("gc");

    let mut all_targets = HashSet::new();

    // Garbage collect old nightly versions
    if let Some(days) = gc_days {
        let days: i64 = days.parse().expect("days integer");
        let mirror = Path::new(mirror_path);
        let dist = mirror.join("dist");
        let mut day = Local::today().naive_local();
        day -= Duration::days(days);
        println!("Nightly before {} will be deleted", day);
        for path in read_dir(dist).expect("read_dir") {
            let path = path.unwrap();
            let file_type = path.file_type().unwrap();
            if file_type.is_dir() {
                if let Ok(file_name) = path.file_name().into_string() {
                    if let Ok(date) = NaiveDate::parse_from_str(&file_name, "%Y-%m-%d") {
                        if date < day {
                            println!("Removing Nightly of Date: {}", date);
                            remove_dir_all(path.path()).expect("remove dir");
                        }
                    }
                }
            }
        }
    }

    // Fetch rust components
    let channels = ["stable", "beta", "nightly"];
    for channel in channels.iter() {
        let name = format!("dist/channel-rust-{}.toml", channel);
        let file_path = download(orig_path, &name).unwrap();
        let sha256_name = format!("dist/channel-rust-{}.toml.sha256", channel);
        let sha256_file_path = download(orig_path, &sha256_name).unwrap();

        let mut file = File::open(file_path.clone()).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let mut sha256_file = File::open(sha256_file_path.clone()).unwrap();
        let mut sha256_data = String::new();
        sha256_file.read_to_string(&mut sha256_data).unwrap();
        assert_eq!(
            file_sha256(file_path.as_path()).unwrap(),
            &sha256_data[..64]
        );

        let mut value = data.parse::<Value>().unwrap();
        assert_eq!(value["manifest-version"].as_str(), Some("2"));
        println!(
            "Channel {} date {}",
            channel,
            value["date"].as_str().unwrap()
        );

        let pkgs = value["pkg"].as_table_mut().unwrap();
        let keys: Vec<String> = pkgs.keys().cloned().collect();
        for pkg_name in keys {
            let pkg = pkgs.get_mut(&pkg_name).unwrap().as_table_mut().unwrap();
            let pkg_targets = pkg.get_mut("target").unwrap().as_table_mut().unwrap();
            let targets: Vec<String> = pkg_targets.keys().cloned().collect();
            for target in targets {
                let pkg_target = pkg_targets
                    .get_mut(&target)
                    .unwrap()
                    .as_table_mut()
                    .unwrap();
                if pkg_target["available"].as_bool().unwrap() {
                    all_targets.insert(target.clone());

                    let prefixes = ["", "xz_"];
                    for prefix in prefixes.iter() {
                        let url =
                            Url::parse(pkg_target[&format!("{}url", prefix)].as_str().unwrap())
                                .unwrap();
                        let mirror = Path::new(mirror_path);
                        let file_name = url.path().replace("%20", " ");
                        let file = mirror.join(&file_name[1..]);

                        let hash_file = mirror.join(format!("{}.sha256", &file_name[1..]));
                        let hash_file_cont =
                            File::open(hash_file.clone()).ok().and_then(|mut f| {
                                let mut cont = String::new();
                                f.read_to_string(&mut cont).ok().map(|_| cont)
                            });

                        let hash_file_missing = hash_file_cont.is_none();
                        let mut hash_file_cont =
                            hash_file_cont.or_else(|| file_sha256(file.as_path()));

                        let chksum_upstream =
                            pkg_target[&format!("{}hash", prefix)].as_str().unwrap();

                        let need_download = match hash_file_cont {
                            Some(ref chksum) => chksum_upstream != chksum,
                            None => true,
                        };

                        if need_download {
                            download(mirror_path, &file_name[1..]).unwrap();
                            hash_file_cont = file_sha256(file.as_path());
                            assert_eq!(
                                Some(chksum_upstream),
                                hash_file_cont.as_ref().map(|s| s.as_str())
                            );
                        } else {
                            println!("File {} already downloaded, skipping", file_name);
                        }

                        if need_download || hash_file_missing {
                            File::create(hash_file)
                                .unwrap()
                                .write_all(hash_file_cont.unwrap().as_bytes())
                                .unwrap();
                            println!("Writing chksum for file {}", file_name);
                        }

                        pkg_target.insert(
                            format!("{}url", prefix),
                            Value::String(format!("{}{}", mirror_url, file_name)),
                        );
                    }
                }
            }
        }

        let output = value.to_string();
        let path = Path::new(mirror_path).join(&name);
        create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(path.clone()).unwrap();
        println!("Producing /{}", name);
        file.write_all(output.as_bytes()).unwrap();

        let sha256_new_file = file_sha256(&path).unwrap();
        let sha256_new_file_path = Path::new(mirror_path).join(&sha256_name);
        let mut file = File::create(sha256_new_file_path.clone()).unwrap();
        println!("Producing /{}", sha256_name);
        file.write_all(format!("{}  channel-rust-{}.toml", sha256_new_file, channel).as_bytes())
            .unwrap();

        let date = value["date"].as_str().unwrap();

        let alt_name = format!("dist/{}/channel-rust-{}.toml", date, channel);
        let alt_path = Path::new(mirror_path).join(&alt_name);
        create_dir_all(alt_path.parent().unwrap()).unwrap();
        copy(path, alt_path).unwrap();
        println!("Producing /{}", alt_name);

        let alt_sha256_new_file_name =
            format!("dist/{}/channel-rust-{}.toml.sha256", date, channel);
        let alt_sha256_new_file_path = Path::new(mirror_path).join(&alt_sha256_new_file_name);
        copy(sha256_new_file_path, alt_sha256_new_file_path).unwrap();
        println!("Producing /{}", alt_sha256_new_file_name);
    }

    // Fetch rustup self update
    println!("Downloading rustup self update manifest...");
    let self_update_manifest_path = download(orig_path, "rustup/release-stable.toml").unwrap();

    let mut self_update_manifest = File::open(self_update_manifest_path.clone()).unwrap();
    let mut self_update_manifest_data = String::new();
    self_update_manifest
        .read_to_string(&mut self_update_manifest_data)
        .unwrap();

    let self_update_manifest_val = self_update_manifest_data.parse::<Value>().unwrap();
    assert_eq!(
        self_update_manifest_val["schema-version"].as_str(),
        Some("1")
    );

    let self_version = self_update_manifest_val["version"].as_str().unwrap();

    for target in all_targets {
        let is_windows = target.find("windows").is_some();

        let ext = if is_windows { ".exe" } else { "" };

        if download(
            mirror_path,
            &format!(
                "rustup/archive/{}/{}/rustup-init{}",
                self_version, target, ext
            ),
        )
        .is_err()
        {
            println!("Failed to fetch rustup-init for target {}, ignored", target);
        }
    }

    copy(
        self_update_manifest_path,
        Path::new(mirror_path).join("rustup/release-stable.toml"),
    )
    .unwrap();
}
