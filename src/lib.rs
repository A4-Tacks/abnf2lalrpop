use std::fmt;

use smol_strc::SmolStr;

fn lit(expected: &str) -> impl Fn(&str, usize) -> peg::RuleResult<()> {
    move |s, i| {
        if s[i..].starts_with(expected) {
            peg::RuleResult::Matched(i+expected.len(), ())
        } else {
            peg::RuleResult::Failed
        }
    }
}

peg::parser!(pub grammar parser() for str {
    rule ident() -> SmolStr
        = !"r#" !"r\""
        s:quiet!{$((
            ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']
            / [ch if !ch.is_ascii()]
        )+)} {s.replace('-', "_").into()}
        / expected!("ident")
    rule string()
        = quiet!{"r"? d:$("#"*) "\"" (
            !("\"" #{lit(d)})
            [_]
        )* "\"" #{lit(d)}}
        / expected!("string")
    rule _() = quiet!{[' '|'\t'|'\r'|'\n']*}
    rule atom() -> SmolStr
        = s:$(string()) {s.into()}
        / i:ident() !(_ "=") {i}
    rule pat() -> Pat
        = s:atom() { Pat::Atom(s) }
        / "*" _ p:pat() { Pat::Repeat(vec![p], '*') }
        / "+" _ p:pat() { Pat::Repeat(vec![p], '+') }
        / "(" _ list:pat()**_ _ ")" { Pat::List(list) }
        / "[" _ list:pat()++_ _ "]" { Pat::Repeat(list, '?') }
    rule def() -> Rule
        = name:ident() _ "=" _ body:(l:pat()++_ {Pat::List(l)})++(_ "/" _) (_ ";")?
        {
            Rule { name, body }
        }
    pub rule defs() -> Vec<Rule>
        = _ d:(d:def()_{d})* {d}
});

#[derive(Debug)]
pub struct Rule {
    name: SmolStr,
    body: Vec<Pat>,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name, body } = self;
        write!(f, "{name}: () = {{\n")?;
        for pats in body {
            let pats = Some(pats).into_iter().flat_map(|pat| match pat {
                Pat::List(pats) => pats.as_slice(),
                pat => std::slice::from_ref(pat),
            });
            write!(f, "    ")?;
            for pat in pats {
                write!(f, "{pat} ")?;
            }
            writeln!(f, "=> (),")?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub enum Pat {
    List(Vec<Pat>),
    Repeat(Vec<Pat>, char),
    Atom(SmolStr),
}

impl fmt::Display for Pat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pat::List(pats) => {
                write!(f, "(")?;
                fmt_pats(f, pats, |pat, f| pat.fmt(f))?;
                write!(f, ")")?;
            },
            Pat::Repeat(pats, ch) => {
                if pats.len() != 1 { write!(f, "(")?; }
                fmt_pats(f, pats, |pat, f| pat.fmt(f))?;
                if pats.len() != 1 { write!(f, ")")?; }
                write!(f, "{ch}")?;
            },
            Pat::Atom(atom) => write!(f, "{atom}")?,
        }
        Ok(())
    }
}

fn fmt_pats(
    f: &mut fmt::Formatter<'_>,
    pats: &Vec<Pat>,
    mut with: impl FnMut(&Pat, &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>,
) -> Result<(), fmt::Error> {
    Ok(if let Some(first) = pats.first() {
        with(first, f)?;

        for pat in &pats[1..] {
            write!(f, " ")?;
            with(pat, f)?;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings() {
        let data = [
            r###""""###,
            r###""f""###,
            r###""foo""###,
            r###"r"foo""###,
            r###"r#"foo"#"###,
            r###"r#"f"oo"#"###,
            r###"r#"f""oo"#"###,
            r###"r##"f"#oo"##"###,
            r###"r##"f"测试#oo"##"###,
        ];
        for str in data {
            let src = format!("x = {str}");
            let res = parser::defs(&src);
            match res {
                Ok(_) => (),
                Err(e) => {
                    let d = "^";
                    let n = e.location.column;
                    panic!("{src}\n{d:>n$} {e}")
                },
            }
        }
    }

    #[test]
    fn it_works() {
        let src = r#"
        x = "x"
        y = x "foo" extern-rule
          / x x
        "#;
        let dst = r#"
        x: () = {
            "x" => (),
        }
        y: () = {
            x "foo" extern_rule => (),
            x x => (),
        }"#;
        let defs = parser::defs(src).unwrap();
        let out = defs.iter()
            .flat_map(|x| ["\n".into(), x.to_string()])
            .collect::<String>();
        let dst = dst.replace("\n        ", "\n");
        assert_eq!(out, dst);
    }
}
