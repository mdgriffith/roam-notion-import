use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use walkdir::WalkDir;
use std::path::PathBuf;
use regex::Regex;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor, Tree};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;
extern "C" { fn tree_sitter_markdown() -> Language; }

mod edit;

#[derive(Debug, Clone)]
struct Page {
    notion_path: PathBuf,
    notion_link: String,
    title:String,
    shorthand:String,
    tags: String,
    markdown: bool,
    final_name: String
}



/*
Notion exports both markdown and csv files.

For now we only care about the markdown files.
 
Steps to the algorithim.

    1. Traverse all files and gather all paths.
    2. Calculate new name for everything.
    3. Detect collisions in new names and recalculate if necessary.
    4. Parse each markdown file and replace
        - [Inspiration](Elm%20UI%20a717ad4bbdc741e78ae4bbe5da9bb262/Inspiration%20b28cb62e807047fdaf49ff0fdad461d3.md)
        with
        - [Inspiration]([[Elm UI, Inspiration]])
        or 
        - [[Elm UI, Inspiration]]
    5. report files that were skipped like pdf and csvs        

*/
fn main() -> io::Result<()> {
    println!("Let's get started!");
    let mut pages: Vec<Page> = vec![];

    let ids = Regex::new(r"\s[a-z0-9]{32}").unwrap();
    let path = Path::new("./data/NotionExport/Export-238ccc39-c4cb-4b7a-b559-7ff9d6481302/");

    for entry_result in WalkDir::new(path) {
        match entry_result {
            Ok(entry) => {
                if !entry.metadata()?.is_dir() {
                 
                    let str_path = entry.path()
                                            .to_str()
                                            .unwrap()
                                            .trim_start_matches(path.to_str().unwrap());

                    let title = ids.replace_all(str_path, "")
                                           .into_owned();

                    let shorthand: String = Path::new(&title)
                                                .file_stem()
                                                .unwrap()
                                                .to_str()
                                                .unwrap()
                                                .to_string();


                    let mut tags: Vec<String> = title.split("/")
                                    .map(|item| [  item  ].join(""))
                                    .collect::<Vec<String>>();
                                    
                    tags.reverse();

                    let category: String =
                        tags.get(1)
                        .unwrap_or(&String::from(""))
                        .to_string();


                    pages.push(Page { notion_path: entry.path().to_path_buf()
                                    , notion_link: str_path.to_string().replace(" ", "%20") // entry.path().to_str().unwrap().to_string()
                                    , title: [category.clone(), shorthand.clone()].join(", ")
                                    , shorthand: shorthand.clone()
                                    , tags: category  
                                    , markdown: title.ends_with(".md")
                                    , final_name: shorthand
                                    } );
                }
            }

            Err(_err) => {


            }
        }        
    };


    let all_pages = pages.to_vec();
    /*
    After gathering all the page paths and titles,

        1. If there is a title collision, prepend the parent folder to that title

    */
    for page in &mut pages {
        
        for other in &all_pages {
            if page.final_name == other.final_name && page.notion_path != other.notion_path {

                // We're just sorta assuming that if there's a top level collision, 
                // bringing in the name from one level above will make thigns cool.
                // Probably not a good general solution...
                page.final_name = page.title.clone();
                break;
            }
        }

        for other in &all_pages {
            if page.final_name == other.final_name && page.notion_path != other.notion_path {
                println!("    --> Gak, {:#?}", page);
                break;
            }
        }
        println!("{:#?}", page);
    }

    let mut names: HashMap<String, String> = HashMap::new();
    for page in &pages {
        names.insert(page.notion_link.clone(), page.final_name.clone());
    }

    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_markdown() };
    parser.set_language(language).unwrap();


    for page in &mut pages {

        // Read the markdown file
        let file = File::open(Path::new(&page.notion_path))?;
        let mut buf_reader = BufReader::new(file);
        let mut source = String::new();
        buf_reader.read_to_string(&mut source)?;

        let read = source.clone();
        

        let query = Query::new(language, "(link (link_text) @text (link_destination) @dest) @link").unwrap();

      

        edit::edit(&mut parser, &mut source, read.clone(), query, |found| {
            // println!("calculating slowly...");
            // thread::sleep(Duration::from_secs(2));
            // num

            let text = found.captures[0].node.utf8_text(read.as_bytes()).ok();

            let target = found.captures[2].node.utf8_text(read.as_bytes()).ok();
               
            // println!("Captured -> {:?}", found.captures[1].node.to_sexp());
            // println!("         -> {:?}", text);
            // println!(" rename to: {:?} ", names.get(target));

            match target.and_then(|x| names.get(x)) {
                None => None,
                Some(new_name) => {
                    Some((&found.captures[0].node, ["[[", new_name , "]]"].join("")))
                }

            }
        });


        // output new file to new directory
        println!("{:?}", source);

        break;
    }

    print!("fin");

   
    Ok(())
}






