#[macro_use]
extern crate nom;
#[macro_use]
extern crate im;

use im::{OrdMap, OrdSet, Vector};
use nom::{is_alphabetic, is_alphanumeric, is_space, line_ending};
use std::str;

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
    depth: usize,
    keyword: Option<String>,
    priority: Option<char>,
    title: Option<String>,
    tags: OrdSet<String>,
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

fn maybe_get_single_char(candidate: Option<&[u8]>) -> Option<char> {
    match candidate {
        None => None,
        Some(x) => {
            if x.len() == 1 {
                Some(x[0] as char)
            } else {
                None
            }
        }
    }
}

named!(headline_depth<&[u8], usize>, fold_many1!(tag!("*"), 0, |depth, _| depth + 1));
named!(
    keyword<&str>,
    map_res!(alt!(tag!("TODO") | tag!("DONE")), str::from_utf8)
);
named!(
    priority,
    delimited!(tag!("[#"), take_while_m_n!(1, 1, is_alphabetic), tag!("]"))
);
named!(eol, call!(line_ending));
named!(
    tag_list<Vec<&[u8]>>,
    delimited!(
        tag!(":"),
        separated_list_complete!(char!(':'), take_while!(is_valid_tag_char)),
        tag!(":")
    )
);

fn is_not_eol_or_tag_delimiter(c: u8) -> bool {
    c != b'\n' && c != b':'
}

fn to_string_vec(x: Vec<&[u8]>) -> Vec<String> {
    x.iter()
        .map(|y| String::from_utf8(y.to_vec()).unwrap())
        .collect()
}

named!(
    title<&str>,
    map_res!(take_while1!(is_not_eol_or_tag_delimiter), str::from_utf8)
);

named!(
    node<OrgNode>,
    do_parse!(
        depth: headline_depth
            >> take_while1!(is_space)
            >> keyword: opt!(keyword)
            >> take_while1!(call!(is_space))
            >> priority: opt!(priority)
            >> take_while1!(call!(is_space))
            >> title: opt!(title)
            >> tags: opt!(tag_list)
            >> (OrgNode {
                depth: depth,
                keyword: keyword.map(String::from),
                priority: maybe_get_single_char(priority),
                title: title.map(String::from),
                tags: match tags {
                    None => OrdSet::new(),
                    Some(a) => OrdSet::from(to_string_vec(a)),
                },
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
    fn get_headline_depth() {
        assert_eq!(
            headline_depth(b"***** TODO [#A] Heading"),
            Ok((&b" TODO [#A] Heading"[..], 5))
        )
    }

    #[test]
    fn get_keyword() {
        assert_eq!(
            keyword(b"TODO [#A] Heading"),
            Ok((&b" [#A] Heading"[..], "TODO"))
        )
    }

    #[test]
    fn get_priority() {
        assert_eq!(priority(b"[#A] Heading"), Ok((&b" Heading"[..], &b"A"[..])))
    }

    #[test]
    fn get_tag_list() {
        assert_eq!(
            tag_list(b":one:TWO:3hree:four:"),
            Ok((
                &[][..],
                vec![&b"one"[..], &b"TWO"[..], &b"3hree"[..], &b"four"[..]]
            ))
        );
    }

    #[test]
    fn get_node() {
        assert_eq!(
            node(b"*** TODO     [#A]   Some headline title :one:TWO:"),
            Ok((
                &[][..],
                OrgNode {
                    depth: 3,
                    keyword: Some(format!("TODO")),
                    priority: Some('A'),
                    title: Some(format!("Some headline title ")),
                    tags: ordset![format!("one"), format!("TWO")],
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
