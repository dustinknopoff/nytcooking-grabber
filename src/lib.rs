#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![warn(clippy::all)]
//! # Recipe Grabber
//!
//! ![build-and-check](https://github.com/dustinknopoff/nytcooking-grabber/workflows/build-and-check/badge.svg)
//!
//! Deployed to [Cloudflare](https://nytcooking-grabber.knopoff.workers.dev)
//!
//! Pass `/?url=<url>` to produce a markdown representation
//!
//! ## Currently supported sites:
//!
//! - [NYTimes Cooking](https://cooking.nytimes.com)
//! - [Food and Wine](https://foodandwine.com)
//!

mod ld_md;
mod sites;
mod utils;
use cfg_if::cfg_if;
use ld_md::RecipeMarkdownBuilder;
use scraper::{Html, Selector};
use sites::LdRecipe;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
/// Given the contents of a website, The `application/ld+json` attribute is extracted,
/// parsed, and converted in to a markdown document.
pub fn get_ld_json(contents: &str) -> String {
    let document = Html::parse_document(contents);
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
    let ctx = document.select(&selector).next().unwrap();
    let text = ctx.text().collect::<Vec<_>>();
    let as_txt = text.join("");
    let as_txt = traverse_for_type_recipe(&as_txt);
    let as_recipe: LdRecipe<'_> = serde_json::from_str(&as_txt).unwrap();
    let mut builder = RecipeMarkdownBuilder::new(&as_recipe);
    builder.build().into()
}

fn traverse_for_type_recipe(content: &str) -> String {
    let tree: serde_json::Value = serde_json::from_str(content).unwrap();
    // let test_pattern = String::from("Recipe");
    let _recipe_str = serde_json::json!("Recipe");
    // Example: tests/ragu.json
    if let Some(_recipe_str) = tree.get("@type") {
        return content.to_string();
    }
    // Example: tests/chocolate_olive_oil.json
    if let Some(val) = tree.get("@graph") {
        val.as_array()
            .unwrap()
            .iter()
            .filter(|graph_item| graph_item.get("@type") == Some(&_recipe_str))
            .collect::<Vec<_>>()
            .first()
            .unwrap()
            .to_string()
    }
    // Example: tests/full_hummus.json
    else if tree.is_array() {
        tree.as_array()
            .unwrap()
            .iter()
            .filter(|graph_item| graph_item.get("@type") == Some(&_recipe_str))
            .collect::<Vec<_>>()
            .first()
            .unwrap()
            .to_string()
    } else {
        panic!("Invalid recipe!")
    }
}

#[cfg(test)]
mod tests {

    use crate::get_ld_json;

    #[test]
    fn hummus() {
        let src = include_str!("../tests/hummus.html");
        let expected = include_str!("../tests/hummus.md");
        assert_eq!(get_ld_json(src), expected);
    }

    #[test]
    fn ragu() {
        let src = include_str!("../tests/ragu.html");
        let expected = include_str!("../tests/ragu.md");
        assert_eq!(get_ld_json(src), expected);
    }

    #[test]
    fn chocolate_olive_oil() {
        let src = include_str!("../tests/chocolate_olive_oil.html");
        let expected = include_str!("../tests/chocolate_olive_oil.md");
        assert_eq!(get_ld_json(src), expected);
    }
}
