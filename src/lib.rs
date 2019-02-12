#[macro_use]
extern crate nom;
#[macro_use]
extern crate im;

use im::{OrdMap, OrdSet, Vector};
use nom::{is_alphanumeric};
use headline::{Headline, headline};

mod headline;

#[allow(dead_code)]
pub struct OrgContext {
    keywords: OrdSet<String>,
    inlinetask_min_level: usize,
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug, Clone)]
enum OrgElement {
    Block,
    Drawer,
    PlainList,
    Footnote,
    Table,
    InlineTask,
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
pub struct OrgSection {
    contents: Vector<OrgElement>,
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
pub struct OrgNode {
    headline: Headline,
    scheduled: Option<String>,
    deadline: Option<String>,
    closed: Option<String>,
    properties: OrdMap<String, String>,
    body: Option<OrgSection>,
}

fn is_valid_tag_char(candidate: u8) -> bool {
    let candidate_char = candidate as char;
    is_alphanumeric(candidate) || candidate_char == '_' || candidate_char == '@'
}

named!(
    node<OrgNode>,
    do_parse!(
        headline: headline
            >> (OrgNode {
                headline: headline,
                body: None,
                scheduled: None,
                deadline: None,
                closed: None,
                properties: ordmap! {}
            })
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_node() {
        assert_eq!(
            node(b"*** TODO     [#A]   Some headline title :one:TWO:\n"),
            Ok((
                &[][..],
                OrgNode {
                    headline: Headline {
                        depth: 3,
                        keyword: Some(format!("TODO")),
                        priority: Some('A'),
                        stats: None,
                        timestamp: None,
                        title: Some(format!("Some headline title")),
                        tags: ordset![format!("one"), format!("TWO")],
                    },
                    closed: None,
                    deadline: None,
                    scheduled: None,
                    properties: ordmap! {},
                    body: None
                }
            ))
        );
    }
}
