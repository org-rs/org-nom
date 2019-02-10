use std::str;
use im::OrdSet;
use nom::{is_alphabetic, is_alphanumeric, is_digit};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Headline {
    depth: u8,
    keyword: Option<String>,
    priority: Option<char>,
    title: Option<String>,
    timestamp: Option<String>,
    stats: Option<Stat>,
    tags: OrdSet<String>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct TitleMeta {
    start: Option<String>,
    stats: Option<Stat>,
    tags: OrdSet<String>,
    leftovers: Option<String>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum Stat {
    Percentage(u8),
    Ratio(u8, u8),
}

named!(
    headline_depth<usize>,
    fold_many1!(tag!("*"), 0, |depth, _| depth + 1)
);

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

named!(
    stats<Stat>,
    alt!(stats_percentage | stats_ratio)
);

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

/*
named!(
    title_meta<TitleMeta>,
    do_parse!(
        start: opt!(title_start)
            >> stats: opt!(stats)
            >> tags: opt!(tag_list)
            >> leftovers: opt!(title_rest)
            >> (TitleMeta {
                start: start,
                stats: stats,
                tags: tags,
                leftovers: leftovers,
            })
    )
);
*/

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
        assert_eq!(
            stats_ratio(b"[3/4]"),
            Ok((&[][..], Stat::Ratio(3u8, 4u8)))
        );
        assert_eq!(
            stats_ratio(b"[12/4]"),
            Ok((&[][..], Stat::Ratio(12u8, 4u8)))
        )
    }

    #[test]
    fn get_stats() {
        assert_eq!(
            stats(b"[3/4]"),
            Ok((&[][..], Stat::Ratio(3u8, 4u8)))
        );
        assert_eq!(
            stats_percentage(b"[23%]"),
            Ok((&[][..], Stat::Percentage(23u8)))
        )
    }
}
