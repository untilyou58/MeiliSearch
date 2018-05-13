// TODO make the raptor binary expose multiple subcommand
//      make only one binary

extern crate raptor;
extern crate serde_json;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::iter;

use raptor::{MapBuilder, Map, Value, AttrIndex};
use serde_json::from_str;

fn main() {
    let data = File::open("products.json_lines").unwrap();
    let data = BufReader::new(data);

    let common_words = {
        match File::open("fr.stopwords.txt") {
            Ok(file) => {
                let file = BufReader::new(file);
                let mut set = HashSet::new();
                for line in file.lines().filter_map(|l| l.ok()) {
                    for word in line.split_whitespace() {
                        set.insert(word.to_owned());
                    }
                }
                set
            },
            Err(e) => {
                eprintln!("{:?}", e);
                HashSet::new()
            },
        }
    };

    let mut builder = MapBuilder::new();
    for line in data.lines() {
        let line = line.unwrap();

        let product: serde_json::Value = from_str(&line).unwrap();

        // TODO use a real tokenizer
        let title = iter::repeat(0).zip(product["title"].as_str().expect("invalid `title`").split_whitespace())
                                    .filter(|(_, s)| !common_words.contains(*s))
                                    .enumerate();
        let description = iter::repeat(1).zip(product["ft"].as_str().expect("invalid `ft`").split_whitespace())
                                    .filter(|(_, s)| !common_words.contains(*s))
                                    .enumerate();

        let words = title.chain(description);
        for (i, (attr, word)) in words {
            let value = Value {
                id: product["product_id"].as_u64().expect("invalid `product_id`"),
                attr_index: AttrIndex {
                    attribute: attr,
                    index: i as u64,
                },
            };
            builder.insert(word, value);
        }
    }

    let map = File::create("map.fst").unwrap();
    let values = File::create("values.vecs").unwrap();
    let (map, values) = builder.build(map, values).unwrap();

    eprintln!("Checking the dump consistency...");
    unsafe { Map::<Value>::from_paths("map.fst", "values.vecs").unwrap() };
}
