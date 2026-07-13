use std::collections::HashMap;
use crate::ast::{Expr, Program, Stmt};

// The runtime types the interpreter works with
#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4([[f32; 4]; 4])
}

// Execute every statement in main() in order, storing results back into env
pub fn run(program: &Program, env: &mut HashMap<String, Value>, type_env: &mut HashMap<String, String>) {
    for stmt in &program.stmts {
        match stmt {
            Stmt::Assign {name, expr} => {
                let val = eval(expr, env);
                env.insert(name.clone(), val);
            }
            // Record the declared type alongside the runtime value
            Stmt::Decl {ty, name, expr} => {
                let val = eval(expr, env);
                env.insert(name.clone(), val);
                type_env.insert(name.clone(), ty.clone());
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
        Expr::Ident(name) => env.get(name).cloned().unwrap_or_else(|| panic!("undefined variable: {name}")),

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
            let lv = eval(left, env);
            let rv = eval(right, env);
            match (lv, rv) {
                (Value::Float(a), Value::Float(b)) => Value::Float(match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => a / b,
                    _ => panic!("unknown op: {op}")
                }),
                // Scalar * vector, component-wise
                (Value::Float(s), Value::Vec3(v)) if *op == '*' => Value::Vec3([s*v[0], s*v[1], s*v[2]]),
                (Value::Vec3(v), Value::Float(s)) if *op == '*' => Value::Vec3([s*v[0], s*v[1], s*v[2]]),
                (Value::Vec3(a), Value::Vec3(b)) => match op {
                    '+' => Value::Vec3([a[0]+b[0], a[1]+b[1], a[2]+b[2]]),
                    '-' => Value::Vec3([a[0]-b[0], a[1]-b[1], a[2]-b[2]]),
                    _ => panic!("unsupported vec3 op: {op}")
                },
                // Mat4 * Vec4 — row dot column
                (Value::Mat4(m), Value::Vec4(v)) if *op == '*' => Value::Vec4([
                    m[0][0]*v[0] + m[0][1]*v[1] + m[0][2]*v[2] + m[0][3]*v[3],
                    m[1][0]*v[0] + m[1][1]*v[1] + m[1][2]*v[2] + m[1][3]*v[3],
                    m[2][0]*v[0] + m[2][1]*v[1] + m[2][2]*v[2] + m[2][3]*v[3],
                    m[3][0]*v[0] + m[3][1]*v[1] + m[3][2]*v[2] + m[3][3]*v[3]
                ]),
                (Value::Mat4(a), Value::Mat4(b)) if *op == '*' => {
                    let mut c = [[0f32; 4]; 4];
                    for i in 0..4 {
                        for j in 0..4 {
                            for k in 0..4 {
                                c[i][j] += a[i][k] * b[k][j];
                            }
                        }
                    }
                    Value::Mat4(c)
                }
                (l, r) => panic!("unsupported: {l:?} {op} {r:?}")
            }
        }

        // A function call
        Expr::Call {name, args} => {
            let vals: Vec<Value> = args.iter().map(|a| eval(a, env)).collect();
            match name.as_str() {
                "vec3" => {
                    // vec3(float, float, float)
                    if vals.len() == 3 {
                        let f: Vec<f32> = vals.into_iter().map(as_float).collect();
                        return Value::Vec3([f[0], f[1], f[2]]);
                    }
                    // vec3(vec4) — drop w
                    if let Value::Vec4(v) = vals[0].clone() {
                        return Value::Vec3([v[0], v[1], v[2]]);
                    }
                    panic!("vec3: unsupported args")
                }
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
                "normalize" => {
                    if let Value::Vec3(v) = vals.into_iter().next().unwrap() {
                        let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
                        Value::Vec3([v[0]/len, v[1]/len, v[2]/len])
                    } else {
                        panic!("normalize: expected vec3")
                    }
                }
                "dot" => {
                    if let (Value::Vec3(a), Value::Vec3(b)) = (vals[0].clone(), vals[1].clone()) {
                        Value::Float(a[0]*b[0] + a[1]*b[1] + a[2]*b[2])
                    } else {
                        panic!("dot: expected vec3, vec3")
                    }
                }
                "reflect" => {
                    // reflect(I, N) = I - 2*dot(N,I)*N
                    if let (Value::Vec3(i), Value::Vec3(n)) = (vals[0].clone(), vals[1].clone()) {
                        let d = i[0]*n[0] + i[1]*n[1] + i[2]*n[2];
                        Value::Vec3([
                            i[0] - 2.0*d*n[0],
                            i[1] - 2.0*d*n[1],
                            i[2] - 2.0*d*n[2]
                        ])
                    } else {
                        panic!("reflect: expected vec3, vec3")
                    }
                }
                "max" => {
                    let a = as_float(vals[0].clone());
                    let b = as_float(vals[1].clone());
                    Value::Float(a.max(b))
                }
                "pow" => {
                    let a = as_float(vals[0].clone());
                    let b = as_float(vals[1].clone());
                    Value::Float(a.powf(b))
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
