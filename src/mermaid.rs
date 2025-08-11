use std::collections::HashMap;
use crate::Branch;
use std::fs::File;
use std::io::Write;
use crate::{AST, Token, TokenContext, TokenStream};

const MARKDOWN_PREVIEW : bool = false;

pub struct MermaidGraph {
    defined: HashMap<usize, String>,
    node_lines: Vec<String>,
    edge_lines: Vec<String>,
    id_counter: usize,
    expression: Option<String>,
    variables: Vec::<String>,
}

impl MermaidGraph {
    fn new() -> Self {
        Self {
            defined: HashMap::new(),
            node_lines: Vec::new(),
            edge_lines: Vec::new(),
            id_counter: 0,
            expression: None,
            variables: Vec::new()
        }
    }

    pub fn from_ast(ast: &Branch) -> Self {
        let mut graph = Self::new();
        graph.recurse_tree(ast);
        graph
    }

    pub fn from_expr(expr: String, vars: &[&str]) -> Self {
        let mut graph = Self::new();
        graph.expression = Some(expr.clone());
        for v in vars {
            graph.variables.push(v.to_owned().to_owned());
        }
        
        let mut ts = TokenStream::new();
        ts.update(&expr,&vars).unwrap(); //TODO eliminate this unwrap
        let mut ast = AST::new(ts);
        _ = ast.parse_tokens().unwrap();

        graph.recurse_tree(&ast.tree.unwrap());
        graph
    }

    pub fn write_output(&self, path: &str) -> std::io::Result<()> {
        let mut output = File::create(path)?;
        if MARKDOWN_PREVIEW { 
            writeln!(output, "::: mermaid")?;
        }
        
        Self::write_header(&mut output)?;

        if let Some(expr) = &self.expression {
            writeln!(output, "  expr@{{ shape: doc, label: \"{}\" }}", 
                expr.replace("*", "⋅"))?;
        }
        for (i,var) in self.variables.iter().enumerate() {
            writeln!(output, "  var{}@{{ shape: cyl, label: \"{}\" }}", i, var)?;
        }
        for l in &self.node_lines {
            writeln!(output, "  {}", l)?;
        }
        for l in &self.edge_lines {
            writeln!(output, "  {}", l)?;
        }

        if MARKDOWN_PREVIEW {
            writeln!(output, ":::")?;
        }
        Ok(())
    }

    fn write_header(output: &mut File) -> std::io::Result<()> {
        writeln!(output, "---")?;
        writeln!(output, "config:")?;        
        writeln!(output, "  layout: elk")?;
        writeln!(output, "  look: handDrawn")?;
        writeln!(output, "  theme: light")?;
        writeln!(output, "---")?;
        writeln!(output, "flowchart TB")?;
        Ok(())
    }

    fn add_node(&mut self, tc: &TokenContext) {
        let id = format!("{}", self.id_counter);
        let node_string = match tc.token {
            // TODO: fill in missing cases
            Token::ArOp(x) => format!("{{\" \\{} \"}}", x),
            Token::Func(_, 1) => format!("[\\ {} /]", tc.token),
            Token::Func(_, _) => format!("> {} ]", tc.token),
            Token::Const(_) => format!("[[ {} ]]", tc.token),
            Token::Number(_)=> format!("[ {} ]", tc.token),
            Token::Var(_)=> format!("( {} )", tc.token),
            _ => format!("[\" {} \"]", tc.token)
        };

        self.node_lines.push(format!("S{}{}", id, node_string));
        self.defined.insert(tc.at, id);
        self.id_counter += 1;
    }

    fn add_edge(&mut self, fr: usize, to: usize) {
        let id_from = self.defined.get(&fr).unwrap();
        let id_to = self.defined.get(&to).unwrap();
        self.edge_lines.push(format!("S{}-->S{}", id_from, id_to));
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
        let graph = MermaidGraph::from_expr(expr.into(), input_var);

        if let Err(e) = graph.write_output("./mermaid/mermaid_graph1.mmd") {
            panic!("Cannot write graph: {}", e)
        };        
    }

}