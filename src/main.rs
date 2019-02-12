extern crate clap;
extern crate crypto;
extern crate filebuffer;
extern crate indicatif;
extern crate reqwest;
extern crate toml;
extern crate url;

use clap::{App, Arg};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use filebuffer::FileBuffer;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{create_dir_all, File};
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

fn download(dir: &str, path: &str) -> PathBuf {
    let real_path = path.replace("%20", " ");
    let manifest = format!("{}{}", UPSTREAM_URL, real_path);
    let mut response = reqwest::get(&manifest).unwrap();
    let mirror = Path::new(dir);
    let file_path = mirror.join(&real_path);
    create_dir_all(file_path.parent().unwrap()).unwrap();
    let mut dest = File::create(file_path).unwrap();

    println!("File /{} downloading", real_path);
    let length = response.content_length().unwrap();
    let pb = ProgressBar::new(length);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} (ETA {eta_precise})")
        .progress_chars("#>-"));

    let mut buffer = [0u8; 4096];
    let mut read = 0;

    while read < length {
        let len = response.read(&mut buffer).unwrap();
        dest.write(&buffer[..len]).unwrap();
        read = read + len as u64;
        pb.set_position(read);
    }

    pb.finish_and_clear();
    println!("File /{} downloaded", real_path);
    mirror.join(real_path)
}

fn main() {
    let args = App::new("rustup-mirror")
        .version("0.1.1")
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
        .get_matches();

    let orig_path = args.value_of("orig_path").unwrap_or("./orig");
    let mirror_path = args.value_of("mirror_path").unwrap_or("./mirror");
    let mirror_url = args
        .value_of("mirror_url")
        .unwrap_or("http://127.0.0.1:8000");

    let channels = ["stable"];
    for channel in channels.iter() {
        let name = format!("dist/channel-rust-{}.toml", channel);
        let file_path = download(orig_path, &name);
        let sha256_name = format!("dist/channel-rust-{}.toml.sha256", channel);
        let sha256_file_path = download(orig_path, &sha256_name);

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
                    let prefixes = ["", "xz_"];
                    for prefix in prefixes.iter() {
                        let url =
                            Url::parse(pkg_target[&format!("{}url", prefix)].as_str().unwrap())
                                .unwrap();
                        let mirror = Path::new(mirror_path);
                        let file = mirror.join(&url.path()[1..]);

                        let mut need_download = true;
                        if let Some(sha256) = file_sha256(file.as_path()) {
                            if sha256 == pkg_target[&format!("{}hash", prefix)].as_str().unwrap() {
                                need_download = false;
                            }
                        }

                        if need_download {
                            download(mirror_path, &url.path()[1..]);
                        } else {
                            println!("File {} already downloaded", url.path());
                        }

                        pkg_target.insert(
                            format!("{}url", prefix),
                            Value::String(format!("{}{}", mirror_url, url.path())),
                        );
                    }
                }
            }
        }

        let output = value.to_string();
        let path = Path::new(mirror_path).join(&name);
        let mut file = File::create(path.clone()).unwrap();
        println!("Producing /{}", name);
        file.write_all(output.as_bytes()).unwrap();

        let sha256_new_file = file_sha256(&path).unwrap();
        let sha256_new_file_path = Path::new(mirror_path).join(&sha256_name);
        let mut file = File::create(sha256_new_file_path.clone()).unwrap();
        println!("Producing /{}", sha256_name);
        file.write_all(format!("{}  channel-rust-{}.toml", sha256_new_file, channel).as_bytes())
            .unwrap();
    }
}
