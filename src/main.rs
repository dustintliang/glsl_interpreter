use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

mod ast;
mod interpreter;

// Pest reads grammar.pest at compile time and generates the parser code
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GlslParser;

fn main() {
    // Expect: glsl_interpreter <shader.frag> <inputs.json>
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: glsl_interpreter <shader.frag> <inputs.json>");
        std::process::exit(1);
    }

    let frag_src = std::fs::read_to_string(&args[1]).expect("could not read .frag file");
    let json_src = std::fs::read_to_string(&args[2]).expect("could not read .json file");

    // Load JSON inputs into env
    let json: serde_json::Value = serde_json::from_str(&json_src).expect("invalid JSON");
    let mut env: HashMap<String, interpreter::Value> = HashMap::new();
    if let Some(obj) = json.as_object() {
        for (k, v) in obj {
            // Expected_output is only for testing
            if k == "expected_output" {
                continue;
            }
            if let Some(f) = v.as_f64() {
                env.insert(k.clone(), interpreter::Value::Float(f as f32));
            } else if let Some(arr) = v.as_array() {
                let floats: Vec<f32> = arr.iter()
                    .filter_map(|x| x.as_f64())
                    .map(|f| f as f32)
                    .collect();
                match floats.len() {
                    3 => { env.insert(k.clone(), interpreter::Value::Vec3([floats[0], floats[1], floats[2]])); }
                    4 => { env.insert(k.clone(), interpreter::Value::Vec4([floats[0], floats[1], floats[2], floats[3]])); }
                    _ => {}
                }
            }
        }
    }

    // Parse the .frag file into a raw parse tree, then convert to AST, then run it
    let pairs = GlslParser::parse(Rule::program, &frag_src).expect("parse error");
    let program = ast::parse_program(pairs);
    interpreter::run(&program, &mut env);

    // gl_FragColor is what the shader outputs — printed as [r, g, b, a]
    match env.get("gl_FragColor") {
        Some(interpreter::Value::Vec4(v)) => println!("[{}, {}, {}, {}]", v[0], v[1], v[2], v[3]),
        _ => eprintln!("gl_FragColor not set")
    }
}
