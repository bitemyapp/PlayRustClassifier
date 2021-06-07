#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use(time)]
extern crate playrust_alert;

#[macro_use(stack)]
extern crate ndarray;

extern crate clap;
extern crate csv;
extern crate dedup_by;
extern crate rand;
extern crate rayon;
extern crate rsml;
extern crate rustc_serialize;
extern crate serde_json;
extern crate stopwatch;
extern crate tfidf;

use clap::{App, Arg};
use dedup_by::dedup_by;
use playrust_alert::reddit::RawPostFeatures;
use rsml::tfidf_helper::get_unique_word_list;

use std::collections::BTreeMap;

fn get_train_data() -> Vec<RawPostFeatures> {
    let matches = App::new("Model Generator")
        .version("1.0")
        .about("Generates a random forest based on a training set")
        .arg(
            Arg::with_name("train")
                .help("The CSV to train on")
                .required(true)
                .index(1),
        )
        .get_matches();

    let train_path = matches.value_of("train").unwrap();

    let mut rdr = csv::Reader::from_file(train_path).unwrap();

    let mut posts: Vec<RawPostFeatures> = rdr.decode().map(|raw_post| raw_post.unwrap()).collect();

    posts.sort_by(|a, b| a.title.cmp(&b.title));
    dedup_by(&mut posts, |a, b| a.title == b.title);
    posts
}

fn word_freqs(posts: &[RawPostFeatures]) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();

    for post in posts {
        let post = vec![post.selftext.as_str()];

        for word in get_unique_word_list(&post[..]) {
            *map.entry(word).or_insert(0) += 1;
        }
    }
    map
}

fn main() {
    let posts = get_train_data();
    let (rust, play): (Vec<RawPostFeatures>, Vec<RawPostFeatures>) =
        posts.into_iter().partition(|post| post.subreddit == "rust");

    let mut rust_word_freq: Vec<(String, u64)> = word_freqs(&rust[..]).into_iter().collect();
    rust_word_freq.sort_by(|a, b| a.1.cmp(&b.1));

    // let mut play_word_freq: Vec<(String, u64)> = word_freqs(&play[..]).into_iter().collect();
    // play_word_freq.sort_by(|a, b| a.1.cmp(&b.1));

    println!("{:#?}", rust_word_freq);
}
