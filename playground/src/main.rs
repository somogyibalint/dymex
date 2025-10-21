use std::collections::HashMap;

use dioxus::{prelude::*};
use dioxus::logger::tracing::{Level, debug, error, info, warn};
use dioxus_primitives::hover_card::{HoverCard, HoverCardTrigger, HoverCardContent};
use dioxus_primitives::ContentSide;
use dioxus_primitives::tabs::{Tabs, TabList, TabContent,TabTrigger};

use dymex::{styled_ast_graph, MermaidStyle, MermaidStyleEnum, TokenContext, TokenStream, AST};
use dymex::Latex;

static CSS: Asset = asset!("/assets/style.css");

// static START_EXPR: &str = "(1-R)^2 / (1 - 2*R*cos(4*pi*n*d*cos(theta) / lambda) + R^2)";
// static START_VAR: [&str; 5] = ["lambda", "n","d", "theta", "R"];

static START_EXPR: &str = "1 / (1 + exp((E - E_f) / (k_b * T)))";
static START_VAR: [&str; 4] = ["E", "E_f","k_b", "T"];
static START_VAL: [f64; 4] = [0.2, 0.0, 8.617333262E-5, 297.0];

mod helpers;
use helpers::*;



// #[cfg(feature = "playground")]
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {

    //TODO: interact with the css somehow???
    let mmd_style: MermaidStyle = MermaidStyle {
        include_expr: false,
        include_variables: false,
        style: MermaidStyleEnum::Fancy,
        node_styles: HashMap::from([
            ("mmdVar".to_string(), "stroke:#be100e,stroke-width:4px".to_string()),
            ("mmdOp".to_string(), "stroke:#eaa549,stroke-width:4px".to_string()),
            ("mmdConst".to_string(), "stroke:#426a79,stroke-width:4px".to_string()),
            ("mmdFunc".to_string(), "stroke:#97522c,stroke-width:4px".to_string()),
        ])
    };

    let mut tokenstream =  TokenStream::new();

    let raw_expression = use_signal(|| START_EXPR.to_string());
    let variables = use_signal(|| 
        START_VAR
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<String>>()
    );

    let name_value: HashMap<String, f64> = START_VAR
        .into_iter()
        .zip(START_VAL.into_iter())
        .map(|(key, value)| {
            return (key.to_string(), value);
        }).collect();
    let input_values = use_signal(|| name_value );

    let mut lexer_msg = use_signal(|| "".to_string());
    let mut parser_msg = use_signal(|| "".to_string());
    let mut eval_msg = use_signal(|| "".to_string());
    let mut mermaid_script = use_signal(|| "".to_string());
    let mut mermaid_innerHTML = use_signal(|| "".to_string());
    let mut latex_tex = use_signal(|| "".to_string());
    let mut tokens = use_signal(|| Vec::new());
    let mut valid_expression = use_signal(|| false);

    let mut num_result = use_signal(|| Some(f64::NAN));
    let num_result_formatted = use_memo(move || format_num_result(num_result()));

    // Handle updates to the expression and variable names
    use_effect(move || {
        valid_expression.set(false);
        // info!("{}", valid_expression());
        let _v = variables();
        let varnames: Vec<&str> = _v.iter().map(|x| x.as_ref()).collect();

        // lexing
        let lexer_result = tokenstream.update(&raw_expression(),&varnames); 
        lexer_msg.set(
            match &lexer_result {
                Ok(_) => "✓".to_string(),
                Err(err) => err.user_message().full_message(&raw_expression())
            }
        );
        tokens.set(tokenstream.tokens.clone());
        
        // parsing
        if let Ok(_) = lexer_result {
            let mut ast = AST::new(tokenstream.clone());

            match ast.parse_tokens() {
                Ok(_) => {
                    parser_msg.set("✓".to_string());
                    if let Some(branch) = ast.tree {
                        mermaid_script.set(styled_ast_graph(&branch, &mmd_style));
                        latex_tex.set(format!("{}", &branch.latex()));
                        valid_expression.set(true)
                    }
                },
                Err(err) => {
                    parser_msg.set(err.user_message().full_message(&raw_expression()));
                    mermaid_script.set("".to_string());
                    latex_tex.set("".to_string());
                }
            }
        } else {
            parser_msg.set("".to_string());
            mermaid_script.set("".to_string());
        }
    });

    // Update mermaid graph
    use_effect(move || {
        let m = mermaid_script();
        mermaid_innerHTML.set(
            format!("<pre class='mermaid'> {m} </pre>")
        );
    });

    // Update evaulated value 
    use_effect(move || {
        if !valid_expression() {
            num_result.set(None);
            return;
        }
        match evaluate(
            &raw_expression.peek(), 
            &variables.peek(), 
            input_values.peek().clone()
        ) {
            Ok(x) => {
                num_result.set(Some(x));
                eval_msg.set("✓".to_string());
            },
            Err(err) => {
                num_result.set(None);
                eval_msg.set(format!("{}", err))
            }
        }
    });

    rsx! {
        document::Stylesheet { href: CSS }
        

        div { id: "title",
            text_align: "center",
            h1 { "Dymex playground" }
        }
        div { id: "latexOutput",
            class: "renderLatex"
        }

        Tabs {
            default_value: "tab1".to_string(),
            horizontal: true,
            max_width: "100%",
            class: "tabs",
            TabList {
                justify_content: "center",
                class: "tabs-list",
                TabTrigger { class: "tabs-trigger", value: "tab1".to_string(), index: 0usize, "Parse" }
                TabTrigger { class: "tabs-trigger", value: "tab2".to_string(), index: 1usize, "Eval" }
                TabTrigger { class: "tabs-trigger", value: "tab3".to_string(), index: 2usize, "Docs" }
            }
            TabContent { 
                index: 0usize, 
                value: "tab1".to_string(),
                div {
                    width: "100%",
                    // height: "5rem",
                    display: "flex",
                    flex_direction: "row",
                    align_items: "start",
                    justify_content: "center",

                    // div {
                    //     class: "hstack",
                    //     width: "70%",

                        div {id: "parse_leftcol",
                            width: "50%",
                            display: "flex",
                            flex_direction: "column",
                            justify_content: "center",
                            div { ExpressionInput { raw_expression } }
                            div { InputList { variables } }
                            div { LexerOutput {tokens, lexer_msg} }
                            div {
                                h3 {"Parser"}
                                pre {class: "errMsg", 
                                    {parser_msg}
                                }
                            }
                            div { class: "renderLatex",
                                display: "none",
                                pre { id: "latexInput",
                                {latex_tex}} 
                            }
                        }
                        div {id: "parse_rightcol",
                            width: "50%",
                            display: "flex",
                            flex_direction: "column",
                            justify_content: "center",
                            h3 {text_align: "center", "Syntax Tree"}
                            div {id: "mermaid-div",
                                justify_content: "center",
                                display: "flex",
                                flex_direction: "row",
                                dangerous_inner_html: "{mermaid_innerHTML}"
                            }
                        }
                    // }
                }
            }
            TabContent {
                index: 1usize,
                class: "tabs-content",
                value: "tab2".to_string(),
                div {
                    width: "100%",
                    display: "flex",
                    justify_content: "center",
                    flex_direction: "row",
                    align_items: "start",
                    div {
                        id: "parse_leftcol",
                        width: "50%",
                        display: "flex",
                        justify_content: "center",
                        flex_direction: "column",
                        align_items: "start",

                        InputValues { variables, input_values }
                        span {class: "numberResult",
                            {num_result_formatted}
                        }
                        pre {class: "errMsg", 
                            {eval_msg}
                        }
                        // InputValDebug { input_values }
                    }
                    // div {
                    //     id: "parse_rightcol",
                    //     width: "50%",
                    //     display: "flex",
                    //     justify_content: "center",
                    //     flex_direction: "column",
                    //     align_items: "start",
                    // }
                }
            }
            TabContent { index: 2usize, value: "tab3".to_string(),
                div {
                    width: "100%",
                    height: "5rem",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    "LOL good one!"
                }
            }
        }
    }
}


#[component]
fn InputList(mut variables: Signal<Vec<String>>) -> Element {
    rsx!{
        h3 {"Variables"}
        for (i, varname) in variables.iter().enumerate() {
            div { 
                class: "hstack",
                input {
                    class: "varInput",
                    value: variables.read()[i].clone(),
                    oninput: move |event: Event<FormData>|  { 
                        variables.write()[i] = event.value();
                    }
                }
                button { id: "var{i}_{varname}" ,
                 "×"}
            }
        }
        button { id: "add", onclick: move |_| variables.push("Name".to_string()), "New variable" }
    }
}

#[component]
fn ExpressionInput(mut raw_expression: Signal<String>) -> Element {
    rsx! {
        h3 {"Expression"}
        input {class: "exprInput",
            value: "{raw_expression}",
            oninput: move |event: Event<FormData>|  { 
                raw_expression.set(event.value());
            }
        }
    }
}

#[component]
fn LexerOutput(tokens: Signal<Vec<TokenContext>>, lexer_msg: Signal<String>) -> Element {
    rsx! {
        h3 {"Lexer"}
        div {
            class: "tokenList hstack",
            for t in tokens() {
                HoverCard {
                    HoverCardTrigger {
                        span { class:token_style(&t), 
                        { format!("{}", t.token)} }
                    }
                    HoverCardContent { side: ContentSide::Bottom,
                        div { padding: "1rem", "todo!" }
                    }
                }
            }
        }
        pre {class: "errMsg", 
            {lexer_msg}
        }
    }
}

#[component]
fn InputValues(
    variables: Signal<Vec<String>>, 
    input_values: Signal<HashMap<String, f64>> 
) -> Element {
    rsx! {
        div {
            class: "vstack",
            h3 {"Input values"}
            div {
                class: "vstack",

                for varname in variables().into_iter() {
                    div {
                        padding: "3px", 
                        class: "hstack",
                        label { text_align: "center", width: "30%", "{varname}" }

                        input {
                            class: "numberInput",
                            type: "number",
                            step: "any",
                            value: float_formatter(input_values.read().get(&*varname)),
                            oninput: move |event: Event<FormData>|  { 
                                input_values.write().insert(
                                    varname.to_string(),
                                    event.value().parse::<f64>().unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InputValDebug(
    input_values: Signal<HashMap<String, f64>> 
) -> Element {
    let mut text = String::new();
    for (k, v) in input_values.read().iter() {
        text.push_str(&format!("{} = {} \n", k, v));
    } 
    rsx!{
        pre {
            padding: "20px",
            {text}
        }
    }
}