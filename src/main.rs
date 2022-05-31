use generic_parser::GenericParser;
use std::fs::read_to_string;
use parser::*;


mod parser;
mod render;


fn main() {
    let filename="example.docbuild";
    let contents=read_to_string(filename).unwrap();
    let items_res=GenericParser::new(&contents,filename).into_document();
    match items_res {
        Ok(items)=>println!("Items: {:#?}",items),
        Err(e)=>e.print_with_context(&contents,false),
    }
}
