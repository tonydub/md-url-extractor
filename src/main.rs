use clap::{Arg, Command, value_parser};
use indicatif::{ProgressBar, ProgressStyle};
use pulldown_cmark::{Event, Parser, Tag};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use url::Url;
use walkdir::WalkDir;

struct Args {
    input_dir: PathBuf,
    output_format: OutputFormat,
    output_path: Option<PathBuf>,
    filter_domain: Option<String>,
    filter_protocol: Vec<String>,
}

enum OutputFormat {
    Stdout,
    Text,
    Csv,
    Html,
}

#[derive(Clone, Debug)]
struct UrlInfo {
    url: String,
    source_file: PathBuf,
    link_text: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();

    let markdown_files = scan_directory(&args.input_dir)?;

    let url_infos = extract_urls_from_files(markdown_files, &args)?;

    let processed_urls = process_urls(url_infos, &args)?;

    output_results(processed_urls, &args)?;

    Ok(())
}

fn parse_args() -> Args {
    let matches = Command::new("md-url-extractor")
        .version("1.0")
        .about("Extracts URLs from Markdown files")
        .arg(
            Arg::new("input_dir")
                .help("Directory containing Markdown files")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("output_format")
                .short('f')
                .long("format")
                .help("Output format (stdout, text, csv, html)")
                .value_parser(["stdout", "text", "csv", "html"])
                .default_value("stdout"),
        )
        .arg(
            Arg::new("output_path")
                .short('o')
                .long("output")
                .help("Output file path")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("filter_domain")
                .long("domain")
                .help("Filter URLs by domain"),
        )
        .arg(
            Arg::new("filter_protocol")
                .long("protocol")
                .help("Filter URLs by protocol (http, https, etc.)")
                .value_parser(["http", "https", "ftp", "file", "mailto"])
                .action(clap::ArgAction::Append)
                .default_values(["http", "https"]),
        )
        .get_matches();

    let input_dir = matches
        .get_one::<PathBuf>("input_dir")
        .expect("required")
        .clone();

    let output_format = match matches.get_one::<String>("output_format").unwrap().as_str() {
        "stdout" => OutputFormat::Stdout,
        "text" => OutputFormat::Text,
        "csv" => OutputFormat::Csv,
        "html" => OutputFormat::Html,
        _ => OutputFormat::Stdout,
    };

    let output_path = matches.get_one::<PathBuf>("output_path").cloned();
    let filter_domain = matches.get_one::<String>("filter_domain").cloned();
    let filter_protocol: Vec<String> = matches
        .get_many::<String>("filter_protocol")
        .unwrap_or_default()
        .cloned()
        .collect();

    Args {
        input_dir,
        output_format,
        output_path,
        filter_domain,
        filter_protocol,
    }
}

fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut markdown_files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            markdown_files.push(path.to_path_buf());
        }
    }

    Ok(markdown_files)
}

fn extract_urls_from_files(
    files: Vec<PathBuf>,
    _args: &Args,
) -> Result<Vec<UrlInfo>, Box<dyn std::error::Error>> {
    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files")?
            .progress_chars("##-"),
    );

    let url_infos = Arc::new(Mutex::new(Vec::new()));

    files.par_iter().for_each(|file| {
        let file_content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error reading file {:?}: {}", file, err);
                return;
            }
        };

        let file_urls = extract_urls_from_markdown(&file_content, file);

        if !file_urls.is_empty() {
            if let Ok(mut infos) = url_infos.lock() {
                infos.extend(file_urls);
            }
        }

        progress_bar.inc(1);
    });

    progress_bar.finish_with_message("URL extraction complete");

    let result = Arc::try_unwrap(url_infos)
        .expect("Failed to unwrap Arc")
        .into_inner()
        .expect("Failed to unwrap Mutex");

    Ok(result)
}

fn extract_urls_from_markdown(content: &str, source_file: &Path) -> Vec<UrlInfo> {
    let mut urls = Vec::new();
    let parser = Parser::new(content);

    let mut current_link: Option<(String, String)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Link(_, destination, _)) => {
                current_link = Some((destination.to_string(), String::new()));
            }
            Event::Text(text) if current_link.is_some() => {
                if let Some((_, ref mut link_text)) = current_link {
                    link_text.push_str(text.as_ref());
                }
            }
            Event::End(Tag::Link(_, _, _)) => {
                if let Some((url, link_text)) = current_link.take() {
                    urls.push(UrlInfo {
                        url,
                        source_file: source_file.to_path_buf(),
                        link_text,
                    });
                }
            }
            _ => {}
        }
    }

    urls
}

fn process_urls(
    url_infos: Vec<UrlInfo>,
    args: &Args,
) -> Result<Vec<UrlInfo>, Box<dyn std::error::Error>> {
    let mut unique_urls = HashMap::new();
    let base_url = Url::parse("http://base.example.com/")?;

    for info in url_infos {
        // Skip pure fragment/anchor links
        if info.url.starts_with('#') {
            continue;
        }

        // Try to parse the URL to validate and check against filters
        let url_str = &info.url;
        let parsed_url = Url::parse(url_str).or_else(|_| base_url.join(url_str));

        // Skip malformed URLs with a warning
        if parsed_url.is_err() {
            eprintln!(
                "Warning: Skipping malformed URL: {} from file {:?}",
                url_str, info.source_file
            );
            continue;
        }

        let parsed_url = parsed_url.unwrap();

        // Apply domain filter if specified
        if let Some(ref domain) = args.filter_domain {
            if let Some(host) = parsed_url.host_str() {
                if !host.contains(domain) {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Apply protocol filter if specified
        if !args.filter_protocol.is_empty() {
            if !args
                .filter_protocol
                .contains(&parsed_url.scheme().to_string())
            {
                continue;
            }
        }

        // Keep first occurrence of each URL (deduplicate)
        if !unique_urls.contains_key(url_str) {
            unique_urls.insert(url_str.clone(), info);
        }
    }

    Ok(unique_urls.into_values().collect())
}

fn output_results(url_infos: Vec<UrlInfo>, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    match args.output_format {
        OutputFormat::Stdout => {
            for info in &url_infos {
                println!("{}", info.url);
            }
        }
        OutputFormat::Text => {
            let output_path = args.output_path.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Output path required for text format",
                )
            })?;

            let mut file = File::create(output_path)?;
            for info in &url_infos {
                writeln!(file, "{}", info.url)?;
            }
        }
        OutputFormat::Csv => {
            let output_path = args.output_path.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Output path required for CSV format",
                )
            })?;

            let mut wtr = csv::Writer::from_path(output_path)?;

            wtr.write_record(&["URL", "Source File", "Link Text"])?;

            for info in &url_infos {
                wtr.write_record(&[
                    &info.url,
                    &info.source_file.to_string_lossy().to_string(),
                    &info.link_text,
                ])?;
            }

            wtr.flush()?;
        }
        OutputFormat::Html => {
            let output_path = args.output_path.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Output path required for HTML format",
                )
            })?;

            let mut file = File::create(output_path)?;

            writeln!(file, "<!DOCTYPE NETSCAPE-Bookmark-file-1>")?;
            writeln!(file, "<!-- This is an automatically generated file. -->")?;
            writeln!(
                file,
                "<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">"
            )?;
            writeln!(file, "<TITLE>Bookmarks</TITLE>")?;
            writeln!(file, "<H1>Bookmarks</H1>")?;
            writeln!(file, "<DL><p>")?;

            for info in &url_infos {
                let title = if info.link_text.is_empty() {
                    &info.url
                } else {
                    &info.link_text
                };
                writeln!(file, "    <DT><A HREF=\"{}\">{}</A>", info.url, title)?;
                writeln!(
                    file,
                    "    <DD>Source: {}",
                    info.source_file.to_string_lossy()
                )?;
            }

            writeln!(file, "</DL><p>")?;
        }
    }

    Ok(())
}
