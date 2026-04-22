use std::collections::HashMap;
use dioxus::html::tr;
use indexmap::{IndexMap};

use dymex::{DymexError, DynMath, Float, Token, TokenContext};
use dymex::{Evaluator, InputVars, EvaluationError, AST};
use std::rc::Rc;

const MAXLEN : usize = 1024 * 64;
const MACRO_GRID : &str = "!grid";
const MACRO_LINSPACE : &str = "!linspace";

#[derive(Clone)]
pub struct VarData {
    // name: String,
    pub text: String,
    pub value: Option<Rc<dyn DynMath>>
}
impl VarData {
    pub fn from_text(text: &str) -> Self {
        Self {
            text: text.to_string(),
            value: parse_variable_value(text)
        }
    }
}

pub fn parse_variable_value(varstring: &str) -> Option<Rc<dyn DynMath>> {
    if let Ok(x) = varstring.parse::<f64>() {
        return Some(Rc::new(x));
    }
    if let Some(v) = parse_macro_input(varstring) {
        return Some(Rc::new(v));
    }
    None
}

pub fn parse_macro_input(s: &str) -> Option<Vec<f64>> {
    if s.starts_with(MACRO_GRID) {
        let args = parse_macro(s, MACRO_GRID, 3);
        if let Some(args) = args {
            let x0 = args[0].parse::<f64>();
            let x1 = args[1].parse::<f64>();
            let dx = args[2].parse::<f64>();
            match (x0, x1, dx) {
                (Ok(x0), Ok(x1), Ok(dx)) => { return Some(generate_grid(x0, x1, dx));},
                _ => {return None;}
            }
        }
    }
    if s.starts_with(MACRO_LINSPACE) {
        let args = parse_macro(s, MACRO_LINSPACE, 3);
        if let Some(args) = args {
            let x0 = args[0].parse::<f64>();
            let x1 = args[1].parse::<f64>();
            let n = args[2].parse::<usize>();
            match (x0, x1, n) {
                (Ok(x0), Ok(x1), Ok(n)) => { return Some(generate_linspace(x0, x1, n));},
                _ => {return None;}
            }
        }
    }
    // if s.starts_with("!grid") {
    //     let chars = s.trim().as_bytes();
    //     let l = chars.get(5);
    //     let r = chars.last();
    //     match (l, r) {
    //         (Some(b'('), Some(b')')) => {
    //             let args_substring = &chars[6..chars.len()-1];
    //             if let Some(args) = split_macro_args(args_substring, 3) {
    //                 let x0 = args[0].parse::<f64>();
    //                 let x1 = args[1].parse::<f64>();
    //                 let dx = args[2].parse::<f64>();
    //                 match (x0, x1, dx) {
    //                     (Ok(x0), Ok(x1), Ok(dx)) => { return Some(generate_grid(x0, x1, dx));},
    //                     _ => {return None;}
    //                 }
    //             } else {
    //                 return None;
    //             }
    //         },
    //         _ => {
    //             return None;
    //         }
    //     }
    // }
    None
}

fn parse_macro(s: &str, pattern: &str, n_args: usize) -> Option<Vec<String>> {
    let s = s.trim();
    if !s.starts_with(pattern) {
        return  None;
    }
    let chars = s.trim().as_bytes();
    let l = chars.get(pattern.len());
    let r = chars.last();
    return match (l, r) {
        (Some(b'('), Some(b')')) => {
            let args_substring = &chars[pattern.len()+1..chars.len()-1];
            split_macro_args(args_substring, n_args)
        },
        _ => None
    }
}

fn split_macro_args(args: &[u8], npar: usize) -> Option<Vec<String>> {
    let s = str::from_utf8(args);
    if let Ok(s) = s {
        let args: Vec<String> = s.split(",").map(|s| s.to_string()).collect();
        if npar != args.len() {
            return None;
        }
        return Some(args)
    }
    None
}


fn generate_grid(x0: f64, x1: f64, dx: f64) -> Vec<f64> {
    let n = ((x1-x0).abs() / dx).ceil() as usize;
    assert!(n < MAXLEN, "Size of requested vector is too large"); //TODO remove assert, fail gracefuilly
    let mut grid = Vec::with_capacity(n);
    if x0 < x1 {
        let mut x = x0;
        loop {
            if x > x1 {
                break;
            }
            grid.push(x);
            x += dx;
        }
    } else {
        let mut x = x1;
        loop {
            if x < x0 {
                break;
            }
            grid.push(x);
            x -= dx;
        }
    }
    grid
}

fn generate_linspace(x0: f64, x1: f64, n: usize) -> Vec<f64> {
    let dx = (x1-x0) / (n as f64);
    let mut v = Vec::with_capacity(n);
    let mut x = x0;
    for _ in 0..n {
        v.push(x);
        x += dx;
    }
    v
}