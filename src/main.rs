use std::collections::BTreeMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

use anyhow::{Result, Context};
use clap::{Arg, Command};
use flate2::read::GzDecoder;
use itertools::Itertools;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WordData {
    word: String,
    count: String,
    percent: String
}

// trying to read frequency-alpha-alldicts.txt.gz obtained from https://github.com/hackerb9/gwordlist
fn main() -> Result<()> {
    let matches = Command::new("Create JSON files with Google 1-gram words")
        .version("0.1")
        .author("Ab Tiwary")
        .about("A Rust app to create JSON files with Google 1-gram words")
        .arg(Arg::new("input-file"))
            .short_flag('f')
            .long_flag("input-file")
        .arg(Arg::new("output-dir"))
            .short_flag('o')
            .long_flag("output-dir")
        .get_matches();

    let input_file = matches.value_of("input-file").context("require a path to an input file")?;
    let output_dir = matches.value_of("output-dir").context("require a path to an output directory")?;
    println!("{:?}", &input_file);
    println!("{:?}", &output_dir);
    
    //let input_file = "/Users/abtiwary/temp/frequency-alpha-alldicts.txt.gz";
    //let output_dir = "/Users/abtiwary/temp/words";

    let in_file = File::open(&input_file).context("expected to read an input file")?;
    let reader = BufReader::new(GzDecoder::new(in_file));

    let path_json_four_letters = Path::new(&output_dir).join("four_letters.json");
    let path_json_five_letters = Path::new(&output_dir).join("five_letters.json");
    let path_json_eight_letters = Path::new(&output_dir).join("eight_letters.json");
    let path_json_nine_letters = Path::new(&output_dir).join("nine_letters.json");

    let mut file_json_four_letters = File::create(path_json_four_letters)
        .context("expected to be able to write an output file")?;
    let mut file_json_five_letters = File::create(path_json_five_letters)
        .context("expected to be able to write an output file")?;
    let mut file_json_eight_letters = File::create(path_json_eight_letters)
        .context("expected to be able to write an output file")?;
    let mut file_json_nine_letters = File::create(path_json_nine_letters)
        .context("expected to be able to write an output file")?;

    let mut four_letter_words: BTreeMap<String, Vec<WordData>> = BTreeMap::new();
    let mut five_letter_words: BTreeMap<String, Vec<WordData>> = BTreeMap::new();
    let mut eight_letter_words: BTreeMap<String, Vec<WordData>> = BTreeMap::new();
    let mut nine_letter_words: BTreeMap<String, Vec<WordData>> = BTreeMap::new();

    let map_updater = | m: &mut BTreeMap<String, Vec<WordData>>, k: String, v: WordData | {
        if !m.contains_key(&k) {
            m.insert(k.clone(), Vec::new());
        }
        m.get_mut(&k).unwrap().push(v);
    };

    let mut line_num: usize = 0;
    for l in reader.lines() {
        if line_num == 0 {
            line_num += 1;
            continue;
        }

        match l {
            Ok(line_str) => {
                let words: Vec<&str> = line_str.split_ascii_whitespace()
                                            .into_iter()
                                            .filter(|w| *w != " ")
                                            .collect_vec();
                
                let mut total_uppercase_chars = 0;
                words[1].chars().into_iter().for_each(|c| {
                    if c.is_uppercase() {
                        total_uppercase_chars += 1;
                    }
                });

                if total_uppercase_chars > 0 {
                    continue;
                }

                // sort the word alphabetically to form a key
                let sorted = words[1].to_lowercase().chars().sorted().collect::<String>();
                let temp_word_data = WordData{
                    word: words[1].to_string(),
                    count: words[2].to_string(),
                    percent: words[3].to_string()
                };

                match words[1].len() {
                    4 => map_updater(&mut four_letter_words, sorted.clone(), temp_word_data),
                    5 => map_updater(&mut five_letter_words, sorted.clone(), temp_word_data),
                    8 => map_updater(&mut eight_letter_words, sorted.clone(), temp_word_data),
                    9 => map_updater(&mut nine_letter_words, sorted.clone(), temp_word_data),
                    _ => continue
                }

            },
            Err(error) => {
                eprintln!("error on line {}: {:?}", line_num, error);
            }
        }

        line_num += 1;
    }

    let out_str_four = serde_json::to_string(&four_letter_words).context("expect marshaling of four letter words to JSON")?;
    file_json_four_letters.write_all(out_str_four.as_bytes());

    let out_str_five = serde_json::to_string(&five_letter_words).context("expect marshaling of five letter words to JSON")?;
    file_json_five_letters.write_all(out_str_five.as_bytes());

    let out_str_eight = serde_json::to_string(&eight_letter_words).context("expect marshaling of eight letter words to JSON")?;
    file_json_eight_letters.write_all(out_str_eight.as_bytes());

    let out_str_nine = serde_json::to_string(&nine_letter_words).context("expect marshaling of nine letter words to JSON")?;
    file_json_nine_letters.write_all(out_str_nine.as_bytes());

    println!("done!");
    Ok(())
}
