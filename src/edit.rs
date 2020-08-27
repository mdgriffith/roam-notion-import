

use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor, Tree, QueryMatch};

extern "C" { fn tree_sitter_markdown() -> Language; }



// Specify a query and possibly make an edit to when that query matches
//
pub fn edit<Replacement>( parser: &mut Parser
                        , source: &mut String
                        , read: String
                        , query: Query
                        , replacement: Replacement 
                        ) 
                        where Replacement: for<'a> Fn(&'a QueryMatch) -> Option<(&'a Node<'a>, String)> {

    let tree = parser.parse(read.clone(), None).unwrap();
    let root = tree.root_node();

    let mut cursor = QueryCursor::new();
    let results = cursor.matches(&query, root, |node| {
        match node.utf8_text(read.as_bytes()) {
            Ok(str) => str.to_string(),
            Err(_err) => (String::from("Incorrect!")),
        }
    });


    // As we make edits, we need to keep track of if our edit modified the positions of the source code
    // First we have a byte_offset which can be negative or positive as we delete or add things
    let mut byte_offset:isize = 0;

    // however we also track row and column
    // For multiline edits the rows may have changed
    let mut row_offset: isize = 0;

    // for multiple edits on a single line, the column may have shifted.
    // the `last_row_edited` is the first row in a multi-line deletion.
    let mut last_row_edited: usize = 0;
    let mut column_offset: isize = 0;


    for found in results {
        match replacement(&found) {
            None => {}
            Some((node, new)) => {

                let new_length = new.len();
                let new_lines = new.lines().count() as isize;

                let start_byte = (node.start_byte() as isize + byte_offset) as usize;
                let old_end_byte = (node.end_byte() as isize + byte_offset) as usize;
                let new_end_byte = (node.start_byte() as isize + byte_offset) as usize + new_length;

                let edit = InputEdit {
                    // We start, accounting for offsets of previous edits
                    start_byte: start_byte,
                    old_end_byte: old_end_byte,
                    
                    new_end_byte: new_end_byte,
                    start_position: node.start_position(),      // start_position , Point
                    old_end_position: node.end_position(),      // old_end_position , Point
                    new_end_position: Point::new(
                        (node.start_position().row as isize + new_lines - 1) as usize,
                        node.start_position().column + new_length,
                    ), 
                };

               

                row_offset = row_offset + new_lines - 1;
                column_offset =  node.start_position().column as isize;
                last_row_edited = node.start_position().row;
                byte_offset = byte_offset + new_end_byte as isize - old_end_byte as isize;

                // println!("Delta {:?}", new_end_byte as isize - old_end_byte as isize);
                // println!("Full D {:?}", byte_offset);
                // println!("D row {:?}", new_lines - 1);

                // println!("{:#?}", new);
                let range = std::ops::Range 
                            { start: start_byte
                            , end: old_end_byte 
                            };

                source.replace_range(range, &new);

            }
        }   
    }
}