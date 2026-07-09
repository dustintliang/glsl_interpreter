use std::collections::HashMap;
use crate::ast::{Expr, Program, Stmt};

// The runtime types the interpreter works with
#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Vec3([f32; 3]),
    Vec4([f32; 4])
}

// Execute every statement in main() in order, storing results back into env
pub fn run(program: &Program, env: &mut HashMap<String, Value>) {
    for stmt in &program.stmts {
        match stmt {
            // Assign and Decl both evaluate the right side and store it
            Stmt::Assign {name, expr} | Stmt::Decl {name, expr} => {
                let val = eval(expr, env);
                env.insert(name.clone(), val);
            }
            // Read-modify-write a single component in place
            Stmt::SwizzleAssign {name, field, expr} => {
                let delta = as_float(eval(expr, env));
                let idx = swizzle_index(field);
                match env.get_mut(name).unwrap_or_else(|| panic!("undefined: {name}")) {
                    Value::Vec3(v) => v[idx] += delta,
                    _ => panic!("{name} is not a vec3")
                }
            }
        }
    }
}

// Recursively evaluate an expression down to a Value
fn eval(expr: &Expr, env: &HashMap<String, Value>) -> Value {
    match expr {
        // A literal number (ex. 0.5 or 1.0)
        Expr::Float(f) => Value::Float(*f),

        // Look up the variable name in env
        Expr::Ident(name) => env
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("undefined variable: {name}")),

        // Read one component of a vector
        Expr::Swizzle {name, field} => {
            let idx = swizzle_index(field);
            match env.get(name).cloned().unwrap_or_else(|| panic!("undefined: {name}")) {
                Value::Vec3(v) => Value::Float(v[idx]),
                Value::Vec4(v) => Value::Float(v[idx]),
                _ => panic!("{name} is not a vector")
            }
        }

        // Evaluate both sides then apply the operator
        Expr::BinOp {op, left, right} => {
            let l = as_float(eval(left, env));
            let r = as_float(eval(right, env));
            Value::Float(match op {
                '+' => l + r,
                '-' => l - r,
                '*' => l * r,
                '/' => l / r,
                _ => panic!("unknown op: {op}")
            })
        }

        // A function call
        Expr::Call {name, args} => {
            let vals: Vec<Value> = args.iter().map(|a| eval(a, env)).collect();
            match name.as_str() {
                "vec4" => {
                    // vec4(vec3, float) — lift a vec3 into homogeneous form
                    if vals.len() == 2 {
                        if let (Value::Vec3(v), Value::Float(w)) = (vals[0].clone(), vals[1].clone()) {
                            return Value::Vec4([v[0], v[1], v[2], w]);
                        }
                    }
                    // vec4(float, float, float, float)
                    let f: Vec<f32> = vals.into_iter().map(as_float).collect();
                    assert_eq!(f.len(), 4, "vec4 needs 4 args");
                    Value::Vec4([f[0], f[1], f[2], f[3]])
                }
                other => panic!("unknown function: {other}")
            }
        }
    }
}

// Map swizzle letter to array index — supports both xyzw and rgba
fn swizzle_index(field: &str) -> usize {
    match field {
        "x" | "r" => 0,
        "y" | "g" => 1,
        "z" | "b" => 2,
        "w" | "a" => 3,
        _ => panic!("unknown swizzle: {field}")
    }
}

// Unwrap a Float or panic
fn as_float(v: Value) -> f32 {
    match v {
        Value::Float(f) => f,
        other => panic!("expected float, got {other:?}")
    }
}
