use std::{collections::HashMap};
use crate::Branch;
use std::fs::File;
use std::io::{Write};
use crate::{AST, Token, TokenContext, TokenStream};

// TODO Ended up being really clunky, too much cloning, mut ...
// TODO
// TODO


const MARKDOWN_PREVIEW : bool = false;
const SIMPLE_HEADER: &str =
"---
config:
  layout: elk
  look: handDrawn
  theme: light
---
flowchart TB";
const FANCY_HEADER: &str =
"---
config:
  layout: elk
---
flowchart TB";
const CSS_HEADER: &str =
"---
config:
  layout: elk
---
flowchart TB";


pub fn styled_ast_graph(ast: &Branch, style: &MermaidStyle) -> String {
    let mut m = MermaidGraph::from_ast(ast).with_style(style);
    m.to_string()
}

fn assign_mermaid_class(t: &Token) -> &str {
    match &t {
        Token::ArOp(_)
        |Token::RelOp(_)
        |Token::LogicOp(_)
        |Token::AssignOp(_)
        |Token::Dot => "mmdOp",
        Token::Number(_)
        |Token::Const(_) => "mmdConst",
        Token::Var(_) => "mmdVar",
        Token::Func(_, _) => "mmdFunc",
        _ => ""
    }
}

#[derive(Clone, Debug)]
pub enum MermaidStyleEnum {
    Plain,
    Fancy,
    CSS
}

#[derive(Clone)]
pub struct MermaidStyle {
    pub style: MermaidStyleEnum,
    pub include_expr: bool,
    pub include_variables: bool,
    pub node_styles: HashMap<String, String>,
}
impl MermaidStyle {
    pub fn new() -> Self {
        Self {
            style: MermaidStyleEnum::Plain,
            include_expr: true,
            include_variables: true,
            node_styles: HashMap::new()
        }
    }
}

pub struct MermaidGraph {
    ast: Option<Branch>,
    defined: HashMap<usize, String>,
    node_lines: Vec<String>,
    edge_lines: Vec<String>,
    id_counter: usize,
    expression: Option<String>,
    variables: Vec::<String>,
    pub style_options: MermaidStyle
}

impl MermaidGraph {
    fn new() -> Self {
        Self {
            ast: None,
            defined: HashMap::new(),
            node_lines: Vec::new(),
            edge_lines: Vec::new(),
            id_counter: 0,
            expression: None,
            variables: Vec::new(),
            style_options: MermaidStyle::new(),
        }
    }

    pub fn with_style(mut self, style: &MermaidStyle) -> Self {
        self.style_options = style.clone();
        self
    }

    pub fn from_ast(ast: &Branch) -> Self {
        let mut graph = Self::new();
        graph.ast = Some(ast.clone());
        graph
    }

    pub fn from_expr(expr: String, vars: &[&str]) -> Self {
        let mut graph = Self::new();
        graph.expression = Some(expr.clone());
        for v in vars {
            graph.variables.push(v.to_owned().to_owned());
        }

        let ts = TokenStream::new(&expr).unwrap(); //TODO eliminate this unwrap
        let ast = AST::new(ts).unwrap();

        graph.ast = Some(ast.tree);
        graph
    }

    pub fn to_string(&mut self) -> String {
        // String does not implement std::io::Write
        // https://users.rust-lang.org/t/how-do-i-write-to-an-in-memory-buffered-string/45035/2

        self.generate_graph();

        let mut output = Vec::new();
        //TODO: clean up unwraps?
        self.write_to_buffer(&mut output).unwrap(); // writing to string should be trivial
        String::from_utf8(output).unwrap()
    }

    pub fn write_to_buffer<T: Write>(&self, output: &mut T)-> std::io::Result<()> {
        self.write_header(output)?;
        self.write_expression(output)?;
        self.write_variables(output)?;
        for l in &self.node_lines {
            writeln!(output, "  {}", l)?;
        }
        for l in &self.edge_lines {
            writeln!(output, "  {}", l)?;
        }
        self.write_classdef(output)?;
        Ok(())
    }

    pub fn write_to_file(&mut self, path: &str) -> std::io::Result<()> {
        self.generate_graph();
        let mut output = File::create(path)?;
        if MARKDOWN_PREVIEW {
            writeln!(output, "::: mermaid")?;
            self.write_to_buffer(&mut output)?;
            writeln!(output, ":::")?;
            return Ok(())
        }
        self.write_to_buffer(&mut output)?;
        Ok(())
    }

    fn style(&self) -> &MermaidStyleEnum {
        &self.style_options.style
    }

    fn write_header<T: Write>(&self, output: &mut T) -> std::io::Result<()> {
        match &self.style() {
            MermaidStyleEnum::CSS => writeln!(output, "{}", CSS_HEADER)?,
            MermaidStyleEnum::Fancy => writeln!(output, "{}", FANCY_HEADER)?,
            MermaidStyleEnum::Plain => writeln!(output, "{}", SIMPLE_HEADER)?,
        }
        Ok(())
    }

    fn write_expression<T: Write>(&self, output: &mut T) -> std::io::Result<()> {
        if self.style_options.include_expr {
            if let Some(expr) = &self.expression {
                writeln!(output, "  expr@{{ shape: doc, label: \"{}\" }}", expr.replace("*", "⋅"))?;
            }
        }
        Ok(())
    }

    fn write_variables<T: Write>(&self, output: &mut T) -> std::io::Result<()> {
        if self.style_options.include_variables {
            for (i,var) in self.variables.iter().enumerate() {
              writeln!(output, "  var{}@{{ shape: cyl, label: \"{}\" }}", i, var)?;
            };
        }
        Ok(())

    }

    fn write_classdef<T: Write>(&self, output: &mut T) -> std::io::Result<()> {
        for (k, v) in &self.style_options.node_styles {
            writeln!(output, "classDef {} {}", k, v)?;
        }
        Ok(())
    }


    fn add_node(&mut self, tc: &TokenContext) {
        let id = format!("{}", self.id_counter);
        let node_string = match &self.style() {
            MermaidStyleEnum::Fancy => self.fancy_node(tc),
            MermaidStyleEnum::Plain => self.plain_node(tc),
            MermaidStyleEnum::CSS => self.plain_node(tc),
        };

        self.node_lines.push(format!("S{}{}", id, node_string));
        self.defined.insert(tc.at, id);
        self.id_counter += 1;
    }

    fn plain_node(&self, tc: &TokenContext) -> String {
        match tc.token {
            // TODO: fill in missing cases
            Token::ArOp(x) => format!("{{\" \\{} \"}}", x),
            Token::Func(_, 1) => format!("[\\ {} /]", tc.token),
            Token::Func(_, _) => format!("> {} ]", tc.token),
            Token::Const(_) => format!("[[ {} ]]", tc.token),
            Token::Number(_)=> format!("[ {} ]", tc.token),
            Token::Var(_)=> format!("( {} )", tc.token),
            _ => format!("[\" {} \"]", tc.token)
        }
    }

    fn fancy_node(&self, tc: &TokenContext) -> String {
        let mut base = match tc.token {
            // TODO: fill in missing cases
            Token::ArOp(x) => format!("(\" \\{} \")", x),
            Token::Func(_, 1) => format!("( {} )", tc.token),
            Token::Func(_, _) => format!("( {} )", tc.token),
            Token::Const(_) => format!("( {} )", tc.token),
            Token::Number(_)=> format!("( {} )", tc.token),
            Token::Var(_)=> format!("( {} )", tc.token),
            _ => format!("(\" {} \")", tc.token)
        };
        base.push_str(":::");
        base.push_str(assign_mermaid_class(&tc.token));
        base
    }

    fn add_edge(&mut self, fr: usize, to: usize) {
        let id_from = self.defined.get(&fr).unwrap();
        let id_to = self.defined.get(&to).unwrap();
        self.edge_lines.push(format!("S{}-->S{}", id_from, id_to));
    }

    fn generate_graph(&mut self) {
        if let Some(ast) = &self.ast {
            self.recurse_tree(&ast.clone()); // TODO: unecessary clone
        }
    }

    fn recurse_tree(&mut self, ast: &Branch) {
        match ast {
            Branch::Atom(tc) =>
                self.add_node(tc),
            Branch::Expression(tc_from, children) => {
                self.add_node(tc_from);
                for branch in children {
                    self.recurse_tree(branch);
                    let (Branch::Atom(tc_to) | Branch::Expression(tc_to, _)) = branch;
                    self.add_edge(tc_from.at, tc_to.at);
                }
            }
        }
    }
}




#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn create_graph1() {
        let input_var = &["x", "y"];
        let expr = "(2.0*pi * exp(-x*x)) / max(1.0 + sqrt(y), 0)";
        let mut graph = MermaidGraph::from_expr(expr.into(), input_var);

        if let Err(e) = graph.write_to_file("../mermaid/mermaid_graph1.mmd") {
            panic!("Cannot write graph: {}", e)
        };
    }

    #[test]
    fn create_assignment() {
        let input_var = &["x", "y"];
        let expr = "z = x+y";
        let mut graph = MermaidGraph::from_expr(expr.into(), input_var);

        if let Err(e) = graph.write_to_file("../mermaid/mermaid_graph2.mmd") {
            panic!("Cannot write graph: {}", e)
        };
    }


}