use rayon::prelude::*;
use rsml::tfidf_helper::*;
use tfidf::{TfIdf, TfIdfDefault};
use std::collections::BTreeMap;

pub fn convert_is_self(b: bool) -> f64 {
    if b {
        0f64
    } else {
        1f64
    }
}

pub fn convert_author_to_popularity<T: AsRef<str>>(authors: &[T],
                                                   rust_authors: &[&str])
                                                   -> Vec<f64> {
    let mut auth_count: BTreeMap<&str, _> = BTreeMap::new();

    for author in authors {
        *auth_count.entry(author.as_ref()).or_insert(0) += 1;
    }
    let mut freqs = Vec::with_capacity(authors.len());

    for author in rust_authors {
        let author_freq = *auth_count.get(author.as_ref()).unwrap_or(&0);
        freqs.push(author_freq as f64);
    }
    freqs
}

pub fn text_to_docs<T: AsRef<str>>(texts: &[&str]) -> Vec<Vec<(String, usize)>> {
    let mut docs = Vec::with_capacity(texts.len());
    texts.par_iter()
         .map(|s| str_to_doc(s))
         .collect_into(&mut docs);
    docs
}

// TODO: This should probably return an ndarray
pub fn tfidf_reduce_selftext(self_texts: &[&str],
                             words: &[&str],
                             docs: &[Vec<(String, usize)>])
                             -> Vec<Vec<f64>> {

    let docs: Vec<Vec<_>> = docs.iter()
                                .map(|doc| doc.iter().map(|t| (t.0.as_str(), t.1)).collect())
                                .collect();
    let all_docs = docs.clone();

    let mut term_frequency_matrix = Vec::with_capacity(self_texts.len());
    println!("TFIDF over {:?} words and {} docs", words.len(), docs.len());

    for doc in docs {
        let mut term_frequencies: Vec<f64> = Vec::with_capacity(words.len());

        words.par_iter()
             .weight_max()
             .map(|word| TfIdfDefault::tfidf(word, &doc, all_docs.iter()))
             .collect_into(&mut term_frequencies);

        term_frequency_matrix.push(term_frequencies);
    }

    term_frequency_matrix
}

pub fn symbol_counts(self_texts: &[&str]) -> Vec<Vec<f64>> {
    let symbols = ['{', '}', '(', ')', '<', '>', ';', '.', ',', '&', '[', ']', ':', '?', '*', '=',
                   '!', '/', '\\', '$', '-', '+', '|', '`', '_', '~', '%'];

    let mut freq_matrix = Vec::with_capacity(self_texts.len());

    for text in self_texts {
        let mut char_map = BTreeMap::new();

        for ch in text.chars() {
            *char_map.entry(ch).or_insert(0) += 1;
        }

        let mut freq_vec = Vec::with_capacity(symbols.len());

        for symbol in symbols.iter() {
            let symbol_count = *char_map.get(symbol).unwrap_or(&0);
            freq_vec.push(symbol_count as f64);
        }

        freq_matrix.push(freq_vec);
    }
    freq_matrix
}

pub fn interesting_word_freq(self_texts: &[&str], spec_words: &[String]) -> Vec<Vec<f64>> {

    let mut freq_matrix = Vec::with_capacity(self_texts.len());
    let text_words: Vec<Vec<String>> = self_texts.iter()
                                                 .map(|t| tfidf_helper::get_words(*t))
                                                 .collect();

    for words in text_words.iter() {
        let mut freq_map: BTreeMap<String, u64> = BTreeMap::new();

        for word in words {
            *freq_map.entry(word.to_owned()).or_insert(0) += 1;
        }

        let mut freq_vec: Vec<_> = Vec::with_capacity(spec_words.len());

        for word in spec_words.iter() {
            let word_count = *freq_map.get(word).unwrap_or(&0);
            freq_vec.push(word_count as f64);
        }

        freq_matrix.push(freq_vec);
    }

    freq_matrix
}

pub fn subs_to_float(subs: &[&str]) -> Vec<f64> {
    let mut sub_float_map = BTreeMap::new();
    let mut sub_floats = Vec::with_capacity(subs.len());
    let mut cur_sub = 0;

    for sub in subs {
        let f = *sub_float_map.entry(sub).or_insert_with(|| {
            let c = cur_sub;
            cur_sub = c + 1;
            c
        });
        sub_floats.push(f);
    }
    let sub_floats = sub_floats.into_iter()
                               .map(|f| f as f64)
                               .collect();
    // println!("{:?}", sub_floats);
    sub_floats
}
