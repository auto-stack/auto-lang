use crate::ast::Expr;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Arrow {
    pub src: Option<Expr>,
    pub dest: Option<Expr>,
    pub with: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct CondArrow {
    pub src: Option<Expr>,
    pub cond: Expr,
    pub subs: Vec<Arrow>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Arrow(Arrow),
    CondArrow(CondArrow),
}

#[derive(Debug, Clone)]
pub struct OnEvents {
    pub branches: Vec<Event>,
}

impl Arrow {
    pub fn new(src: Option<Expr>, dest: Option<Expr>, with: Option<Expr>) -> Self {
        Self { src, dest, with }
    }
}

impl CondArrow {
    pub fn new(src: Option<Expr>, cond: Expr, subs: Vec<Arrow>) -> Self {
        Self { src, cond, subs }
    }
}

impl OnEvents {
    pub fn new(branches: Vec<Event>) -> Self {
        Self { branches }
    }
}

impl fmt::Display for Arrow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src)?;
        }
        if let Some(dest) = &self.dest {
            write!(f, " (to {})", dest)?;
        }
        if let Some(with) = &self.with {
            write!(f, " (with {})", with)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for CondArrow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(cond-arrow")?;
        if let Some(src) = &self.src {
            write!(f, " (from {})", src)?;
        }
        write!(f, " (cond {})", self.cond)?;
        for sub in &self.subs {
            write!(f, " {}", sub)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Arrow(a) => write!(f, "{}", a),
            Event::CondArrow(c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Display for OnEvents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(on")?;
        for branch in &self.branches {
            write!(f, " {}", branch)?;
        }
        write!(f, ")")
    }
}
