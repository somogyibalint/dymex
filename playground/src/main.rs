use std::collections::HashMap;
use std::ops::Deref;
use charming::element::Symbol;
use indexmap::{IndexMap};

use dioxus::{prelude::*};
// use dioxus::logger::tracing::{Level, debug, error, info, warn};
use dioxus_primitives::hover_card::{HoverCard, HoverCardTrigger, HoverCardContent};
use dioxus_primitives::ContentSide;
use dioxus_primitives::tabs::{Tabs, TabList, TabContent,TabTrigger};
use dioxus_primitives::select::{Select, SelectGroup, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,};
use dioxus_primitives::context_menu::{ContextMenu, ContextMenuItem, ContextMenuTrigger, ContextMenuContent};
use dioxus_elements::keyboard_types::Key;


use charming::{
    component::{Axis, Toolbox, Feature, DataView, Restore, SaveAsImage, ToolboxDataZoom, DataZoomType},
    element::{AxisType, JsFunction, Tooltip},
    series::Line,
    Chart, WasmRenderer,
};

use dymex::{AST, Category, EvaluationError, Evaluator, InputVars, MermaidStyle, MermaidStyleEnum, TokenContext, TokenStream, styled_ast_graph};
use dymex::DynMath;
use std::rc::Rc;
use dymex::Latex;

static CSS: Asset = asset!("/assets/style.css");


// TODO move this to config?
// static START_EXPR: &str = "(1-R)^2 / (1 - 2*R*cos(4*pi*n*d*cos(theta) / lambda) + R^2)";
// static START_VAR: [&str; 5] = ["lambda", "n","d", "theta", "R"];

// const START_EXPR: &str = "1 / (1 + exp((E - E_f) / (k_b * T)))";
// const START_VAR: [&str; 4] = ["E", "E_f","k_b", "T"];
// const START_VAL: [&str; 4] = ["0.2", "0.0", "8.617333262E-5", "297.0"];


const START_EXPR: &str = "(1 + alpha * cos(2*pi*x*omega) * exp(-x**2 / delta)) / sqrt(1 + x**2) * cos(omega*sqrt(x+3))**2";
// const START_EXPR: &str = "(1 + alpha * cos(2*pi*x*omega) )";
const START_VAR: [&str; 4] = ["x", "alpha","omega", "delta"];
const START_VAL: [&str; 4] = ["!linspace(0, 20, 2000)", "0.6", "3.5", "10"];

const MACRO_LABELS : [&str; 4] = ["grid", "linspace", "rand", "normal"];
const MACRO_DEFAULTS : [&str; 4] = ["!grid(0.0, 1.0, 0.005)", "!linspace(0.0, 1.0, 200)", "!rand(100)", "!normal(100)"];


mod helpers;
use helpers::*;
mod eval;
use eval::*;




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
    // let mut evaluator =  EvaluatorAdapter::new();

    let raw_expression = use_signal(|| START_EXPR.to_string());
    let variables : IndexMap<String, VarData> = START_VAR
        .into_iter()
        .zip(START_VAL.into_iter())
        .map(|(key, _value)| {
            return (key.to_string(), VarData::from_text(_value));
        }).collect();
    let variables = use_signal(|| variables );
    let mut referenced_variables: Signal<Vec<String>> = use_signal(|| START_VAR.iter().map(|s| s.to_string()).collect());

    let mut tokens = use_signal(|| Vec::new());
    let mut evaluator: Signal<Option<Evaluator>> = use_signal(|| None);

    let mut lexer_msg = use_signal(|| "".to_string());
    let mut parser_msg = use_signal(|| "".to_string());
    let mut eval_msg = use_signal(|| "".to_string());
    let mut console_text = use_signal(|| Vec::new() );

    let mut mermaid_script = use_signal(|| "".to_string());
    let mut mermaid_innerHTML = use_signal(|| "".to_string());
    let mut latex_tex = use_signal(|| "".to_string());

    let mut valid_expression = use_signal(|| false);
    let mut show_graph= use_signal(|| false);


    let mut num_result = use_signal(|| Some(f64::NAN));
    let mut vec_result = use_signal(|| None::<Vec<f64>>);
    let mut valid_x_axes= use_signal(|| Vec::new());
    let mut x_axis_name = use_signal(|| None::<String>);
    let mut x_axis: Signal<Option<Vec<f64>>> = use_signal(|| None);
    let num_result_formatted = use_memo(move || format_num_result(num_result()));



    // Handle updates to the expression and variable names
    use_effect(move || {
        valid_expression.set(false);
        // info!("{}", valid_expression());
        // TODO: find better way
        let _v = variables.read();
        let varnames: Vec<&str> = _v.iter().map(|(k, _)| k.as_ref()).collect();

        // lexing
        let lexer_result = tokenstream.update(&raw_expression(), &varnames);
        lexer_msg.set(
            match &lexer_result {
                Ok(_) => "✓".to_string(),
                Err(err) => err.user_message().full_message(&raw_expression())
            }
        );
        if let Ok(_) = lexer_result {
            let varnames = tokenstream.variable_names();
            referenced_variables.set(varnames);
        }
        tokens.set(tokenstream.tokens.clone());

        // parsing
        if let Ok(_) = lexer_result {
            let mut ast = AST::new(tokenstream.clone());

            match ast.parse_tokens() {
                Ok(_) => {
                    parser_msg.set("✓".to_string());
                    if let Some(branch) = &ast.tree {
                        mermaid_script.set(styled_ast_graph(branch, &mmd_style));
                        latex_tex.set(format!("{}", branch.latex().replace("⋅", " ")));
                        valid_expression.set(true);
                    }
                },
                Err(err) => {
                    parser_msg.set(err.user_message().full_message(&raw_expression()));
                    mermaid_script.set("".to_string());
                    latex_tex.set("".to_string());
                }
            }
            if valid_expression() {
                evaluator.set(Some(Evaluator::from_ast(ast)));
            } else {
                evaluator.set(None);
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

    // Update evaluated value
    use_effect(move || {
        if !valid_expression() {
            num_result.set(None);
            return;
        }

        // collect the values of referenced variables
        let mut input = InputVars::new();
        for _name in referenced_variables.read().iter() {
            if let Some(var)= variables.read().get(_name) {
                match &var.value {
                    Some(_value) => {
                        input.insert_ref(_name.clone(), Rc::clone(_value));
                    },
                    None => {return}
                }
            } else {
                return
            }
        }
        // debug
        console_text.write().clear();
        for (k , v) in input.iter() {
            match v.category() {
                Category::Number => {
                    console_text.write().push(format!("{:?}: {:?}\n", k, v.as_number())); //DEBUG
                }
                Category::Array => {
                    let v = v.as_any().downcast_ref::<Vec<f64>>().unwrap().to_vec();
                    console_text.write().push(format!("{:?}: {:?}..{:?}\n", k, v[0], v[v.len()-1])); //DEBUG
                }
                _ => {}
            }
        }

        if let Some(mut eval) = evaluator() {
            match eval.evaluate(&input) {
                Ok(x) => {
                    eval_msg.set("✓".to_string());
                    console_text.write().push(format!("result: {:?}", x.category())); //DEBUG
                    match x.category() {
                        Category::Number => {
                            num_result.set(Some(x.as_number()));
                            vec_result.set(None);
                            show_graph.set(false);
                        },
                        Category::Array => {
                            let v = x.as_any().downcast_ref::<Vec<f64>>().unwrap().to_vec();
                            num_result.set(None);
                            vec_result.set(Some(v));
                            show_graph.set(true);
                        },
                        _ => {}
                    }
                },
                Err(err) => {
                    num_result.set(None);
                    vec_result.set(None);
                    eval_msg.set(format!("{}", err));
                }
            };
        }

    });

    // Update valid x axis option: same length arrays as the current result
    use_effect(move || {
        valid_x_axes.write().clear();

        if let Some(v) = vec_result.read().deref(){
            let array_len = v.len();
            let mut tmp = variables.read()
            .iter()
            .filter(|&(_, v)| matches!(&v.value, Some(var) if
                matches!(var.category(), Category::Array) && var.shape()[0] == array_len))
            .map(|(k, _)| k.clone()).collect::<Vec<String>>();
        tmp.push("Indices".to_string());
        valid_x_axes.set(tmp);
        } else {
            valid_x_axes.write().push("Indices".to_string());
        }
    });

    // update x axis
    use_effect(move|| {
        let xax = match x_axis_name.read().deref() {
            Some(name) => {
                match valid_x_axes.read().contains(name) {
                    true => {
                        if let Some(v) = variables.read().get(name) {
                            if let Some(val) = &v.value {
                                Some(val.as_any().downcast_ref::<Vec<f64>>().unwrap().to_vec())
                            } else { None }
                        } else { None }
                    },
                    false => None
                }
            },
            None => None
        };
        x_axis.set(xax)
    });

    // charming
    let renderer = use_signal(|| WasmRenderer::new_opt(None, Some(600)).theme(charming::theme::Theme::Vintage));
    use_effect(move || {
        let (xax, yax) = match vec_result() {
            Some(yax) => {
                match x_axis() {
                    Some(xax) => (xax, yax),
                    None => (yax.iter().enumerate().map(|(i, _)| i as f64).collect(), yax)
                }
            }
            None => return,
        };
        let series = xax.iter().zip(yax.iter()).map(|(x, y)|vec![*x, *y]).collect::<Vec<Vec<f64>>>();

        let chart = Chart::new()
                .tooltip(Tooltip::new().formatter(JsFunction::new_with_args(
                "params",
                r#"
                    var tooltip = "Value: ".concat(String(params.value));
                    return tooltip;
                "#,
            )))
            .x_axis(Axis::new().type_(AxisType::Value))
            .y_axis(Axis::new())
            .series(Line::new().symbol(Symbol::None).data(series))
            .animation(false).toolbox(
                Toolbox::new().feature(
                    Feature::new()
                    .data_view(DataView::new().read_only(false))
                    .data_zoom(ToolboxDataZoom::new())
                    .restore(Restore::new())
                    .save_as_image(SaveAsImage::new()),
                ));
        // TODO: log error?
        let _ = renderer.read_unchecked().render("chart", &chart);
    });

    // X axis selector
    // ! deleting the currently selected option does not affect the selection
    let result_display = match show_graph() {
        false => { rsx! {
            div {
                width: "40%",
                span {
                    class: "numberResult",
                    {num_result_formatted}
                    }
                }
            }
        },
        true => {
            let default = match x_axis_name() {
                Some(selected) => Some(selected),
                None => valid_x_axes.read().first().map(|s| s.to_string())
                // None => options.first().map(|s| s.to_string())
            };
            let vxax = valid_x_axes.read();
            let options = vxax.iter().enumerate().map(|(i, f)| {
                rsx! {
                    SelectOption::<String> { index: i, value: f.clone(), {format!("{}", f)}
                    SelectItemIndicator { "✓" }
                    }
                 }
                });
            rsx! {
                div {
                    width: "100%",
                    div {
                        id: "chart",
                        style: "display: inline-block;",
                        width: "100%",
                    },
                    div {
                        Select::<String> {
                            placeholder: "Select variable as X axis",
                            default_value: default,
                            on_value_change: move |value| x_axis_name.set(value) ,
                            SelectTrigger {
                                aria_label: "Select X Trigger",
                                class: "x_select_btn",
                                SelectValue {} }
                            SelectList { aria_label: "Select X",
                                SelectGroup { {options} }
                            }
                        }
                    },
                }
            }
        },
    };

    rsx! {
        document::Stylesheet { href: CSS }

        div { id: "title",
            text_align: "center",
            h1 { "Dymex playground" }
        }

        Tabs {
            default_value: "tab1".to_string(),
            horizontal: true,
            align_content: "center",
            class: "tabs",
            TabList {
                justify_content: "center",
                class: "tabs-list",
                TabTrigger { class: "tabs-trigger", value: "tab1".to_string(), index: 0usize, "Calculator" }
                TabTrigger { class: "tabs-trigger", value: "tab5".to_string(), index: 5usize, "Debug" }
                TabTrigger { class: "tabs-trigger", value: "tab7".to_string(), index: 7usize, "Tab3" }
            }
            TabContent {
                index: 0usize,
                value: "tab1".to_string(),
                div {
                    id: "calculator",
                    display: "flex",
                    flex_direction: "column",
                    justify_content: "center",
                    align_items: "center",
                    div {
                        id: "calculator_inputs",
                        width: "100%",
                        display: "flex",
                        flex_direction: "row",
                        align_items: "start",
                        justify_content: "center",
                        gap: "20px",
                        div {
                            display: "flex",
                            flex_direction: "column",
                            width: "60%",
                            ExpressionInput { raw_expression }
                            div {
                                width: "100%",
                                div {
                                    id: "latexOutput",
                                    class: "renderLatex"
                                }
                            }
                        }
                        div {width: "40%",
                            display: "flex",
                            flex_direction: "column",
                            h3 {"Inputs"}
                            div {class: "varInputList",
                                InputList { variables }
                            }
                        }
                    }
                    // numeric results / graph
                    div {width: "100%",
                        display: "flex",
                        flex_direction: "column",
                        justify_content: "center",
                        {result_display}
                    }
                    // debug message stream
                    Console { messages:console_text }
                }
            }
            TabContent {
                index: 5usize,
                value: "tab5".to_string(),
                div {
                    width: "100%",
                    // height: "5rem",
                    display: "flex",
                    flex_direction: "row",
                    align_items: "start",
                    justify_content: "center",
                    div {id: "parse_leftcol",
                        width: "50%",
                        display: "flex",
                        flex_direction: "column",
                        justify_content: "center",
                        // div { ExpressionInput { raw_expression } }
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
                            dangerous_inner_html: "{mermaid_innerHTML}"
                        }
                    }
                }
            }
            TabContent {
                index: 7usize,
                value: "tab7".to_string(),
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
fn InputList(mut variables: Signal<IndexMap<String, VarData>>) -> Element {
    rsx!{
        for (var_name, _) in variables().into_iter() {
            div {InputElement {variables: variables, var_name: var_name}
            }
        }
        button {
            class: "addVariable",
            onclick: move |_| {
                for i in 1..100 {
                    let _name = format!("var{i}");
                    if !variables.read().contains_key(&_name) {
                        variables.write().insert(_name, VarData{text: "1".to_string(), value: Some(Rc::new(1.0f64))});
                        break;
                    }
                }
            },
        "New variable"
        }
    }
}

#[component]
fn InputElement(mut variables: Signal<IndexMap<String, VarData>>, var_name: String)  -> Element {
    let varname = use_signal(|| var_name.clone());

    let mut buffer =  use_signal(|| variables.read().get(&var_name).unwrap().text.clone());
    let value_input = match variables.read().get(&var_name).unwrap().value {
        None => "invalidValueBox",
        _ => "validValueBox",
    };

    // generate context menu
    let context_menu_list = MACRO_LABELS
    .into_iter()
    .zip(MACRO_DEFAULTS.into_iter())
    .enumerate()
    .map(|(i, (label, default_value))| {
        rsx! {
            ContextMenuItem {
                value: default_value.to_string(),
                index: i,
                on_select: move |value: String| {
                    let _value = parse_variable_value(&value);
                    variables.write().insert(varname(), VarData { text: value, value: _value});
                },
                {format!("{}", label)}
            }
        }
    });

    rsx!{
        div {
            class: "inputElement",
            input {
                class: "varInput",
                value: var_name.clone(),
                oninput: move |event: Event<FormData>| {
                    let new_name = event.value();
                    if new_name == var_name {
                        // do nothing
                    } else {
                        let mut new_map : IndexMap<String, VarData> = IndexMap::new();
                        match variables.read().contains_key(&new_name) {
                            true => { }, //TODO: emit warning?
                            false => {
                                for (_name, data) in variables().into_iter() {
                                    if _name == *varname.read() {
                                        new_map.insert(new_name.clone(), data.clone());
                                    } else {
                                        new_map.insert(_name.clone(), data.clone());
                                    }
                                }
                            },
                        }
                        variables.set(new_map);
                    }
                }
            }
            ContextMenu {
                class: "ctxMenuArea",
                ContextMenuTrigger {
                    input {
                        class: value_input,
                        type: "text",
                        step: "any",
                        value: variables.read().get(varname.read().deref()).unwrap().text.clone(), // !unwrap, this panic when deletion???
                        onkeypress: move |event: KeyboardEvent| {
                            match event.key() {
                                Key::Enter | Key::Tab => {
                                    let new_text = buffer.peek().clone();
                                    let _value = parse_variable_value(&new_text);
                                    variables.write().insert(varname(), VarData { text: new_text, value: _value});
                                },
                                _ => {}
                            }
                        },
                        oninput: move |event: Event<FormData>| {
                            buffer.set(event.value());
                            variables.write().insert(varname(), VarData { text: event.value(), value: None});
                            buffer.set(event.value());
                        }
                    }
                }
                ContextMenuContent {
                    {context_menu_list}
                }
            }

            button {
                class: "removeVariable",
                onclick: move |_| {
                    variables.write().shift_remove(varname.read().deref());
                },
                "☓"
            }
        }
    }
}

#[component]
fn ExpressionInput(mut raw_expression: Signal<String>) -> Element {
    rsx! {
        h3 {"Expression"}
        textarea {class: "exprInput",
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
fn InputValDebug(input_values: Signal<HashMap<String, f64>>) -> Element {
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


#[component]
fn Console(messages: Signal<Vec<String>>) -> Element {
    rsx!{
        // h3 {"Console"}
        div {
            hidden:"false",
            class: "console",
            for msg in messages.read().iter() {
                pre {class: "consoleMessage",
                    dangerous_inner_html: "<code> {msg.clone()} </code>"
                }
            }
        }
    }
}

// #[component]
// fn XAxisSelector(xaxis_options: Vec<String>) -> Element {
//     let temp = xaxis_options.read();
//     let options = temp.iter().enumerate().map(|(i, f)| {
//         rsx! {
//             SelectOption::<Option<String>> { index: i, value: f.clone(), {format!("{}", f)}
//                 SelectItemIndicator {}
//             }
//         }
//     });
//     let options = xaxis_options.iter().enumerate().map(|(i, f)| {
//         rsx! {
//             SelectOption::<Option<String>> { index: i, value: f.clone(), {format!("{}", f)}
//                 SelectItemIndicator {}
//             }
//         }
//     });

//     rsx! {
//         Select::<Option<String>> { placeholder: "Select variable as X axis",
//             SelectTrigger { aria_label: "Select Trigger", width: "12rem", SelectValue {} }
//             SelectList { aria_label: "Select Demo",
//                 SelectGroup {
//                     SelectGroupLabel { "Fruits" }
//                     {options}
//                     SelectOption::<String> {
//                         index: 0usize,
//                         value: "apple",
//                         "Apple"
//                         SelectItemIndicator { "✔️" }
//                     }
//                 }
//             }
//         }
//     }
// }

