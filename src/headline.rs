use im::OrdSet;
use nom::{is_alphabetic, is_alphanumeric, is_digit, line_ending, multispace};
use std::str;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Headline {
    pub depth: u8,
    pub keyword: Option<String>,
    pub priority: Option<char>,
    pub title: Option<String>,
    pub timestamp: Option<String>,
    pub stats: Option<Stat>,
    pub tags: OrdSet<String>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct TitleMeta {
    start: Option<String>,
    stats: Option<Stat>,
    tags: OrdSet<String>,
    leftovers: Option<String>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Stat {
    Percentage(u8),
    Ratio(u8, u8),
}

named!(depth<u8>, fold_many1!(tag!("*"), 0, |depth, _| depth + 1));

named!(
    keyword<&str>,
    map_res!(alt!(tag!("TODO") | tag!("DONE")), str::from_utf8)
);

named!(
    priority,
    delimited!(tag!("[#"), take_while_m_n!(1, 1, is_alphabetic), tag!("]"))
);

fn is_valid_title_char(c: u8) -> bool {
    c != b'\n' && c != b':' && c != b'['
}

named!(
    title_start<&str>,
    map_res!(take_while1!(is_valid_title_char), str::from_utf8)
);

fn numeric_str_to_u8(numeric: &[u8]) -> u8 {
    str::from_utf8(numeric).unwrap().parse::<u8>().unwrap()
}

named!(
    stats_percentage<Stat>,
    map!(
        delimited!(tag!("["), take_while1!(is_digit), tag!("%]")),
        |x| Stat::Percentage(numeric_str_to_u8(x))
    )
);

named!(
    stats_ratio<Stat>,
    do_parse!(
        tag!("[")
            >> first: map!(take_while1!(is_digit), numeric_str_to_u8)
            >> tag!("/")
            >> second: map!(take_while1!(is_digit), numeric_str_to_u8)
            >> tag!("]")
            >> (Stat::Ratio(first, second))
    )
);

named!(stats<Stat>, alt!(stats_percentage | stats_ratio));

fn is_valid_tag_char(candidate: u8) -> bool {
    let candidate_char = candidate as char;
    is_alphanumeric(candidate) || candidate_char == '_' || candidate_char == '@'
}

named!(
    tag_list<Vec<&[u8]>>,
    delimited!(
        tag!(":"),
        separated_list_complete!(char!(':'), take_while!(is_valid_tag_char)),
        tag!(":")
    )
);

named!(title_rest, take_while!(call!(|x| x != b'\n')));

fn byte_slice_to_string(x: &[u8]) -> String {
    String::from(str::from_utf8(x).unwrap())
}

fn vec_byte_slice_to_vec_string(x: Vec<&[u8]>) -> Vec<String> {
    x.iter().map(|y| byte_slice_to_string(y)).collect()
}

named!(
    title_meta<TitleMeta>,
    do_parse!(
        start: opt!(title_start)
            >> opt!(multispace)
            >> stats: opt!(stats)
            >> opt!(multispace)
            >> tags: opt!(tag_list)
            >> opt!(multispace)
            >> leftovers: opt!(title_rest)
            >> (TitleMeta {
                start: start.map(String::from),
                stats: stats,
                tags: match tags {
                    None => ordset![],
                    Some(x) => OrdSet::from(vec_byte_slice_to_vec_string(x)),
                },
                leftovers: leftovers.map(byte_slice_to_string),
            })
    )
);

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

named!(
    pub headline<Headline>,
    do_parse!(
        depth: depth
            >> multispace
            >> keyword: opt!(keyword)
            >> multispace
            >> priority: opt!(priority)
            >> multispace
            >> title_meta: title_meta
            >> (Headline {
                depth: depth,
                keyword: keyword.map(String::from),
                priority: maybe_get_single_char(priority),
                stats: title_meta.stats,
                timestamp: None,
                title: match title_meta.start {
                    None => match title_meta.leftovers {
                        None => None,
                        Some(a) => Some(format!("{}", a)),
                    },
                    Some(a) => match title_meta.leftovers {
                        None => Some(format!("{}", a)),
                        Some(b) => Some(format!("{}{}", a, b))
                    }
                },
                tags: title_meta.tags
            })
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_depth() {
        assert_eq!(
            depth(b"***** TODO [#A] Heading"),
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
    fn get_title_start() {
        assert_eq!(
            title_start(b"Start of the title [1/2] :tag:list: leftovers"),
            Ok((&b"[1/2] :tag:list: leftovers"[..], "Start of the title "))
        )
    }

    #[test]
    fn get_stats_percentage() {
        assert_eq!(
            stats_percentage(b"[23%]"),
            Ok((&[][..], Stat::Percentage(23u8)))
        )
    }

    #[test]
    fn get_stats_ratio() {
        assert_eq!(stats_ratio(b"[3/4]"), Ok((&[][..], Stat::Ratio(3u8, 4u8))));
        assert_eq!(
            stats_ratio(b"[12/4]"),
            Ok((&[][..], Stat::Ratio(12u8, 4u8)))
        )
    }

    #[test]
    fn get_stats() {
        assert_eq!(stats(b"[3/4]"), Ok((&[][..], Stat::Ratio(3u8, 4u8))));
        assert_eq!(
            stats_percentage(b"[23%]"),
            Ok((&[][..], Stat::Percentage(23u8)))
        )
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
    fn get_title_rest() {
        assert_eq!(
            title_rest(b"leftovers\n"),
            Ok((&b"\n"[..], &b"leftovers"[..]))
        );
    }

    #[test]
    fn get_title_meta() {
        assert_eq!(
            title_meta(b"Some title [10%] :tag:list: rest\n"),
            Ok((&b"\n"[..], TitleMeta {
                start: Some(format!("Some title ")),
                stats: Some(Stat::Percentage(10)),
                tags: ordset![format!("tag"), format!("list")],
                leftovers: Some(format!("rest"))
            }))
        );
    }

    #[test]
    fn get_headline() {
        assert_eq!(
            headline(b"**** TODO [#A] Some title [10%] :tag:list: rest\n"),
            Ok((
                &b"\n"[..],
                Headline {
                    depth: 4,
                    keyword: Some(format!("TODO")),
                    priority: Some('A'),
                    stats: Some(Stat::Percentage(10)),
                    timestamp: None,
                    title: Some(format!("Some title rest")),
                    tags: ordset![format!("tag"), format!("list")],
                }
            ))
        );
    }
}
