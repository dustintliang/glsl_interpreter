use std::collections::HashMap;
use crate::ast::{Expr, Program, Stmt, TopDecl};
use crate::types::{GatorType, parse_gator_type};

#[derive(Debug)]
pub struct TypeError {
    pub message: String
}

// Entry point — build type map from top-level declarations, then check each statement
pub fn check(program: &Program) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut type_map: HashMap<String, GatorType> = HashMap::new();

    // gl_FragColor is a GLSL built-in output; Unknown means assignments are not Gator-checked
    type_map.insert("gl_FragColor".to_string(), GatorType::Unknown);

    for decl in &program.decls {
        match decl {
            TopDecl::Uniform {ty, name} => { type_map.insert(name.clone(), parse_gator_type(ty)); }
            TopDecl::Varying {ty, name} => { type_map.insert(name.clone(), parse_gator_type(ty)); }
            TopDecl::Precision {..} => {}
        }
    }

    for stmt in &program.stmts {
        check_stmt(stmt, &mut type_map, &mut errors);
    }

    errors
}

fn check_stmt(stmt: &Stmt, type_map: &mut HashMap<String, GatorType>, errors: &mut Vec<TypeError>) {
    match stmt {
        Stmt::Decl {ty, name, expr} => {
            let declared = parse_gator_type(ty);
            let inferred = infer(expr, type_map, errors);
            check_compatible(&declared, &inferred, name, errors);
            type_map.insert(name.clone(), declared);
        }
        Stmt::Assign {name, expr} => {
            let inferred = infer(expr, type_map, errors);
            if let Some(declared) = type_map.get(name).cloned() {
                check_compatible(&declared, &inferred, name, errors);
            }
        }
        Stmt::SwizzleAssign {expr, ..} => { infer(expr, type_map, errors); }
    }
}

// Infer the GatorType of an expression, returns Unknown when type cannot be determined
fn infer(expr: &Expr, type_map: &HashMap<String, GatorType>, errors: &mut Vec<TypeError>) -> GatorType {
    match expr {
        Expr::Float(_) => GatorType::Plain("float".to_string()),
        Expr::Ident(name) => type_map.get(name).cloned().unwrap_or(GatorType::Unknown),
        Expr::Swizzle {..} => GatorType::Plain("float".to_string()),
        Expr::Call {name, args} => infer_call(name, args, type_map, errors),
        Expr::BinOp {op, left, right} => {
            let lt = infer(left, type_map, errors);
            let rt = infer(right, type_map, errors);
            match op {
                '+' | '-' => check_add(&lt, &rt, errors),
                '*' => check_mul(&lt, &rt, errors),
                _ => GatorType::Unknown
            }
        }
    }
}

fn infer_call(name: &str, args: &[Expr], type_map: &HashMap<String, GatorType>, _errors: &mut Vec<TypeError>) -> GatorType {
    // Evaluate args for type info: errors are discarded so pre-existing annotation issues inside call arguments don't surface as new false positives
    let mut sink = Vec::new();
    let arg_types: Vec<GatorType> = args.iter().map(|a| infer(a, type_map, &mut sink)).collect();
    match name {
        "dot" | "length" | "distance" => GatorType::Plain("float".to_string()),
        "max" | "min" | "pow" | "clamp" | "mix" | "smoothstep" | "step" => GatorType::Plain("float".to_string()),
        // Frame-preserving: return same scheme+frames as first arg, subtype not tracked
        "normalize" | "reflect" | "faceforward" => {
            match arg_types.first() {
                Some(GatorType::Gator {scheme, frames, ..}) =>
                    GatorType::Gator {scheme: scheme.clone(), subtype: None, frames: frames.clone()},
                _ => GatorType::Unknown
            }
        }
        _ => GatorType::Unknown
    }
}

fn check_add(lt: &GatorType, rt: &GatorType, errors: &mut Vec<TypeError>) -> GatorType {
    match (lt, rt) {
        (GatorType::Unknown, _) | (_, GatorType::Unknown) => GatorType::Unknown,
        (GatorType::Plain(_), GatorType::Plain(_)) => lt.clone(),
        _ if lt == rt => lt.clone(),
        _ => {
            errors.push(TypeError {message: format!("type mismatch in addition: {:?} + {:?}", lt, rt)});
            GatorType::Unknown
        }
    }
}

fn check_mul(lt: &GatorType, rt: &GatorType, errors: &mut Vec<TypeError>) -> GatorType {
    match (lt, rt) {
        (GatorType::Unknown, _) | (_, GatorType::Unknown) => GatorType::Unknown,
        // float * Gator -> Gator (scalar multiplication, frame preserved)
        (GatorType::Plain(s), GatorType::Gator {..}) if s == "float" => rt.clone(),
        (GatorType::Gator {..}, GatorType::Plain(s)) if s == "float" => lt.clone(),
        // Plain * Plain - both unannotated, produce unannotated result
        (GatorType::Plain(_), GatorType::Plain(_)) => GatorType::Plain("unannotated".to_string()),
        // Gator * Gator -> apply frame rules
        (GatorType::Gator {frames: lf, ..}, GatorType::Gator {frames: rf, ..}) => {
            if lf.len() == 2 && rf.len() == 1 {
                // Matrix<A,B> * vector<A> -> vector<B>
                if let GatorType::Gator {scheme: rs, subtype: rsub, ..} = rt {
                    if lf[0] == rf[0] {
                        GatorType::Gator {scheme: rs.clone(), subtype: rsub.clone(), frames: vec![lf[1].clone()]}
                    } else {
                        errors.push(TypeError {message: format!("frame mismatch in multiplication: matrix input '{}' but vector frame '{}'", lf[0], rf[0])});
                        GatorType::Unknown
                    }
                } else { unreachable!() }
            } else if lf.len() == 2 && rf.len() == 2 {
                // Matrix<A,B> * Matrix<B,C> -> Matrix<A,C>
                if let GatorType::Gator {scheme: ls, subtype: lsub, ..} = lt {
                    if lf[1] == rf[0] {
                        GatorType::Gator {scheme: ls.clone(), subtype: lsub.clone(), frames: vec![lf[0].clone(), rf[1].clone()]}
                    } else {
                        errors.push(TypeError {message: format!("frame mismatch in matrix * matrix: '{}' != '{}'", lf[1], rf[0])});
                        GatorType::Unknown
                    }
                } else { unreachable!() }
            } else {
                GatorType::Unknown
            }
        }
        // Annotated * unannotated (non-scalar) or vice versa -> error
        _ => {
            errors.push(TypeError {message: format!("cannot multiply {:?} by {:?}: cannot mix annotated and unannotated types", lt, rt)});
            GatorType::Unknown
        }
    }
}

fn check_compatible(declared: &GatorType, inferred: &GatorType, name: &str, errors: &mut Vec<TypeError>) {
    match (declared, inferred) {
        (_, GatorType::Unknown) | (GatorType::Unknown, _) => {}
        // Any plain type is assignable to any plain variable since GLSL handles that check
        (GatorType::Plain(_), GatorType::Plain(_)) => {}
        (a, b) if a == b => {}
        // Normalize/reflect return Gator with no subtype — check scheme+frames only
        (GatorType::Gator {scheme: ds, frames: df, ..}, GatorType::Gator {scheme: is, subtype: None, frames: inf})
            if ds == is && df == inf => {}
        _ => {
            errors.push(TypeError {message: format!("type mismatch for '{name}': declared {:?} but expression has type {:?}", declared, inferred)});
        }
    }
}
