
use pest_derive::Parser;

use pest::{
    iterators::Pairs,
    pratt_parser::{Assoc::*, Op, PrattParser},
    Parser,
};
use std::io::{stdin, stdout, Write};

#[derive(Parser)]
#[grammar = "teal.pest"]
pub struct TealParser;

impl TealParser {
    fn parse(pairs: Pairs<Rule>, pratt: &PrattParser<Rule>) { todo!() }
}


