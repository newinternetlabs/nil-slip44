use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use itertools::Itertools;
use reqwest;

const SLIP_0044_MARKDOWN_URL: &str =
    "https://raw.githubusercontent.com/satoshilabs/slips/master/slip-0044.md";
const SLIP_044_MARKDOWN_HEADER: &str =
    "| Coin type  | Path component (`coin_type'`) | Symbol  | Coin                              |";

#[derive(Debug)]
struct CoinType {
    id: u32,
    ids: Vec<u32>,
    path_component: String,
    symbol: Option<String>,
    name: String,
    original_name: String,
    rustdoc_lines: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching SLIP-0044 markdown from GitHub...");
    let markdown_content = reqwest::blocking::get(SLIP_0044_MARKDOWN_URL)?.text()?;
    println!(
        "Successfully fetched {} bytes of markdown",
        markdown_content.len()
    );

    println!("Processing markdown content...");
    let coin_types = markdown_content
        .split("\n")
        .skip_while(|&line| {
            let skip = line != SLIP_044_MARKDOWN_HEADER;
            if !skip {
                println!("Found header line, starting processing...");
            }
            skip
        })
        .skip(2)
        .filter_map(|line| {
            let columns: Vec<_> = line.split('|').collect();
            if columns.len() != 6 {
                println!(
                    "Warning: Skipping line due to incorrect number of columns: {}",
                    line
                );
                return None;
            }

            let original_name = columns[4].trim();
            if original_name.is_empty() || original_name == "reserved" {
                println!(
                    "Warning: Skipping coin due to empty or reserved name: {}",
                    original_name
                );
                return None;
            }

            let name = match original_name_to_short(original_name) {
                Ok(n) => n,
                Err(e) => {
                    println!("Warning: Skipping coin due to name error: {}", e);
                    return None;
                }
            };

            let id = match columns[1].trim().parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    println!("Warning: Skipping coin due to invalid ID: {}", columns[1]);
                    return None;
                }
            };

            println!("Processing coin: {} (ID: {})", original_name, id);

            Some(CoinType {
                id,
                ids: vec![],
                path_component: columns[2].trim().to_string(),
                symbol: Some(columns[3].trim())
                    .map(prepend_enum)
                    .map(|symbol| match symbol.as_str() {
                        "$DAG" => "DAG".to_string(),
                        symbol => symbol.to_string(),
                    })
                    .filter(|symbol| !symbol.is_empty()),
                name: name.to_string(),
                original_name: original_name.to_string(),
                rustdoc_lines: vec![],
            })
        });

    println!("Building coin type map...");
    let coin_types = coin_types.fold(HashMap::<_, CoinType>::new(), |mut acc, coin_type| {
        let id = coin_type.id.clone();
        acc.entry((
            coin_type.symbol.clone(),
            coin_type.name.clone(),
            coin_type.original_name.clone(),
        ))
        .or_insert(coin_type)
        .ids
        .push(id);
        acc
    });

    println!("Processing {} unique coins...", coin_types.len());

    let coin_types = coin_types
        .into_iter()
        .fold(HashMap::<_, Vec<_>>::new(), |mut acc, (_, coin_type)| {
            acc.entry(coin_type.name.clone())
                .or_default()
                .push(coin_type);
            acc
        })
        .into_iter()
        .map(|(_, coin_types)| {
            let coin_types = if coin_types.len() > 1 {
                println!("Found duplicate coins for name: {}", coin_types[0].name);
                coin_types
                    .into_iter()
                    .map(|coin_type| CoinType {
                        name: format!(
                            "{}_{}",
                            coin_type.name.clone(),
                            match coin_type.symbol.clone() {
                                Some(symbol) => symbol,
                                None => coin_type.ids.clone().into_iter().join("_").to_string(),
                            }
                        ),
                        ..coin_type
                    })
                    .collect()
            } else {
                coin_types
            };

            coin_types
                .into_iter()
                .map(|coin_type| CoinType {
                    rustdoc_lines: vec![
                        format!("/// Coin type: {}", coin_type.ids.iter().join(", ")),
                        if let Some(symbol) = coin_type.symbol.clone() {
                            format!("/// Symbol: {}", symbol)
                        } else {
                            "".to_string()
                        },
                        format!("/// Coin: {}", coin_type.original_name),
                    ],
                    ..coin_type
                })
                .collect::<Vec<_>>()
        })
        .flatten();

    println!("Creating output file...");
    let output_path = Path::new(file!())
        .parent()
        .ok_or("can't get first parent")?
        .parent()
        .ok_or("can't get second parent")?
        .join("coin.rs");
    println!("Writing to: {}", output_path.display());

    let mut file = std::fs::File::create(&output_path)?;

    writeln!(&mut file, "// Code generated by {}; DO NOT EDIT.", file!())?;
    writeln!(&mut file, "use crate::coins;")?;
    writeln!(&mut file, "coins!(")?;

    let mut seen_symbols = HashSet::<String>::new();
    let mut coin_count = 0;

    for coin_type in coin_types.sorted_by_key(|coin_type| coin_type.id) {
        coin_count += 1;

        // Pre-compute escaped symbol if it exists
        let escaped_symbol = coin_type.symbol.as_ref().map(|s| escape_rust_string(s));

        writeln!(
            &mut file,
            "    (\n        {}\n        [{}], {}, \"{}\", {}, {},\n    ),",
            coin_type
                .rustdoc_lines
                .into_iter()
                .filter(|s| !s.is_empty())
                .join("\n        "),
            coin_type.ids.into_iter().join(",").to_string(),
            coin_type.name,
            escape_rust_string(&coin_type.original_name),
            match &escaped_symbol {
                Some(symbol) => {
                    if seen_symbols.contains(symbol) {
                        ""
                    } else {
                        symbol
                    }
                }
                None => "",
            },
            match &escaped_symbol {
                Some(symbol) => {
                    if seen_symbols.contains(symbol) {
                        format!("\"{}\"", symbol)
                    } else {
                        seen_symbols.insert(symbol.clone());
                        "".to_string()
                    }
                }
                None => "".to_string(),
            },
        )?;
    }
    writeln!(&mut file, ");")?;

    println!(
        "Successfully wrote {} coins to {}",
        coin_count,
        output_path.display()
    );
    println!("Done!");

    Ok(())
}

fn parse_markdown_link(input: &str) -> (&str, Option<&str>) {
    if input.starts_with('[') {
        (
            input.splitn(3, &['[', ']'][..]).nth(1).unwrap_or(input),
            input
                .trim_start_matches(']')
                .splitn(3, &['(', ')'][..])
                .nth(1),
        )
    } else {
        (input, None)
    }
}

fn original_name_to_short(original_name: &str) -> Result<String, String> {
    let mut name = original_name.replace(' ', "");
    name = name
        .split_once('(')
        .map_or(name.to_string(), |(name, _)| name.to_string());
    name = prepend_enum(&name);

    // Check direct mappings first
    let name_match = match name.as_str() {
        "Ether" => Ok("Ethereum"),
        "EtherClassic" => Ok("EthereumClassic"),
        name => Ok(name), // Default to original name if no mapping
    };

    // Then handle special characters if needed
    if name.contains(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        let special_match = match name.as_str() {
            "Pl^g" => Ok("Plug"),
            "BitcoinMatteo'sVision" => Ok("BitcoinMatteosVision"),
            "Crypto.orgChain" => Ok("CryptoOrgChain"),
            "Cocos-BCX" => Ok("CocosBCX"),
            "Capricoin+" => Ok("CapricoinPlus"),
            "Seele-N" => Ok("SeeleN"),
            "IQ-Cash" => Ok("IQCash"),
            "XinFin.Network" => Ok("XinFinNetwork"),
            "Unit-e" => Ok("UnitE"),
            "HARMONY-ONE" => Ok("HarmonyOne"),
            "ThePower.io" => Ok("ThePower"),
            "evan.network" => Ok("EvanNetwork"),
            "Ether-1" => Ok("EtherOne"),
            "æternity" => Ok("aeternity"),
            "θ" => Ok("Theta"),
            name => name_match.and_then(|_| Err(format!("unknown original coin name `{}`", name))),
        };
        special_match.map(|name| name.to_string())
    } else {
        name_match.map(|name| name.to_string())
    }
}

fn prepend_enum(name: &str) -> String {
    if name.starts_with(char::is_numeric) {
        ["_", name].join("")
    } else {
        name.to_string()
    }
}

fn escape_rust_string(s: &str) -> String {
    s.replace('@', "") // Remove @ symbols
        .replace('^', "") // Remove ^ symbols
        .replace('\'', "") // Remove single quotes
        .replace('"', "") // Remove double quotes
        .replace('\\', "") // Remove backslashes
        .replace('$', "") // Remove dollar signs
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric()
                || *c == '_'
                || *c == ' '
                || *c == '-'
                || *c == '+'
                || *c == '.'
                || *c == '('
                || *c == ')'
        }) // Only allow these characters
        .collect()
}
