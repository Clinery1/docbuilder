use generic_parser::GenericParser;
use std::{
    fs::{
        read_to_string,
        write as write_file,
    },
    path::Path,
    env::args,
};
use parser::*;
use render::*;


mod parser;
mod render;


fn main() {
    let mut args=args().collect::<Vec<_>>();
    let exe_name=args.remove(0);
    if args.contains(&"--help".to_string())||args.len()==0 {
        help(&exe_name);
        return;
    }
    for file in args {
        let path=Path::new(&file);
        let name=path.file_stem().unwrap().to_str().unwrap();
        let contents=read_to_string(&file).unwrap();
        let doc_res=GenericParser::new(&contents,&file).into_document();
        match doc_res {
            Ok(doc)=>{
                let html=doc.into_html(ParentDirection::None);
                write_file(format!("{}.html",name),html).unwrap();
            },
            Err(e)=>e.print_with_context(&contents,false),
        }
    }
}
fn help(exe_name:&str) {
    println!("Help:");
    println!("    {} FILE1 FILE2 ...",exe_name);
}
