use std::{fmt::Debug, borrow::BorrowMut};
use rand::{distributions::uniform::UniformInt, Rng};
use regex_syntax::ast::{
    parse::Parser, Alternation, Ast, ClassBracketed, ClassPerl, ClassSetUnion, Concat,
    Repetition, RepetitionKind,
};

const WORD: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const REPEATE_MAX:usize=20;

pub trait RegexGenerator: Debug {
    fn generate(&self,max_repeate:usize) -> String;
}

#[derive(Debug)]
struct RepeateRegexGen {
    inner: Box<dyn RegexGenerator>,
    min: usize,
    max: Option<usize>,
}

impl RegexGenerator for RepeateRegexGen {
    fn generate(&self,max_repeate:usize) -> String {
        let max;
        if self.max.is_none() {
            max = max_repeate;
        } else {
            max = self.max.unwrap();
        }
        let len = if max<= self.min {
            self.min
        } else {
            let mut rng = rand::thread_rng();
            let max_i = max.max(self.min);
            rng.gen_range(self.min..max_i)
        };
        let mut output = String::new();
        for _ in 0..len {
            let inner_gens = self.inner.generate(max_repeate);
            output.push_str(&inner_gens);
        }
        output
    }
}

impl From<&Box<Repetition>> for RepeateRegexGen {
    fn from(value: &Box<Repetition>) -> Self {
        let rgi = from_ast(&value.ast,20);
        let rrg = match &value.op.kind {
            RepetitionKind::ZeroOrOne => RepeateRegexGen {
                inner: rgi,
                min: 0,
                max: Some(2),
            },
            RepetitionKind::ZeroOrMore => RepeateRegexGen {
                inner: rgi,
                min: 0,
                max: None,
            },
            RepetitionKind::OneOrMore => RepeateRegexGen {
                inner: rgi,
                min: 1,
                max: None,
            },
            RepetitionKind::Range(r) => match r {
                regex_syntax::ast::RepetitionRange::Exactly(a) => RepeateRegexGen {
                    inner: rgi,
                    min: *a as usize,
                    max: Some(*a as usize),
                },
                regex_syntax::ast::RepetitionRange::AtLeast(a) => RepeateRegexGen {
                    inner: rgi,
                    min: *a as usize,
                    max: None,
                },
                regex_syntax::ast::RepetitionRange::Bounded(a, b) => RepeateRegexGen {
                    inner: rgi,
                    min: *a as usize,
                    max: Some((*b+1) as usize),
                },
            },
        };
        rrg
    }
}

#[derive(Debug)]
struct AltRegexGen {
    inner: Vec<Box<dyn RegexGenerator>>,
}

impl AltRegexGen {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}

impl RegexGenerator for AltRegexGen {
    fn generate(&self,max_repeate:usize) -> String {
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(0..self.inner.len());
        self.inner
            .get(len)
            .map(|rg| rg.generate(max_repeate))
            .unwrap_or_default()
    }
}

impl From<&Box<Alternation>> for AltRegexGen {
    fn from(value: &Box<Alternation>) -> Self {
        let mut alt_rgen = AltRegexGen::new();
        for ast in value.asts.iter() {
            alt_rgen.inner.push(from_ast(ast,20));
        }
        alt_rgen
    }
}

#[derive(Debug)]
struct ConcatRegexGen {
    inner: Vec<Box<dyn RegexGenerator>>,
}

impl ConcatRegexGen {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}

impl From<&Box<Concat>> for ConcatRegexGen {
    fn from(value: &Box<Concat>) -> Self {
        let mut concat = ConcatRegexGen::new();
        let mut lit_temp: Option<String> = None;
        for ast in value.asts.iter() {
            if let Ast::Literal(lit) = ast {
                if let Some(rgi) = lit_temp.as_mut() {
                    rgi.push(lit.c);
                } else {
                    let c = lit.c;
                    lit_temp = Some(c.to_string());
                }
            } else {
                if let Some(rgi) = lit_temp {
                    let temp_rgi = RegexGenItem::new(rgi);
                    concat.inner.push(Box::new(temp_rgi));
                    lit_temp = None
                }
                concat.inner.push(from_ast(ast,REPEATE_MAX));
            }
        }
        if let Some(rgi) = lit_temp {
            let temp_rgi = RegexGenItem::new(rgi);
            concat.inner.push(Box::new(temp_rgi));
        }
        concat
    }
}

impl RegexGenerator for ConcatRegexGen {
    fn generate(&self,max_repeate:usize) -> String {
        let mut output = String::new();
        for rg in self.inner.iter() {
            let gen_str = rg.generate(max_repeate);
            output.push_str(&gen_str);
        }
        output
    }
}

#[derive(Debug)]
struct RegexGenItem {
    candidate: Vec<String>,
}

impl RegexGenerator for RegexGenItem {
    fn generate(&self,_max_repeate:usize) -> String {
        let mut rng = rand::thread_rng();
        if self.candidate.len() == 0 {
            return "".to_string();
        }
        let pos: usize = if self.candidate.len() == 1 {
            0
        } else {
            rng.gen_range(0..self.candidate.len())
        };
        let str = self.candidate.get(pos).unwrap();
        str.to_owned()
    }
}

impl RegexGenItem {
    pub fn new(str: String) -> Self {
        Self {
            candidate: vec![str],
        }
    }

    pub fn empty() -> Self {
        Self { candidate: vec![] }
    }

    pub fn range(start: char, end: char) -> Self {
        let st = WORD.chars().position(|c| c == start).unwrap_or(0);
        let ed = WORD.chars().position(|c| c == end).unwrap_or(WORD.len())+1;

        let d: Vec<String> = WORD
            .chars()
            .skip(st)
            .take(ed - st)
            .map(|x| x.to_string())
            .collect();
        Self { candidate: d }
    }

    pub fn digit() -> Self {
        let d: Vec<String> = "0123456789".chars().map(|c| c.to_string()).collect();
        Self { candidate: d }
    }

    pub fn word() -> Self {
        let d: Vec<String> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .map(|c| c.to_string())
            .collect();
        Self { candidate: d }
    }

    pub fn space() -> Self {
        let d: Vec<String> = "\t\r\n ".chars().map(|c| c.to_string()).collect();
        Self { candidate: d }
    }

    pub fn merge(&mut self, mut rgi: RegexGenItem) {
        self.candidate.append(&mut rgi.candidate);
    }

    // pub fn add(&mut self, str: String) {
    //     self.candidate.push(str);
    // }
}

impl From<&ClassPerl> for RegexGenItem {
    fn from(value: &ClassPerl) -> Self {
        let negated = value.negated;
        match value.kind {
            regex_syntax::ast::ClassPerlKind::Digit => {
                if negated {
                    let mut word = RegexGenItem::word();
                    word.merge(RegexGenItem::space());
                    word
                } else {
                    RegexGenItem::digit()
                }
            }
            regex_syntax::ast::ClassPerlKind::Space => {
                if negated {
                    let mut word = RegexGenItem::word();
                    word.merge(RegexGenItem::digit());
                    word
                } else {
                    RegexGenItem::space()
                }
            }
            regex_syntax::ast::ClassPerlKind::Word => {
                if negated {
                    let mut word = RegexGenItem::space();
                    word.merge(RegexGenItem::digit());
                    word
                } else {
                    RegexGenItem::word()
                }
            }
        }
    }
}

pub fn regex_generator(regex: &str,max_repeate:usize) -> Box<dyn RegexGenerator> {
    let mut parser = Parser::new();
    let hir = parser.parse(regex).unwrap();
    from_ast(&hir,max_repeate)
}

pub fn regex_gen(regex: &str,max_repeate:usize) -> String {
    regex_generator(regex,max_repeate).generate(max_repeate)
}

pub struct RegexGenerate {
    repeat_max:usize,
    regex_generator:Option<Box<dyn RegexGenerator>>
}

impl RegexGenerate {

    pub fn max_repeate(max:usize) -> Self {
        Self {
            repeat_max:max,
            regex_generator:None
        }
    }

    pub fn parse(self,regex: &str) -> Self {
        let regen = regex_generator(regex, self.repeat_max);
        Self { repeat_max: self.repeat_max, regex_generator: Some(regen) }
    }

    pub fn generate(&self) -> String {
        self.regex_generator.as_ref().map(|f|f.generate(self.repeat_max)).unwrap_or_default()
    }

}

fn deal_classunion(cu: &ClassSetUnion) -> Box<dyn RegexGenerator> {
    let mut altg = AltRegexGen::new();
    for ci in cu.items.iter() {
        match ci {
            regex_syntax::ast::ClassSetItem::Empty(_) => {}
            regex_syntax::ast::ClassSetItem::Literal(l) => {
                altg.inner
                    .push(Box::new(RegexGenItem::new(l.c.to_string())));
            }
            regex_syntax::ast::ClassSetItem::Range(r) => {
                let stc = r.start.c;
                let edc = r.end.c;
                altg.inner.push(Box::new(RegexGenItem::range(stc, edc)))
            }
            regex_syntax::ast::ClassSetItem::Ascii(_) => todo!(),
            regex_syntax::ast::ClassSetItem::Unicode(_) => todo!(),
            regex_syntax::ast::ClassSetItem::Perl(p) => {
                let rgi: RegexGenItem = p.into();
                altg.inner.push(Box::new(rgi));
            }
            regex_syntax::ast::ClassSetItem::Bracketed(cbk) => {
                let dckb = deal_classbracketed(cbk.as_ref());
                altg.inner.push(dckb);
            }
            regex_syntax::ast::ClassSetItem::Union(u) => {
                let cu = deal_classunion(u);
                altg.inner.push(cu);
            }
        }
    }
    Box::new(altg)
}

fn deal_classbracketed(cbk: &ClassBracketed) -> Box<dyn RegexGenerator> {
    let negated = cbk.negated;
    if negated {
       Box::new(RegexGenItem::empty()) 
    } else {
        let rg: Box<dyn RegexGenerator> = match &cbk.kind {
            regex_syntax::ast::ClassSet::Item(i) => match i {
                regex_syntax::ast::ClassSetItem::Empty(_) => Box::new(RegexGenItem::empty()),
                regex_syntax::ast::ClassSetItem::Literal(l) => {
                    Box::new(RegexGenItem::new(l.c.to_string()))
                }
                regex_syntax::ast::ClassSetItem::Range(r) => {
                    let stc = r.start.c;
                    let edc = r.end.c;
                    Box::new(RegexGenItem::range(stc, edc))
                }
                regex_syntax::ast::ClassSetItem::Ascii(_a) => todo!(),
                regex_syntax::ast::ClassSetItem::Unicode(_u) => todo!(),
                regex_syntax::ast::ClassSetItem::Perl(cp) => {
                    let rgi: RegexGenItem = cp.into();
                    Box::new(rgi)
                }
                regex_syntax::ast::ClassSetItem::Bracketed(b) => deal_classbracketed(b.as_ref()),
                regex_syntax::ast::ClassSetItem::Union(u) => deal_classunion(u),
            },
            regex_syntax::ast::ClassSet::BinaryOp(_b) => Box::new(RegexGenItem::empty()),
        };
        rg
    }
}

fn from_ast(value: &Ast,max_repeate:usize) -> Box<dyn RegexGenerator> {
    match value {
        Ast::Empty(_) => Box::new(RegexGenItem::empty()),
        Ast::Flags(_) => Box::new(RegexGenItem::empty()),
        Ast::Literal(l) => {
            let lstr = String::from(l.c);
            Box::new(RegexGenItem::new(lstr))
        }
        Ast::Dot(_) => Box::new(RegexGenItem::word()),
        Ast::Assertion(_) => Box::new(RegexGenItem::empty()),
        Ast::ClassUnicode(_cu) => todo!(),
        Ast::ClassPerl(cp) => {
            let rgi: RegexGenItem = cp.as_ref().into();
            Box::new(rgi)
        }
        Ast::ClassBracketed(cbk) => deal_classbracketed(cbk.as_ref()),
        Ast::Repetition(r) => {
            let rg: RepeateRegexGen = r.into();
            Box::new(rg)
        }
        Ast::Group(g) => from_ast(&g.ast,max_repeate),
        Ast::Alternation(alter) => {
            let rg: AltRegexGen = alter.into();
            Box::new(rg)
        }
        Ast::Concat(concat) => {
            let rg: ConcatRegexGen = concat.into();
            Box::new(rg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex_syntax::ast::parse::Parser;

    #[test]
    fn regex_syntax_0() {
        let mut parser = Parser::new();
        let hir = parser.parse("ab+(啊啊吧)+").unwrap();

        println!("{:#?}", hir);
    }
    #[test]
    fn regex_syntax_1() {
        let mut parser = Parser::new();
        let hir = parser.parse("\\d+|\\w*|\\s{4,9}|\\S|\\W").unwrap();

        println!("{:#?}", hir);
    }

    #[test]
    fn regex_syntax_2() {
        let mut parser = Parser::new();
        let hir = parser.parse("[a-z]").unwrap();

        println!("{:#?}", hir);
    }
    #[test]
    fn regex_syntax_3() {
        let mut parser = Parser::new();
        let hir = parser.parse("a@a.com").unwrap();

        println!("{:#?}", hir);
    }

    #[test]
    fn regex_gen_simple_1() {
        let a = regex_gen("a啊@(abc|efd|xyz)\\.com",20);
        println!("{}", a)
    }

    #[test]
    fn regex_gen_simple_2() {
            let a = regex_gen("([1-9][0-9]*)+(\\.[0-Z]{1,2})?",5);
            println!("{}", a)
    }

    #[test]
    fn regex_gen_simple_3() {
        for _ in 0..5 {
            let a = regex_gen(r"\d{3}-\d{8}|\d{4}-\d{7}",20);
            println!("{}", a)
        }
    }
    #[test]
    fn regex_gen_syntax() {
        let mut parser = Parser::new();
        let pattern = "abc\\.(com)?";
        let ast = parser.parse(pattern).unwrap();
        let rgi = from_ast(&ast,20);
        dbg!(rgi);
    }

    #[test]
    fn test_use() {
        for _ in 0..5 {
            let s = RegexGenerate::max_repeate(10).parse("优秀|良好|及格|不及格").generate();
            println!("{}",s);
        }
    }

    #[test]
    fn regex_gen_item() {
        let rgi = RegexGenItem::digit();
        for _ in 0..10 {
            println!("{}", rgi.generate(20));
        }
    }

    #[test]
    fn regex_alt_gen() {
        let mut alt = AltRegexGen::new();
        alt.inner
            .push(Box::new(RegexGenItem::new("选项1".to_owned())));
        alt.inner
            .push(Box::new(RegexGenItem::new("选项2".to_owned())));
        alt.inner
            .push(Box::new(RegexGenItem::new("选项3".to_owned())));
        for _ in 0..10 {
            println!("{}", alt.generate(20));
        }
    }
}
