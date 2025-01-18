use std::{
    fs::{self, File},
    io::Write,
    process::Command,
};

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor};

use crate::{BuildStep, BuildStepError};

pub struct Treesitter {}

impl BuildStep for Treesitter {
    fn run(dir: &crate::WorkingDirectory) -> Result<(), BuildStepError> {
        let c_path = format!("{}/out.c", dir.out);

        let mut code = fs::read_to_string(&c_path)?;
        let code_bytes = code.clone();
        let code_bytes = code_bytes.as_bytes();
        parse_declarator_attributes(&mut code, code_bytes);

        let code_bytes = code.clone();
        let code_bytes = code_bytes.as_bytes();
        parse_call_expression_attributes(&mut code, code_bytes);

        let mut file = File::create(&c_path)?;
        file.write_all(&code.as_bytes())?;

        //Remove All Global Variable Declarations (Because it is mostly duplicated with Global Variable Definitions)
        //TODO: It can cause problems, so it needs to be replaced with a more sophisticated logic
        if let Ok(status) = Command::new("sed")
            .args([
                "/.*Global Variable Declarations.*/,/.*Function Declarations.*/{/^\\//!d;}",
                "-i",
                c_path.as_str(),
            ])
            .status()
        {
            if status.success() {
                return Ok(());
            } else {
                return Err(BuildStepError::ChildProcessFailed(
                    "sed".to_string(),
                    status,
                ));
            }
        } else {
            Err(BuildStepError::ChildExecutionFailed("sed".to_string()))
        }
    }
}

fn parse_declarator_attributes(code: &mut String, code_bytes: &[u8]) {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .unwrap();

    let tree = parser.parse(code_bytes, None).unwrap();
    let query = Query::new(
        &tree_sitter_c::LANGUAGE.into(),
        "(function_declarator(identifier)@a)@b",
    )
    .unwrap();
    let mut cursor = QueryCursor::new();

    let mut diff: i32 = 0;

    let mut matches = cursor.matches(&query, tree.root_node(), code_bytes);
    while let Some(qm) = matches.next() {
        let declarator_node = qm.captures[0].node;
        let identifier_node = qm.captures[1].node;
        let declarator = declarator_node.utf8_text(&code_bytes).unwrap();
        let identifier = identifier_node.utf8_text(&code_bytes).unwrap();
        let attributes: Vec<_> = identifier.split("_AC___").collect(); //` __`

        if attributes.len() < 2 {
            continue;
        }

        match &attributes[..] {
            [] | [_] => {}
            [_, attrib @ ..] => {
                let attrib = format!("__{}", attrib.join(" __"));
                let attrib = attrib
                    .replace("_AC_", " ")
                    .replace("_IC_", "(")
                    .replace("_JC_", ")")
                    .replace("_MC_", ",");

                let start = declarator.find("_AC_").unwrap();
                let end = declarator.find("(").unwrap();
                let mut declarator = String::from(declarator);
                declarator.replace_range(start..end, "");

                code.replace_range(
                    ((declarator_node.start_byte() as i32) + diff) as usize
                        ..((declarator_node.end_byte() as i32) + diff) as usize,
                    &format!("{} {}", declarator, attrib),
                );
                diff += (declarator.len() + attrib.len() + 1) as i32
                    - declarator_node.byte_range().len() as i32;
            }
        }
    }
}

fn parse_call_expression_attributes(code: &mut String, code_bytes: &[u8]) {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .unwrap();

    let tree = parser.parse(code_bytes, None).unwrap();
    let query = Query::new(
        &tree_sitter_c::LANGUAGE.into(),
        "(call_expression(identifier)@a)",
    )
    .unwrap();
    let mut cursor = QueryCursor::new();

    let mut diff: i32 = 0;

    let mut matches = cursor.matches(&query, tree.root_node(), code_bytes);
    while let Some(qm) = matches.next() {
        let identifier_node = qm.captures[0].node;
        let identifier = identifier_node.utf8_text(&code_bytes).unwrap();

        let start = identifier.find("_AC_");
        let start = match start {
            None => continue,
            Some(start) => start,
        };

        let mut identifier = String::from(identifier);
        identifier.replace_range(start.., "");

        code.replace_range(
            ((identifier_node.start_byte() as i32) + diff) as usize
                ..((identifier_node.end_byte() as i32) + diff) as usize,
            &identifier,
        );
        diff += identifier.len() as i32 - identifier_node.byte_range().len() as i32;
    }
}
