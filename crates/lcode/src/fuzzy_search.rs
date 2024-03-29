use std::fmt::Display;

use atoi::atoi;
use inquire::Select;
use lcode_config::config::global::G_USER_CONFIG;
use leetcode_api::dao;
use miette::Result;

// use rayon::prelude::*;

pub async fn select_a_question() -> Result<u32> {
    let questions = dao::query_all_index().await?;

    let indexs = questions
        .into_iter()
        .map(|v| v.to_string())
        .collect();

    let selected = Select::new("Select question ❓:", indexs)
        .with_formatter(&|v| format!("{:.10}", v.to_string()))
        .with_page_size(G_USER_CONFIG.config.page_size)
        .prompt()
        .unwrap_or_default();

    let selected: Vec<&str> = selected.split('[').collect();
    let id_str = selected
        .get(1)
        .copied()
        .unwrap_or_default();

    let id = atoi::<u32>(id_str.as_bytes()).unwrap_or_default();

    Ok(id)
}

#[inline]
pub fn filter<T>(input: &str, _: &T, string_value: &str, _: usize) -> bool
where
    T: Display,
{
    use simsearch::SimSearch;
    let mut search_engine = SimSearch::new();
    search_engine.insert(string_value, string_value);
    let res = search_engine.search(input);

    res.contains(&string_value)
        || string_value
            .to_lowercase()
            .contains(&input.to_lowercase())
}

// pub fn new_filter<T>(a: Vec<T>, pat: &str) -> Vec<(T, u32)>
// where
//     T: Display + Sized + Send + Sync,
// {
//     use nucleo::{
//         pattern::{CaseMatching, Pattern},
//         Config, Matcher,
//     };
//
//     let a: Vec<String> = a
//         .into_par_iter()
//         .map(|v| v.to_string())
//         .collect();
//
//     let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
//     let matches = Pattern::parse(pat, CaseMatching::Ignore).match_list(a, &mut matcher);
//
//     let a = vec![];
//     a
// }
