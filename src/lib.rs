#[macro_use]
extern crate nom;
extern crate im;

use nom::{is_alphabetic};
use im::{OrdMap, OrdSet, Vector};

#[allow(dead_code)]
pub struct OrgContext {
    keywords: OrdSet<String>,
    inlinetask_min_level: usize,
}

#[allow(dead_code)]
enum OrgElement {
    Block,
    Drawer,
    PlainList,
    Footnote,
    Table,
    InlineTask,
}

#[allow(dead_code)]
pub struct OrgSection {
    contents: Vector<OrgElement>,
}

#[allow(dead_code)]
pub struct OrgNode {
    depth: u8,
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

named!(headline_depth<&[u8], usize>, fold_many1!(tag!("*"), 0, |depth, _| depth + 1));
named!(keyword, alt!(tag!("TODO") | tag!("DONE")));
named!(priority, delimited!(tag!("[#"), take_while_m_n!(1, 1, is_alphabetic), tag!("]")));
named!(tags, delimited!(tag!(":"), take_until!(":"), tag!(":\n")));
named!(tag_list<&[u8], Vec<&[u8]>>, separated_list!(tag!(":"), is_not!(":")));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_headline_depth() {
        assert_eq!(headline_depth(b"***** TODO [#A] Heading"), Ok((&b" TODO [#A] Heading"[..], 5)))
    }

    #[test]
    fn get_keyword() {
        assert_eq!(keyword(b"TODO [#A] Heading"), Ok((&b" [#A] Heading"[..], &b"TODO"[..])))
    }

    #[test]
    fn get_priority() {
        assert_eq!(priority(b"[#A] Heading"), Ok((&b" Heading"[..], &b"A"[..])))
    }

    #[test]
    fn get_tag_list() {
        assert_eq!(tag_list(b"one:TWO:3hree"), Ok((&[][..], vec![&b"one"[..], &b"TWO"[..], &b"3hree"[..]])));
    }
}
